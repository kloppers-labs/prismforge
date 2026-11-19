mod support;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use prismforge_core::session::{ReconnectPolicy, RestoreOutcome, SessionConfig};
use prismforge_core::store::{CloseAction, OpenRgbAddress, ProfileStore, StdFileOps};
use prismforge_core::{BackendOrigin, ConnectionState, DeviceRef, OutputTarget};
use prismforge_desktop::{
    AppSettingChange, CommandError, ConnectOutcome, OnConnect, SettingProblem, SettingsState,
    Shell, ShellConfig, StartEntryState, StartProfile, StopResult, StoreUnavailable, Stores,
    TrayState, stop_then_exit,
};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, DemoOutput, FakeServer, FixtureDevice};
use tokio::io::DuplexStream;
use tokio::time::Instant;

use support::lifecycle::{FakeEntry, FakeExit, FakeWindow, lifecycle};
use support::temp::TempDir;
use support::{Recorder, is_connected, is_retrying, stores};

const ADDRESS: &str = "127.0.0.1:6742";
const GPU: &str = "demo-graphics-card";
const BOARD: &str = "demo-motherboard";

struct Local(FakeServer);

impl Connector for Local {
    type Stream = DuplexStream;

    async fn connect(&self) -> io::Result<DuplexStream> {
        Ok(self.0.connect())
    }

    fn address(&self) -> Option<String> {
        Some(ADDRESS.into())
    }
}

struct Refusing;

impl Connector for Refusing {
    type Stream = DuplexStream;

    async fn connect(&self) -> io::Result<DuplexStream> {
        Err(io::Error::new(io::ErrorKind::ConnectionRefused, "refused"))
    }

    fn address(&self) -> Option<String> {
        Some(ADDRESS.into())
    }
}

fn config() -> ShellConfig {
    ShellConfig {
        session: SessionConfig {
            reconnect: ReconnectPolicy {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(800),
            },
            ..SessionConfig::default()
        },
        ..ShellConfig::default()
    }
}

fn shell(label: &str) -> (Shell<Recorder>, Recorder, TempDir) {
    let recorder = Recorder::default();
    let (stores, dir) = stores(label);
    (
        Shell::new(recorder.clone(), config(), stores),
        recorder,
        dir,
    )
}

