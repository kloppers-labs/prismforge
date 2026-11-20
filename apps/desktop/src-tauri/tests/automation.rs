mod support;

use std::sync::Arc;
use std::time::Duration;

use prismforge_core::automation::{
    LightsOffOutcome, OpenRgbAct, OpenRgbSetting, RuleAction, RuleRun, RunFailure, RunOutcome,
    Trigger, WAKE_RECONNECT_LIMIT,
};
use prismforge_core::profile::ProfileId;
use prismforge_core::session::{ReconnectPolicy, SessionConfig};
use prismforge_core::{DeviceRef, OutputTarget, Rgb};
use prismforge_desktop::{
    Automation, Dispatcher, EventSink, EventSupport, ExitGuard, LAST_STOP_LIMIT, OpenRgbCheck,
    RuleProfile, Shell, ShellConfig, Stores, SystemEvent, end_of_session, platform_source,
    quit_with_rule,
};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{
    DemoBackend, DemoControls, DemoOutput, FakeServer, FixtureDevice, FixtureMode, wire,
};
use tokio::time::Instant;

use support::events::FakeEvents;
use support::temp::TempDir;
use support::{Recorder, is_connected};

const GPU: &str = "demo-graphics-card";

fn config() -> ShellConfig {
    ShellConfig {
        session: SessionConfig {
            reconnect: ReconnectPolicy {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(800),
            },
            ..SessionConfig::default()
        },
        view_spacing: Duration::from_millis(50),
        identify_duration: Duration::from_secs(3),
    }
}

fn shell_on(dir: &TempDir) -> (Arc<Shell<Recorder>>, Recorder) {
    let recorder = Recorder::default();
    let stores = Stores::new(dir.path().to_path_buf());
    (
        Arc::new(Shell::new(recorder.clone(), config(), stores)),
        recorder,
    )
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn gpu() -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(GPU),
        zone: None,
    }
}

fn apply(profile: &ProfileId) -> RuleAction {
    RuleAction::ApplyProfile {
        profile: profile.clone(),
    }
}

/// Wires a fake event source to the shell the way the app wires the Windows window.
fn wire(shell: &Arc<Shell<Recorder>>, automation: &Arc<Automation>) -> FakeEvents {
    let events = FakeEvents::default();
    let (shell, automation) = (shell.clone(), automation.clone());
    let dispatcher = Dispatcher::new(tokio::runtime::Handle::current(), move |event| {
        let (shell, automation) = (shell.clone(), automation.clone());
        Box::pin(async move { automation.on_event(&shell, event).await })
    });
    assert_eq!(
        events.source().start(Arc::new(dispatcher)),
        EventSupport::Available
    );
    events
}

async fn ran(recorder: &Recorder, trigger: Trigger) -> RuleRun {
    recorder
        .wait_view(|view| {
            view.rule_run
                .as_ref()
                .is_some_and(|run| run.trigger == trigger)
        })
        .await
        .rule_run
        .expect("a run")
}

async fn demo(shell: &Shell<Recorder>, recorder: &Recorder) -> DemoControls {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_connected).await;
    controls
}

fn gpu_frames(controls: &DemoControls) -> Vec<Vec<Rgb>> {
    controls
        .outputs()
        .into_iter()
        .filter_map(|output| match output {
            DemoOutput::DeviceFrame { device, colors } if device.as_str() == GPU => Some(colors),
            _ => None,
        })
        .collect()
}

