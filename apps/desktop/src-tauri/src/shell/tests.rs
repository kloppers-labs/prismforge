use prismforge_core::automation::{RuleAction, Trigger};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer};

use super::*;
use crate::temp::TempDir;

struct Quiet;

impl ViewSink for Quiet {
    fn deliver(&self, _: &ShellView) {}
}

async fn marked(dir: &TempDir) -> Shell<Quiet> {
    let shell = Shell::new(
        Quiet,
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    shell.start(DemoBackend::new()).await.expect("start");
    settle().await;
    let target = OutputTarget {
        device: DeviceRef::new("demo-graphics-card"),
        zone: None,
    };
    let rainbow = EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    };
    shell.apply_effect(target, rainbow).await.expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    crate::automation::Automation::new(crate::system_events::EventSupport::NotOnThisSystem)
        .set_rule(
            &shell,
            Trigger::Connects,
            true,
            RuleAction::ApplyProfile { profile: id },
        )
        .await
        .expect("mark");
    settle().await;
    shell
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(500)).await;
}

async fn report_of(shell: &Shell<Quiet>) -> Arc<Report<Quiet>> {
    let active = shell.active.lock().await;
    let report = active.as_ref().and_then(|active| active.report.clone());
    report.expect("a report")
}

#[tokio::test(start_paused = true)]
async fn a_report_taken_before_a_disconnect_publishes_nothing_after_it() {
    let dir = TempDir::new("late-report-stop");
    let shell = marked(&dir).await;
    let report = report_of(&shell).await;

    shell.stop().await;
    report.publish().await;

    assert_eq!(shell.view().on_connect, OnConnect::NotApplicable);
}

#[tokio::test(start_paused = true)]
async fn a_report_taken_before_a_switch_leaves_the_new_session_state() {
    let dir = TempDir::new("late-report-switch");
    let shell = marked(&dir).await;
    let report = report_of(&shell).await;
    let server = FakeServer::new();

    shell
        .start(OpenRgbBackend::new(
            server.connector(),
            ClientConfig::default(),
        ))
        .await
        .expect("switch");
    settle().await;
    let fresh = shell.view().on_connect;
    report.publish().await;

    assert!(!matches!(fresh, OnConnect::Fired { next: Some(_), .. }));
    assert_eq!(shell.view().on_connect, fresh);
}
