use std::time::Duration;

use prismforge_core::automation::{RuleAction, RunFailure, RunOutcome, Trigger};
use prismforge_core::profile::ProfileId;
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{
    DemoBackend, DemoControls, DemoOutput, FakeServer, FixtureDevice, FixtureMode, wire,
};
use tokio::time::Instant;

use super::*;
use crate::shell::ShellConfig;
use crate::stores::Stores;
use crate::temp::TempDir;
use crate::view::{ApplyingProfile, CommandError, ShellView};

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
    let (shell, id, _) = demo_watched(dir, rules).await;
    (shell, id)
}

async fn demo_watched(
    dir: &TempDir,
    rules: &[(Trigger, bool)],
) -> (Shell<Quiet>, ProfileId, DemoControls) {
    let shell = shell(dir);
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
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
    (shell, id, controls)
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

    assert_eq!(started.elapsed(), SLEEP_LIMIT - RECORD_LIMIT);
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
async fn the_session_end_rule_runs_once_and_the_second_signal_gets_the_same_run() {
    let dir = TempDir::new("rule-once");
    let (shell, _) = demo_with(&dir, &[(Trigger::SessionEnds, true)]).await;
    let automation = Automation::new(EventSupport::Available);
    let first = automation.at_session_end(&shell).await;
    assert!(first.is_some());
    assert_eq!(automation.at_session_end(&shell).await, first);
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

fn dark_frames(controls: &DemoControls) -> usize {
    controls
        .outputs()
        .iter()
        .filter(|output| match output {
            DemoOutput::DeviceFrame { colors, .. } => colors
                .iter()
                .all(|color| *color == prismforge_core::Rgb::new(0, 0, 0)),
            _ => false,
        })
        .count()
}

#[tokio::test(start_paused = true)]
async fn rules_give_up_on_a_busy_session_slot_at_their_limit_and_never_write_later() {
    let dir = TempDir::new("rule-slot-busy");
    let (shell, _, controls) = demo_watched(
        &dir,
        &[
            (Trigger::Sleeps, true),
            (Trigger::SessionEnds, true),
            (Trigger::Wakes, false),
        ],
    )
    .await;
    let automation = Automation::new(EventSupport::Available);
    let held = shell.current().await.0.expect("a session");

    let stop = shell.stop();
    let rules = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let started = Instant::now();
        let slept = automation.at_sleep(&shell).await.expect("a sleep run");
        assert!(started.elapsed() <= SLEEP_LIMIT, "{:?}", started.elapsed());
        assert!(
            matches!(
                slept.outcome,
                RunOutcome::Failed {
                    failure: RunFailure::TimedOut { .. }
                }
            ),
            "{slept:?}"
        );
        let started = Instant::now();
        crate::exit::end_of_session(&shell, &automation, &crate::exit::ExitGuard::default()).await;
        assert!(
            started.elapsed() <= crate::exit::LAST_STOP_LIMIT,
            "{:?}",
            started.elapsed()
        );
        let started = Instant::now();
        automation.at_wake(&shell).await;
        assert!(
            started.elapsed()
                <= prismforge_core::automation::WAKE_RECONNECT_LIMIT + WAKE_APPLY_LIMIT,
            "{:?}",
            started.elapsed()
        );
        let dark = dark_frames(&controls);
        drop(held);
        dark
    };
    let (_, dark) = tokio::join!(stop, rules);
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert_eq!(dark_frames(&controls), dark, "a rule wrote after its limit");
}

#[tokio::test(start_paused = true)]
async fn a_user_command_during_lights_off_is_refused_with_its_own_reason() {
    let dir = TempDir::new("rule-lights-off-reason");
    let (shell, _) = demo_with(&dir, &[]).await;
    let rule = shell.lock.begin_rule().await;
    let target = OutputTarget {
        device: DeviceRef::new("demo-memory-1"),
        zone: None,
    };
    assert_eq!(
        shell.apply_effect(target, rainbow()).await,
        Err(CommandError::RuleRunning)
    );
    drop(rule);
}

#[test]
fn a_profile_whose_frame_could_not_be_kept_never_reads_as_simply_applied() {
    let applied = RunOutcome::Applied {
        profile: ProfileId::parse("p-a").expect("id"),
        name: "Glow".into(),
        report: prismforge_core::profile::ApplyReport {
            frame_rate: prismforge_core::profile::FrameRateOutcome::Applied { in_use: 30 },
            items: Vec::new(),
        },
        missing: Vec::new(),
        not_kept: None,
    };
    let refused = prismforge_core::session::OutputError::Session(
        prismforge_core::session::SessionError::Busy,
    );
    let RunOutcome::Applied { not_kept, .. } = run::kept(applied.clone(), Err(refused.clone()))
    else {
        panic!("not applied");
    };
    assert_eq!(not_kept, Some(refused));
    assert_eq!(run::kept(applied.clone(), Ok(())), applied);
}

#[tokio::test(start_paused = true)]
async fn a_store_another_prismforge_holds_publishes_why_no_rule_could_run() {
    let dir = TempDir::new("rule-store-in-use");
    let (first, _) = demo_with(&dir, &[(Trigger::Sleeps, true)]).await;
    first.stop().await;
    let second = shell(&dir);
    let automation = Automation::new(EventSupport::Available);

    let run = automation.at_sleep(&second).await.expect("a published run");

    assert!(matches!(
        run.outcome,
        RunOutcome::Failed {
            failure: RunFailure::StoreUnavailable { .. }
        }
    ));
    assert_eq!(second.view().rule_run, Some(run));
    drop(first);
}

#[tokio::test(start_paused = true)]
async fn a_run_the_store_could_not_write_is_shown_on_automation() {
    let dir = TempDir::new("rule-run-unwritten");
    let (shell, _) = demo_with(&dir, &[(Trigger::Quits, true)]).await;
    std::fs::create_dir_all(
        dir.path()
            .join("demo-store")
            .join("rule-runs.json")
            .join("blocker"),
    )
    .expect("a folder in the file's place");
    let automation = Automation::new(EventSupport::Available);

    automation.at_quit(&shell).await.expect("a run");
    let view = automation.view(&shell).await.expect("view");

    assert!(
        view.notices
            .iter()
            .any(|notice| notice.file == "rule-runs.json"
                && matches!(
                    notice.event,
                    prismforge_core::store::NoticeEvent::WriteFailed { .. }
                ))
    );
    let quits = view
        .rules
        .iter()
        .find(|rule| rule.trigger == Trigger::Quits)
        .expect("quit rule");
    assert!(quits.last_run.is_some());
}

#[tokio::test(start_paused = true)]
async fn a_failed_run_stays_unseen_until_the_user_saw_it() {
    let dir = TempDir::new("rule-run-seen");
    let (shell, id) = demo_with(&dir, &[(Trigger::Sleeps, true)]).await;
    let automation = Automation::new(EventSupport::Available);
    let users_apply = shell
        .lock
        .try_begin(busy_apply(&id))
        .await
        .expect("user apply");
    automation.at_sleep(&shell).await.expect("a run");
    drop(users_apply);

    let unseen = automation.view(&shell).await.expect("view");
    let seen = automation
        .mark_runs_seen(&shell, vec![Trigger::Sleeps])
        .await
        .expect("seen");

    let sleeps = |view: &AutomationView| {
        view.rules
            .iter()
            .find(|rule| rule.trigger == Trigger::Sleeps)
            .and_then(|rule| rule.last_run.as_ref())
            .map(|run| run.seen)
    };
    assert_eq!(sleeps(&unseen), Some(false));
    assert_eq!(sleeps(&seen), Some(true));
}