#[tokio::test(start_paused = true)]
async fn lights_off_at_sleep_and_a_profile_at_wake_run_from_the_event_source() {
    let dir = TempDir::new("journey");
    let (shell, recorder) = shell_on(&dir);
    let automation = Arc::new(Automation::new(EventSupport::Available));
    let controls = demo(&shell, &recorder).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    let morning = shell.create_profile("Morning").await.expect("save").id;
    automation
        .set_rule(&shell, Trigger::Sleeps, true, RuleAction::LightsOff)
        .await
        .expect("sleep rule");
    let view = automation
        .set_rule(&shell, Trigger::Wakes, true, apply(&morning))
        .await
        .expect("wake rule");
    let wakes = view
        .rules
        .iter()
        .find(|rule| rule.trigger == Trigger::Wakes)
        .expect("wake rule");
    assert_eq!(
        wakes.profile,
        Some(RuleProfile::Ready {
            name: "Morning".into()
        })
    );
    let events = wire(&shell, &automation);

    events.fire(SystemEvent::Sleep);
    let slept = ran(&recorder, Trigger::Sleeps).await;
    let RunOutcome::LightsOff { report } = &slept.outcome else {
        panic!("lights off expected: {slept:?}");
    };
    assert!(report.devices.iter().any(|device| {
        device.device == Some(DeviceRef::new(GPU)) && device.outcome == LightsOffOutcome::Black
    }));
    let dark = gpu_frames(&controls).last().cloned().expect("a frame");
    assert!(dark.iter().all(|color| *color == Rgb::new(0, 0, 0)));
    recorder
        .wait_for(|session| session.output.scene.assignments.is_empty())
        .await;

    events.fire(SystemEvent::Wake);
    let woke = ran(&recorder, Trigger::Wakes).await;
    let RunOutcome::Applied { name, missing, .. } = &woke.outcome else {
        panic!("applied expected: {woke:?}");
    };
    assert_eq!(name, "Morning");
    assert!(missing.is_empty());
    recorder
        .wait_view(|view| {
            view.session
                .as_ref()
                .is_some_and(|session| !session.output.scene.assignments.is_empty())
        })
        .await;
    let runs = automation.view(&shell).await.expect("view").rules;
    assert_eq!(
        runs.iter()
            .filter_map(|rule| rule.last_run.as_ref())
            .map(|run| run.trigger)
            .collect::<Vec<_>>(),
        [Trigger::Sleeps, Trigger::Wakes]
    );
}

fn strip(label: &str) -> FixtureDevice {
    FixtureDevice::new(label, &format!("Strip {label}"))
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .zone("Ring", 8)
}

fn openrgb(server: &FakeServer) -> OpenRgbBackend<impl Connector + use<>> {
    OpenRgbBackend::new(server.connector(), ClientConfig::default())
}

async fn openrgb_with_wake_rule(
    dir: &TempDir,
    server: &FakeServer,
) -> (Arc<Shell<Recorder>>, Recorder, Arc<Automation>) {
    let (shell, recorder) = shell_on(dir);
    shell.start(openrgb(server)).await.expect("start");
    recorder.wait_for(is_connected).await;
    let target = OutputTarget {
        device: shell.view().session.expect("session").devices[0]
            .identity
            .reference()
            .cloned()
            .expect("resolved"),
        zone: None,
    };
    shell.apply_effect(target, rainbow()).await.expect("effect");
    let id = shell.create_profile("Desk").await.expect("save").id;
    let automation = Arc::new(Automation::new(EventSupport::Available));
    automation
        .set_rule(&shell, Trigger::Wakes, true, apply(&id))
        .await
        .expect("wake rule");
    (shell, recorder, automation)
}

#[tokio::test(start_paused = true)]
async fn after_a_wake_the_rule_applies_to_the_devices_that_came_back_and_names_the_others() {
    let dir = TempDir::new("wake-missing");
    let server = FakeServer::new();
    server.add_device(0, strip("a"));
    server.add_device(1, strip("b"));
    let (shell, recorder, automation) = openrgb_with_wake_rule(&dir, &server).await;
    let events = wire(&shell, &automation);
    events.fire(SystemEvent::Sleep);
    tokio::time::sleep(Duration::from_millis(100)).await;

    server.restart(vec![(0, strip("a").into())]);
    events.fire(SystemEvent::Wake);
    let woke = ran(&recorder, Trigger::Wakes).await;

    let RunOutcome::Applied { missing, .. } = &woke.outcome else {
        panic!("applied expected: {woke:?}");
    };
    assert_eq!(missing, &["Strip b".to_owned()]);
}

#[tokio::test(start_paused = true)]
async fn an_openrgb_that_never_comes_back_after_a_wake_is_recorded_at_the_limit() {
    let dir = TempDir::new("wake-never");
    let server = FakeServer::new();
    server.add_device(0, strip("a"));
    let (shell, recorder, automation) = openrgb_with_wake_rule(&dir, &server).await;
    let events = wire(&shell, &automation);

    server.set_protocol_version(None);
    server.disconnect_all();
    let started = Instant::now();
    events.fire(SystemEvent::Wake);
    let woke = ran(&recorder, Trigger::Wakes).await;

    assert!(started.elapsed() >= WAKE_RECONNECT_LIMIT);
    assert!(matches!(
        woke.outcome,
        RunOutcome::Failed {
            failure: RunFailure::OpenRgbNotBack { .. }
        }
    ));
}

#[tokio::test(start_paused = true)]
async fn a_wake_without_a_wake_rule_writes_nothing() {
    let dir = TempDir::new("wake-off");
    let (shell, recorder) = shell_on(&dir);
    let automation = Arc::new(Automation::new(EventSupport::Available));
    let controls = demo(&shell, &recorder).await;
    let events = wire(&shell, &automation);
    let before = controls.outputs().len();
    events.fire(SystemEvent::Sleep);
    events.fire(SystemEvent::Wake);
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert_eq!(controls.outputs().len(), before);
    assert_eq!(shell.view().rule_run, None);
}