fn target(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn frames(shell: &Shell<Recorder>) -> u64 {
    shell
        .view()
        .session
        .map_or(0, |session| session.diagnostics.counters.frames_written)
}

async fn demo_with_effect(shell: &Shell<Recorder>, recorder: &Recorder) {
    shell.start(DemoBackend::new()).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell
        .apply_effect(target(GPU), rainbow())
        .await
        .expect("effect");
}

fn on_close(view: &prismforge_desktop::LifecycleView) -> Option<CloseAction> {
    match &view.settings {
        SettingsState::Ready { settings, .. } => settings.on_close,
        SettingsState::Unavailable { .. } => panic!("settings unavailable"),
    }
}

#[tokio::test(start_paused = true)]
async fn closing_hides_the_window_and_effects_keep_running_when_the_user_chose_the_tray() {
    let (shell, recorder, _dir) = shell("close-hide");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;
    demo_with_effect(&shell, &recorder).await;
    lifecycle
        .change(&shell, AppSettingChange::OnClose(CloseAction::Background))
        .await
        .expect("choose background");

    lifecycle.close_requested(&shell, &window, &exit).await;
    let before = frames(&shell);
    tokio::time::sleep(Duration::from_secs(2)).await;

    assert!(!window.is_visible());
    assert_eq!(exit.requested(), 0);
    assert!(!views.last().window_visible);
    assert!(!views.last().close_prompt);
    assert!(frames(&shell) > before + 30, "{before} {}", frames(&shell));
    assert!(shell.view().session.is_some());
}

#[tokio::test(start_paused = true)]
async fn the_first_close_asks_and_the_answer_is_remembered() {
    let (shell, _recorder, dir) = shell("close-first");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;

    lifecycle.close_requested(&shell, &window, &exit).await;
    let asked = views.last();
    lifecycle
        .answer_close(&shell, Some(CloseAction::Background), &window, &exit)
        .await
        .expect("answer");
    let answered = views.last();
    lifecycle.show(&shell, &window).await;
    lifecycle.close_requested(&shell, &window, &exit).await;

    assert!(asked.close_prompt);
    assert!(asked.window_visible);
    assert!(!answered.close_prompt);
    assert_eq!(on_close(&answered), Some(CloseAction::Background));
    assert!(!views.last().close_prompt);
    assert!(!window.is_visible());
    assert_eq!(exit.requested(), 0);
    let stored = std::fs::read_to_string(dir.path().join("app-settings.json")).expect("file");
    assert!(stored.contains("\"on_close\": \"Background\""), "{stored}");
}

#[tokio::test(start_paused = true)]
async fn cancelling_the_first_close_keeps_the_window_and_chooses_nothing() {
    let (shell, _recorder, _dir) = shell("close-cancel");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;

    lifecycle.close_requested(&shell, &window, &exit).await;
    lifecycle
        .answer_close(&shell, None, &window, &exit)
        .await
        .expect("cancel");

    assert!(window.is_visible());
    assert_eq!(exit.requested(), 0);
    assert!(!views.last().close_prompt);
    assert_eq!(on_close(&views.last()), None);
}

#[tokio::test(start_paused = true)]
async fn quitting_from_the_first_close_requests_one_exit_and_remembers_quit() {
    let (shell, _recorder, _dir) = shell("close-quit");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;

    lifecycle.close_requested(&shell, &window, &exit).await;
    lifecycle
        .answer_close(&shell, Some(CloseAction::Quit), &window, &exit)
        .await
        .expect("answer");
    lifecycle
        .answer_close(&shell, Some(CloseAction::Quit), &window, &exit)
        .await
        .expect("a second answer");

    assert_eq!(exit.requested(), 1);
    assert_eq!(on_close(&views.last()), Some(CloseAction::Quit));
}

#[tokio::test(start_paused = true)]
async fn without_a_tray_closing_quits_and_background_running_is_refused() {
    let (shell, _recorder, _dir) = shell("no-tray");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Unavailable {
        detail: "no status area".into(),
    });
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;

    let background = lifecycle
        .change(&shell, AppSettingChange::OnClose(CloseAction::Background))
        .await;
    let hidden = lifecycle
        .change(&shell, AppSettingChange::StartHidden(true))
        .await;
    lifecycle.close_requested(&shell, &window, &exit).await;

    assert_eq!(
        background,
        Err(CommandError::Setting(SettingProblem::TrayUnavailable))
    );
    assert_eq!(
        hidden,
        Err(CommandError::Setting(SettingProblem::TrayUnavailable))
    );
    assert_eq!(exit.requested(), 1);
    assert!(!views.last().close_prompt);
    assert_eq!(
        views.last().tray,
        TrayState::Unavailable {
            detail: "no status area".into()
        }
    );
}

#[tokio::test(start_paused = true)]
async fn connect_at_start_needs_an_address_the_user_connected_to() {
    let (shell, _recorder, _dir) = shell("connect-at-start");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);

    let early = lifecycle
        .change(&shell, AppSettingChange::ConnectAtStart(true))
        .await;
    lifecycle
        .remember_address(&shell, "192.168.1.20", 6743)
        .await
        .expect("remember");
    let later = lifecycle
        .change(&shell, AppSettingChange::ConnectAtStart(true))
        .await
        .expect("connect at start");

    assert_eq!(early, Err(CommandError::Setting(SettingProblem::NoAddress)));
    let SettingsState::Ready { settings, .. } = later.settings else {
        panic!("settings unavailable");
    };
    assert!(settings.connect_at_start);
    assert_eq!(
        settings.address,
        Some(OpenRgbAddress {
            host: "192.168.1.20".into(),
            port: 6743
        })
    );
    let plan = lifecycle.startup(&shell, false).await;
    assert_eq!(plan.connect, settings.address);
    assert!(plan.show_window);
}

