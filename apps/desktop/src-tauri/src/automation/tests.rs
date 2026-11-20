use std::time::Duration;

use prismforge_core::automation::{RuleAction, RunFailure, RunOutcome, Trigger};
use prismforge_core::profile::ProfileId;
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice, FixtureMode, wire};
use tokio::time::Instant;

use super::*;
use crate::shell::ShellConfig;
use crate::stores::Stores;
use crate::temp::TempDir;
use crate::view::{ApplyingProfile, ShellView};

struct Quiet;

impl ViewSink for Quiet {
    fn deliver(&self, _: &ShellView) {}
}

fn shell(dir: &TempDir) -> Shell<Quiet> {
    Shell::new(
        Quiet,
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    )
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(500)).await;
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

/// A demo shell with one saved profile and the given rules switched on.
async fn demo_with(dir: &TempDir, rules: &[(Trigger, bool)]) -> (Shell<Quiet>, ProfileId) {
    let shell = shell(dir);
    shell.start(DemoBackend::new()).await.expect("start");
    settle().await;
    let target = OutputTarget {
        device: DeviceRef::new("demo-graphics-card"),
        zone: None,
    };
    shell.apply_effect(target, rainbow()).await.expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    let automation = Automation::new(EventSupport::Available);
    for (trigger, lights_off) in rules {
        let action = if *lights_off {
            RuleAction::LightsOff
        } else {
            RuleAction::ApplyProfile {
                profile: id.clone(),
            }
        };
        automation
            .set_rule(&shell, *trigger, true, action)
            .await
            .expect("rule");
    }
    (shell, id)
}

fn busy_apply(id: &ProfileId) -> ApplyingProfile {
    ApplyingProfile {
        id: id.clone(),
        name: "Glow".into(),
        automatic: false,
    }
}

#[tokio::test(start_paused = true)]
async fn a_sleep_rule_waits_a_bounded_time_for_a_users_apply_and_records_that_it_did_not_run() {
    let dir = TempDir::new("rule-busy");
    let (shell, id) = demo_with(&dir, &[(Trigger::Sleeps, true)]).await;
    let automation = Automation::new(EventSupport::Available);
    let users_apply = shell
        .lock
        .try_begin(busy_apply(&id))
        .await
        .expect("user apply");

    let started = Instant::now();
    let run = automation.at_sleep(&shell).await.expect("a run");

    assert_eq!(started.elapsed(), RULE_LOCK_WAIT);
    assert_eq!(
        run.outcome,
        RunOutcome::Failed {
            failure: RunFailure::Busy
        }
    );
    assert_eq!(shell.view().rule_run, Some(run));
    drop(users_apply);
}

#[tokio::test(start_paused = true)]
async fn a_user_command_during_a_rule_is_refused_while_the_rule_holds_the_apply_lock() {
    let dir = TempDir::new("rule-holds");
    let (shell, id) = demo_with(&dir, &[(Trigger::Wakes, false)]).await;
    let rule = shell
        .lock
        .begin(Some(ApplyingProfile {
            automatic: true,
            ..busy_apply(&id)
        }))
        .await;
    let target = OutputTarget {
        device: DeviceRef::new("demo-memory-1"),
        zone: None,
    };
    assert_eq!(
        shell.apply_effect(target, rainbow()).await,
        Err(CommandError::ProfileApplying)
    );
    drop(rule);
}

#[tokio::test(start_paused = true)]
async fn a_sleep_rule_that_cannot_finish_gives_up_at_the_sleep_limit() {
    let dir = TempDir::new("rule-timeout");
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8)
            .modes(vec![FixtureMode::direct()]),
    );
    let shell = shell(&dir);
    shell
        .start(OpenRgbBackend::new(
            server.connector(),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    settle().await;
    let automation = Automation::new(EventSupport::Available);
    automation
        .set_rule(&shell, Trigger::Sleeps, true, RuleAction::LightsOff)
        .await
        .expect("rule");
    server.silence(wire::SET_CUSTOM_MODE);

    let started = Instant::now();
    let run = automation.at_sleep(&shell).await.expect("a run");

    assert_eq!(started.elapsed(), SLEEP_LIMIT);
    assert_eq!(
        run.outcome,
        RunOutcome::Failed {
            failure: RunFailure::TimedOut {
                limit_ms: SLEEP_LIMIT.as_millis() as u64
            }
        }
    );
}

#[tokio::test(start_paused = true)]
async fn the_session_end_rule_runs_once_whichever_signal_comes_first() {
    let dir = TempDir::new("rule-once");
    let (shell, _) = demo_with(&dir, &[(Trigger::SessionEnds, true)]).await;
    let automation = Automation::new(EventSupport::Available);
    assert!(automation.at_session_end(&shell).await.is_some());
    assert!(automation.at_session_end(&shell).await.is_none());
}

#[tokio::test(start_paused = true)]
async fn a_rule_that_is_off_does_nothing_and_records_nothing() {
    let dir = TempDir::new("rule-off");
    let (shell, _) = demo_with(&dir, &[]).await;
    let automation = Automation::new(EventSupport::Available);
    for trigger in [Trigger::Sleeps, Trigger::Wakes, Trigger::Quits] {
        let run = match trigger {
            Trigger::Sleeps => automation.at_sleep(&shell).await,
            Trigger::Wakes => automation.at_wake(&shell).await,
            _ => automation.at_quit(&shell).await,
        };
        assert_eq!(run, None, "{trigger:?}");
    }
    assert_eq!(shell.view().rule_run, None);
}

#[tokio::test(start_paused = true)]
async fn a_newer_sleep_cancels_the_wake_rule_of_the_earlier_wake() {
    let dir = TempDir::new("rule-wake-cancel");
    let (shell, _) = demo_with(&dir, &[(Trigger::Wakes, false)]).await;
    let automation = Automation::new(EventSupport::Available);
    let wake = automation.at_wake(&shell);
    let sleep = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        automation.at_sleep(&shell).await
    };
    let (woke, _) = tokio::join!(wake, sleep);
    assert_eq!(woke, None);
}