#[tokio::test(start_paused = true)]
async fn lights_off_at_quit_stays_dark_after_the_quit_stop() {
    let dir = TempDir::new("quit-dark");
    let (shell, recorder) = shell_on(&dir);
    let automation = Automation::new(EventSupport::Available);
    let controls = demo(&shell, &recorder).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    automation
        .set_rule(&shell, Trigger::Quits, true, RuleAction::LightsOff)
        .await
        .expect("quit rule");

    let mut exited = false;
    quit_with_rule(&shell, &automation, &ExitGuard::default(), || exited = true).await;

    assert!(exited);
    assert!(
        controls
            .outputs()
            .iter()
            .all(|output| !matches!(output, DemoOutput::ModeReapplied { .. })),
        "the quit restored a device"
    );
    let dark = gpu_frames(&controls).last().cloned().expect("a frame");
    assert!(dark.iter().all(|color| *color == Rgb::new(0, 0, 0)));
}

#[tokio::test(start_paused = true)]
async fn a_profile_at_session_end_keeps_its_lights_and_the_stop_fits_the_last_four_seconds() {
    let dir = TempDir::new("session-end");
    let (shell, recorder) = shell_on(&dir);
    let automation = Automation::new(EventSupport::Available);
    let controls = demo(&shell, &recorder).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    automation
        .set_rule(&shell, Trigger::SessionEnds, true, apply(&id))
        .await
        .expect("rule");

    let started = Instant::now();
    assert!(end_of_session(&shell, &automation, &ExitGuard::default()).await);

    assert!(started.elapsed() <= LAST_STOP_LIMIT);
    assert!(
        controls
            .outputs()
            .iter()
            .all(|output| !matches!(output, DemoOutput::ModeReapplied { .. })),
        "the end of the session restored a device"
    );
    assert!(matches!(
        shell.view().rule_run.map(|run| run.outcome),
        Some(RunOutcome::Applied { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn without_session_events_on_this_system_the_end_of_a_session_runs_no_rule() {
    let dir = TempDir::new("session-end-unsupported");
    let (shell, recorder) = shell_on(&dir);
    let automation = Automation::new(EventSupport::NotOnThisSystem);
    demo(&shell, &recorder).await;
    automation
        .set_rule(&shell, Trigger::SessionEnds, true, RuleAction::LightsOff)
        .await
        .expect("rule");
    assert!(end_of_session(&shell, &automation, &ExitGuard::default()).await);
    assert_eq!(shell.view().rule_run, None);
}

#[cfg(not(windows))]
#[test]
fn macos_and_linux_say_that_sleep_wake_and_session_end_are_not_reported() {
    let sink = Dispatcher::new(
        tokio::runtime::Runtime::new()
            .expect("runtime")
            .handle()
            .clone(),
        |_| Box::pin(async {}),
    );
    assert_eq!(
        platform_source().start(Arc::new(sink)),
        EventSupport::NotOnThisSystem
    );
}

#[tokio::test(start_paused = true)]
async fn the_starts_rule_applies_at_the_connect_at_start_in_place_of_the_connects_rule() {
    let dir = TempDir::new("starts-rule");
    let (shell, recorder) = shell_on(&dir);
    let automation = Automation::new(EventSupport::Available);
    demo(&shell, &recorder).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    let start = shell.create_profile("Start").await.expect("save").id;
    let connect = shell.create_profile("Connect").await.expect("save").id;
    automation
        .set_rule(&shell, Trigger::Starts, true, apply(&start))
        .await
        .expect("starts");
    automation
        .set_rule(&shell, Trigger::Connects, true, apply(&connect))
        .await
        .expect("connects");
    shell.stop().await;

    shell
        .start_at_launch(DemoBackend::new())
        .await
        .expect("launch");
    let launched = ran(&recorder, Trigger::Starts).await;
    shell.stop().await;
    shell.start(DemoBackend::new()).await.expect("connect");
    let connected = ran(&recorder, Trigger::Connects).await;

    assert!(matches!(&launched.outcome, RunOutcome::Applied { name, .. } if name == "Start"));
    assert!(matches!(&connected.outcome, RunOutcome::Applied { name, .. } if name == "Connect"));
}

#[tokio::test(start_paused = true)]
async fn a_profile_marked_by_an_earlier_version_still_applies_when_prismforge_connects() {
    let dir = TempDir::new("migrated-mark");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    let id = shell.create_profile("Evening").await.expect("save").id;
    shell.stop().await;
    drop(shell);
    let settings = serde_json::json!({
        "format": "prismforge.settings",
        "schema_version": {"major": 1, "minor": 0},
        "apply_on_connect": id.as_str(),
    });
    std::fs::write(
        dir.path().join("demo-store").join("settings.json"),
        serde_json::to_vec(&settings).expect("json"),
    )
    .expect("write");
    let (shell, recorder) = shell_on(&dir);

    let controls = demo(&shell, &recorder).await;
    let connected = ran(&recorder, Trigger::Connects).await;

    assert!(matches!(&connected.outcome, RunOutcome::Applied { name, .. } if name == "Evening"));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!gpu_frames(&controls).is_empty());
}

#[tokio::test(start_paused = true)]
async fn the_screen_says_when_openrgb_also_acts_on_an_event() {
    let dir = TempDir::new("two-programs");
    let server = FakeServer::new();
    server.add_device(0, strip("a"));
    server.set_settings(
        "ProfileManager",
        r#"{"suspend_profile":{"enabled":true,"name":"Dark"}}"#,
    );
    let (shell, recorder) = shell_on(&dir);
    shell.start(openrgb(&server)).await.expect("start");
    recorder.wait_for(is_connected).await;
    let automation = Automation::new(EventSupport::Available);

    let view = automation.view(&shell).await.expect("view");

    assert!(matches!(view.openrgb, OpenRgbCheck::Read { .. }));
    let sleeps = view
        .rules
        .iter()
        .find(|rule| rule.trigger == Trigger::Sleeps)
        .expect("sleep rule");
    assert_eq!(
        sleeps.openrgb_also,
        [OpenRgbAct {
            setting: OpenRgbSetting::Suspend,
            profile: "Dark".into()
        }]
    );
    assert!(server.received_with_id(252).is_empty());
    assert!(server.received_with_id(253).is_empty());
}

#[tokio::test(start_paused = true)]
async fn both_session_end_signals_share_one_run_and_the_stop_restores_nothing_over_it() {
    let dir = TempDir::new("session-end-both");
    let (shell, recorder) = shell_on(&dir);
    let automation = Arc::new(Automation::new(EventSupport::Available));
    let controls = demo(&shell, &recorder).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    automation
        .set_rule(&shell, Trigger::SessionEnds, true, RuleAction::LightsOff)
        .await
        .expect("rule");
    let events = wire(&shell, &automation);

    let _window = events.fire(SystemEvent::SessionEnd);
    tokio::task::yield_now().await;
    assert!(end_of_session(&shell, &automation, &ExitGuard::default()).await);

    assert!(matches!(
        shell.view().rule_run.map(|run| run.outcome),
        Some(RunOutcome::LightsOff { .. })
    ));
    assert!(
        controls
            .outputs()
            .iter()
            .all(|output| !matches!(output, DemoOutput::ModeReapplied { .. })),
        "the stop restored a device over the session-end rule"
    );
    let dark = gpu_frames(&controls).last().cloned().expect("a frame");
    assert!(dark.iter().all(|color| *color == Rgb::new(0, 0, 0)));
}

fn direct_strip(label: &str) -> FixtureDevice {
    strip(label)
        .modes(vec![FixtureMode::direct(), FixtureMode::breathing()])
        .active_mode(0)
}

#[tokio::test(start_paused = true)]
async fn the_wake_rule_sets_direct_mode_again_after_the_devices_settled() {
    let dir = TempDir::new("wake-full-write");
    let server = FakeServer::new();
    server.add_device(0, direct_strip("a"));
    let (shell, recorder, automation) = openrgb_with_wake_rule(&dir, &server).await;
    let events = wire(&shell, &automation);
    tokio::time::sleep(Duration::from_millis(500)).await;

    events.fire(SystemEvent::Wake);
    ran(&recorder, Trigger::Wakes).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let ids: Vec<u32> = server
        .received()
        .iter()
        .map(|packet| packet.packet_id)
        .collect();
    let last_read = ids
        .iter()
        .rposition(|id| *id == wire::REQUEST_CONTROLLER_COUNT)
        .expect("a device read");
    let last_direct = ids
        .iter()
        .rposition(|id| *id == wire::SET_CUSTOM_MODE)
        .expect("direct mode");
    assert!(
        last_direct > last_read,
        "Direct Mode was not set after the settle"
    );
}

#[tokio::test(start_paused = true)]
async fn a_wake_without_a_sleep_names_no_device_from_an_older_list() {
    let dir = TempDir::new("wake-no-sleep");
    let server = FakeServer::new();
    server.add_device(0, strip("a"));
    server.add_device(1, strip("b"));
    let (shell, recorder, automation) = openrgb_with_wake_rule(&dir, &server).await;
    let events = wire(&shell, &automation);
    events.fire(SystemEvent::Sleep);
    tokio::time::sleep(Duration::from_millis(100)).await;
    server.restart(vec![(0, strip("a").into())]);
    events.fire(SystemEvent::Wake);
    let first = ran(&recorder, Trigger::Wakes).await;

    events.fire(SystemEvent::Wake);
    let second = recorder
        .wait_view(|view| {
            view.rule_run
                .as_ref()
                .is_some_and(|run| run.trigger == Trigger::Wakes && *run != first)
        })
        .await
        .rule_run
        .expect("a run");

    let RunOutcome::Applied { missing, .. } = &second.outcome else {
        panic!("applied expected: {second:?}");
    };
    assert!(missing.is_empty(), "{missing:?}");
}

#[tokio::test(start_paused = true)]
async fn the_connects_rule_can_switch_the_lights_off() {
    let dir = TempDir::new("connect-dark");
    let (shell, recorder) = shell_on(&dir);
    let automation = Automation::new(EventSupport::Available);
    demo(&shell, &recorder).await;
    automation
        .set_rule(&shell, Trigger::Connects, true, RuleAction::LightsOff)
        .await
        .expect("rule");
    shell.stop().await;

    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
    let connected = ran(&recorder, Trigger::Connects).await;

    assert!(matches!(connected.outcome, RunOutcome::LightsOff { .. }));
    let dark = gpu_frames(&controls).last().cloned().expect("a frame");
    assert!(dark.iter().all(|color| *color == Rgb::new(0, 0, 0)));
}

#[tokio::test(start_paused = true)]
async fn a_profile_marked_by_an_earlier_version_applies_at_the_connect_at_start() {
    let dir = TempDir::new("migrated-launch");
    let server = FakeServer::new();
    server.add_device(0, strip("a"));
    let (shell, recorder, _) = openrgb_with_wake_rule(&dir, &server).await;
    let id = shell.create_profile("Evening").await.expect("save").id;
    shell.stop().await;
    drop(shell);
    let settings = serde_json::json!({
        "format": "prismforge.settings",
        "schema_version": {"major": 1, "minor": 0},
        "apply_on_connect": id.as_str(),
    });
    std::fs::write(
        dir.path().join("store").join("settings.json"),
        serde_json::to_vec(&settings).expect("json"),
    )
    .expect("write");
    drop(recorder);
    let (shell, recorder) = shell_on(&dir);
    let before = server.color_writes().len();

    shell
        .start_at_launch(openrgb(&server))
        .await
        .expect("launch");
    let connected = ran(&recorder, Trigger::Connects).await;

    assert!(matches!(&connected.outcome, RunOutcome::Applied { name, .. } if name == "Evening"));
    server.wait_for_color_writes(before + 1).await;
}

#[tokio::test(start_paused = true)]
async fn a_seam_that_could_not_start_says_so_and_no_session_end_rule_runs() {
    let dir = TempDir::new("seam-failed");
    let (shell, recorder) = shell_on(&dir);
    let events = FakeEvents::failing("the event window could not be created");
    let support = events.source().start(Arc::new(Dispatcher::new(
        tokio::runtime::Handle::current(),
        |_| Box::pin(async {}),
    )));
    let automation = Automation::new(support);
    demo(&shell, &recorder).await;
    automation
        .set_rule(&shell, Trigger::SessionEnds, true, RuleAction::LightsOff)
        .await
        .expect("rule");

    assert_eq!(
        automation.view(&shell).await.expect("view").support,
        EventSupport::Failed {
            detail: "the event window could not be created".into()
        }
    );
    assert!(end_of_session(&shell, &automation, &ExitGuard::default()).await);
    assert_eq!(shell.view().rule_run, None);
}

#[test]
fn the_event_thread_waits_for_work_only_as_long_as_its_limit() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_time()
        .build()
        .expect("runtime");
    let dispatcher = Dispatcher::new(runtime.handle().clone(), |event| match event {
        SystemEvent::Sleep => Box::pin(std::future::pending()),
        _ => Box::pin(async {}),
    });
    let limit = Duration::from_millis(50);

    let started = std::time::Instant::now();
    assert!(!dispatcher.event(SystemEvent::Sleep).wait(limit));
    let waited = started.elapsed();
    assert!(waited >= limit && waited < limit * 10, "{waited:?}");
    assert!(
        dispatcher
            .event(SystemEvent::Wake)
            .wait(Duration::from_secs(5))
    );
}