#[tokio::test(start_paused = true)]
async fn a_failed_start_entry_is_reported_and_the_switch_shows_the_real_state() {
    let (shell, _recorder, _dir) = shell("entry-failed");
    let entry = FakeEntry::default();
    *entry.refuse.lock().expect("entry") = Some("access denied".into());
    let (lifecycle, views) = lifecycle(&entry);

    let on = lifecycle
        .change(&shell, AppSettingChange::StartWithSystem(true))
        .await;

    assert_eq!(
        on,
        Err(CommandError::Setting(SettingProblem::StartEntry {
            wanted: true,
            detail: Some("access denied".into())
        }))
    );
    assert_eq!(
        views.last().start_with_system,
        StartEntryState::NotRegistered
    );

    *entry.refuse.lock().expect("entry") = None;
    let view = lifecycle
        .change(&shell, AppSettingChange::StartWithSystem(true))
        .await
        .expect("register");
    assert_eq!(view.start_with_system, StartEntryState::Registered);
    assert!(entry.is_on());
}

#[tokio::test(start_paused = true)]
async fn a_second_start_shows_the_hidden_window() {
    let (shell, recorder, _dir) = shell("second-start");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let window = FakeWindow::default();
    demo_with_effect(&shell, &recorder).await;
    lifecycle.show(&shell, &window).await;
    lifecycle.hide(&shell, &window).await;

    lifecycle.second_instance(&shell, &window).await;

    assert!(window.is_visible());
    assert!(views.last().window_visible);
    assert!(shell.view().session.is_some());
}

#[tokio::test(start_paused = true)]
async fn stop_all_stops_each_device_and_keeps_the_result_for_the_window() {
    let (shell, recorder, _dir) = shell("stop-all");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    demo_with_effect(&shell, &recorder).await;
    shell
        .apply_effect(target(BOARD), rainbow())
        .await
        .expect("second effect");

    let report = lifecycle.stop_all(&shell).await;

    let devices: Vec<&str> = report
        .stops
        .iter()
        .map(|stop| stop.device.as_str())
        .collect();
    assert_eq!(devices, [GPU, BOARD]);
    assert!(report.stops.iter().all(|stop| matches!(
        stop.result,
        StopResult::Stopped {
            restore: Some(RestoreOutcome::Restored)
        }
    )));
    assert!(report.stops.iter().all(|stop| !stop.label.is_empty()));
    assert_eq!(views.last().tray_stop, Some(report));
    let scene = shell.snapshot().await.expect("session").output.scene;
    assert!(scene.assignments.is_empty());
}

#[tokio::test(start_paused = true)]
async fn stop_all_during_a_profile_apply_is_refused_per_device() {
    let (shell, recorder, _dir) = shell("stop-all-apply");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);
    demo_with_effect(&shell, &recorder).await;
    let id = shell.create_profile("Glow").await.expect("save").id;

    let (_, report) = tokio::join!(shell.apply_profile(&id), async {
        recorder.wait_view(|view| view.applying.is_some()).await;
        lifecycle.stop_all(&shell).await
    });

    assert!(!report.stops.is_empty());
    assert!(report.stops.iter().all(|stop| matches!(
        stop.result,
        StopResult::Failed {
            error: CommandError::ProfileApplying
        }
    )));
}

#[tokio::test(start_paused = true)]
async fn the_settings_say_so_when_another_prismforge_holds_the_folder() {
    let (shell, _recorder, dir) = shell("settings-in-use");
    let other = Stores::new(dir.path().to_path_buf());
    other.settings().await.expect("first holds the lock");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;

    let view = lifecycle.view(&shell).await;
    lifecycle.close_requested(&shell, &window, &exit).await;
    let answered = lifecycle
        .answer_close(&shell, Some(CloseAction::Background), &window, &exit)
        .await;

    assert_eq!(
        view.settings,
        SettingsState::Unavailable {
            reason: StoreUnavailable::InUse
        }
    );
    assert_eq!(
        answered,
        Err(CommandError::StoreUnavailable(StoreUnavailable::InUse))
    );
    assert!(!window.is_visible());
}

#[tokio::test(start_paused = true)]
async fn the_tray_shows_only_what_the_snapshot_and_store_say() {
    let (shell, recorder, _dir) = shell("tray-model");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);
    let window = FakeWindow::default();

    let idle = lifecycle.tray_model(&shell).await;
    demo_with_effect(&shell, &recorder).await;
    shell.create_profile("Glow").await.expect("save");
    lifecycle.show(&shell, &window).await;
    let demo = lifecycle.tray_model(&shell).await;

    assert_eq!(idle.connection, "Not connected");
    assert_eq!(idle.effects, "No PrismForge effects running");
    assert_eq!(idle.profile, "No active profile");
    assert_eq!(idle.toggle, "Show PrismForge window");
    assert!(!idle.stop_enabled);
    assert_eq!(demo.connection, "Demo mode: simulated devices, no hardware");
    assert_eq!(demo.effects, "PrismForge effects running on 1 device");
    assert_eq!(demo.profile, "Active profile: Glow");
    assert_eq!(demo.toggle, "Hide PrismForge window");
    assert!(demo.stop_enabled);
    assert_eq!(
        demo.tooltip,
        "PrismForge: Demo mode: simulated devices, no hardware"
    );
}

#[tokio::test(start_paused = true)]
async fn the_tray_names_the_openrgb_address_and_the_retrying_state() {
    let (shell, recorder, _dir) = shell("tray-openrgb");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip").zone("Ring", 8),
    );

    shell
        .start(OpenRgbBackend::new(
            Local(server.clone()),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    recorder.wait_for(is_connected).await;
    let connected = lifecycle.tray_model(&shell).await;
    shell
        .start(OpenRgbBackend::new(Refusing, ClientConfig::default()))
        .await
        .expect("start");
    recorder.wait_for(is_retrying).await;
    let retrying = lifecycle.tray_model(&shell).await;

    assert_eq!(
        connected.connection,
        "Connected to OpenRGB at 127.0.0.1:6742"
    );
    assert_eq!(
        retrying.connection,
        "OpenRGB at 127.0.0.1:6742 is not reachable, retrying"
    );
}

#[tokio::test(start_paused = true)]
async fn quitting_with_effects_running_releases_the_devices_before_the_exit() {
    let (shell, recorder, _dir) = shell("quit-effects");
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell
        .apply_effect(target(GPU), rainbow())
        .await
        .expect("effect");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut at_exit = None;

    stop_then_exit(&shell, || {
        at_exit = Some((controls.outputs(), shell.view()));
    })
    .await;

    let (outputs, view) = at_exit.expect("exit");
    let last_gpu = outputs.iter().rev().find(|output| match output {
        DemoOutput::DirectMode { device }
        | DemoOutput::ModeReapplied { device, .. }
        | DemoOutput::DeviceFrame { device, .. }
        | DemoOutput::ZoneFrame { device, .. } => device.as_str() == GPU,
    });
    assert!(
        matches!(last_gpu, Some(DemoOutput::ModeReapplied { .. })),
        "{last_gpu:?}"
    );
    assert_eq!(view.session, None);
}

#[tokio::test(start_paused = true)]
async fn quitting_while_the_connection_retries_ends_at_once() {
    let (shell, recorder, _dir) = shell("quit-retry");
    shell
        .start(OpenRgbBackend::new(Refusing, ClientConfig::default()))
        .await
        .expect("start");
    recorder.wait_for(is_retrying).await;
    let started = Instant::now();
    let exited = AtomicBool::new(false);

    stop_then_exit(&shell, || exited.store(true, Ordering::SeqCst)).await;

    assert!(exited.load(Ordering::SeqCst));
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(shell.view().session, None);
}

#[tokio::test(start_paused = true)]
async fn quit_hides_the_window_and_asks_for_one_exit() {
    let (shell, _recorder, _dir) = shell("quit");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.show(&shell, &window).await;

    lifecycle.quit(&window, &exit);

    assert!(!window.is_visible());
    assert_eq!(exit.requested(), 1);
}

#[tokio::test(start_paused = true)]
async fn connect_at_start_retries_with_the_marked_profile_armed_and_never_shows_demo_devices() {
    let (shell, recorder, _dir) = shell("start-connect");
    shell.start(DemoBackend::new()).await.expect("demo");
    recorder.wait_for(is_connected).await;
    shell.stop().await;
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);
    lifecycle
        .remember_address(&shell, "127.0.0.1", 6742)
        .await
        .expect("remember");
    lifecycle
        .change(&shell, AppSettingChange::ConnectAtStart(true))
        .await
        .expect("connect at start");
    let plan = lifecycle.startup(&shell, true).await;

    assert!(plan.connect.is_some());
    shell
        .start(OpenRgbBackend::new(Refusing, ClientConfig::default()))
        .await
        .expect("start");
    let view = recorder.wait_for(is_retrying).await;

    let session = view.session.expect("session");
    assert_eq!(session.backend.origin, BackendOrigin::OpenRgb);
    assert!(session.devices.is_empty());
    assert!(matches!(
        session.connection,
        ConnectionState::Retrying { .. }
    ));
    assert!(matches!(
        shell.view().on_connect,
        OnConnect::NotArmed | OnConnect::Armed { .. }
    ));
}

#[tokio::test(start_paused = true)]
async fn the_store_files_stay_valid_when_the_server_drops_the_session_during_output() {
    let (shell, recorder, dir) = shell("abnormal");
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    shell
        .start(OpenRgbBackend::new(
            Local(server.clone()),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    let connected = recorder.wait_for(is_connected).await;
    let device = connected.session.expect("session").devices[0]
        .identity
        .reference()
        .cloned()
        .expect("resolved");
    shell
        .apply_effect(
            OutputTarget {
                device: device.clone(),
                zone: None,
            },
            rainbow(),
        )
        .await
        .expect("effect");
    let id = shell.create_profile("Desk").await.expect("save").id;
    shell
        .set_device_name(device, Some("Desk strip".into()))
        .await
        .expect("name");

    let (update, ()) = tokio::join!(shell.update_profile(&id), async {
        server.disconnect_all();
    });
    recorder.wait_for(is_retrying).await;
    shell.stop().await;

    assert!(update.is_ok() || matches!(update, Err(CommandError::Store(_))));
    let reopened = ProfileStore::open(dir.path().join("store"), StdFileOps).expect("reopen");
    assert!(reopened.notices().is_empty(), "{:?}", reopened.notices());
    assert_eq!(reopened.profile(&id).expect("profile").name, "Desk");
    assert_eq!(reopened.known_devices().len(), 1);
    let names = reopened.registry().devices.clone();
    assert!(
        names
            .iter()
            .any(|device| device.name.as_deref() == Some("Desk strip"))
    );
}

fn desk_server() -> FakeServer {
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    server
}

/// Saves "Evening" in the real store with an effect on the desk strip and marks it for apply on connect.
async fn marked_evening(shell: &Shell<Recorder>, recorder: &Recorder, server: &FakeServer) {
    shell
        .start(OpenRgbBackend::new(
            Local(server.clone()),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    let connected = recorder.wait_for(is_connected).await;
    let device = connected.session.expect("session").devices[0]
        .identity
        .reference()
        .cloned()
        .expect("resolved");
    shell
        .apply_effect(OutputTarget { device, zone: None }, rainbow())
        .await
        .expect("effect");
    let id = shell.create_profile("Evening").await.expect("save").id;
    shell.set_apply_on_connect(Some(id)).await.expect("mark");
    shell.stop().await;
}

#[tokio::test(start_paused = true)]
async fn the_settings_name_the_real_profile_that_start_up_applies() {
    let (shell, recorder, _dir) = shell("start-profile");
    let entry = FakeEntry::default();
    let (lifecycle, _views) = lifecycle(&entry);

    let none = lifecycle.view(&shell).await.start_profile;
    shell.start(DemoBackend::new()).await.expect("demo");
    recorder.wait_for(is_connected).await;
    let demo = shell.create_profile("Demo glow").await.expect("save").id;
    shell
        .set_apply_on_connect(Some(demo))
        .await
        .expect("mark demo");
    shell.stop().await;
    let demo_only = lifecycle.view(&shell).await.start_profile;
    marked_evening(&shell, &recorder, &desk_server()).await;
    let marked = lifecycle.view(&shell).await.start_profile;

    assert_eq!(none, StartProfile::NotMarked);
    assert_eq!(demo_only, StartProfile::NotMarked);
    assert_eq!(
        marked,
        StartProfile::Marked {
            name: "Evening".into()
        }
    );
}

#[tokio::test(start_paused = true)]
async fn with_every_start_step_on_the_marked_profile_applies_at_system_start_without_a_click() {
    let (shell, recorder, _dir) = shell("start-path");
    let server = desk_server();
    marked_evening(&shell, &recorder, &server).await;
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    lifecycle
        .remember_address(&shell, "127.0.0.1", 6742)
        .await
        .expect("address");
    for change in [
        AppSettingChange::StartWithSystem(true),
        AppSettingChange::StartHidden(true),
        AppSettingChange::ConnectAtStart(true),
    ] {
        lifecycle.change(&shell, change).await.expect("step on");
    }
    let window = FakeWindow::default();
    let connected_to = std::sync::Mutex::new(None);

    let plan = lifecycle
        .begin(&shell, true, &window, |address| async {
            *connected_to.lock().expect("address") = Some(address);
            let _ = shell
                .start(OpenRgbBackend::new(
                    Local(server.clone()),
                    ClientConfig::default(),
                ))
                .await;
        })
        .await;
    let fired = recorder
        .wait_view(|view| matches!(view.on_connect, OnConnect::Fired { .. }))
        .await;

    assert!(!plan.show_window);
    assert_eq!(window.shown.load(Ordering::SeqCst), 0);
    assert!(!views.last().window_visible);
    assert_eq!(
        *connected_to.lock().expect("address"),
        Some(OpenRgbAddress {
            host: "127.0.0.1".into(),
            port: 6742
        })
    );
    let OnConnect::Fired { ran: Some(ran), .. } = fired.on_connect else {
        panic!("{:?}", fired.on_connect);
    };
    assert_eq!(ran.name.as_deref(), Some("Evening"));
    assert!(matches!(ran.outcome, ConnectOutcome::Applied { .. }));
    let running = shell.snapshot().await.expect("session");
    assert_eq!(running.output.scene.assignments.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_start_by_the_user_shows_the_window_and_connects_nothing_by_default() {
    let (shell, _recorder, _dir) = shell("start-user");
    let entry = FakeEntry::default();
    let (lifecycle, views) = lifecycle(&entry);
    lifecycle.set_tray(TrayState::Available);
    let window = FakeWindow::default();
    let mut connects = 0;

    let plan = lifecycle
        .begin(&shell, false, &window, |_| {
            connects += 1;
            async {}
        })
        .await;

    assert!(plan.show_window);
    assert!(window.is_visible());
    assert!(views.last().window_visible);
    assert_eq!(connects, 0);
    assert_eq!(shell.view().session, None);
}
