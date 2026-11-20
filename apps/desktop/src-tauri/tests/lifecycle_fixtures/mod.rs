use std::time::Duration;

use prismforge_core::session::SessionConfig;
use prismforge_core::store::CloseAction;
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_desktop::{
    AppSettingChange, CommandError, Lifecycle, LifecycleView, Shell, ShellConfig, ShellView,
    Stores, TrayState, ViewSink,
};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice};

use crate::Module;
use crate::support::ConnectsRule;
use crate::support::lifecycle::{FakeEntry, FakeExit, FakeWindow, LifecycleRecorder};
use crate::support::temp::TempDir;

const FOLDER: &str = "store-folder";

struct Silent;

impl ViewSink for Silent {
    fn deliver(&self, _: &ShellView) {}
}

struct Setup {
    shell: Shell<Silent>,
    lifecycle: Lifecycle<LifecycleRecorder>,
    entry: FakeEntry,
    window: FakeWindow,
    exit: FakeExit,
    dir: TempDir,
}

async fn setup(label: &str, tray: bool, session: SessionConfig) -> Setup {
    let dir = TempDir::new(label);
    let shell = Shell::new(
        Silent,
        ShellConfig {
            session,
            ..ShellConfig::default()
        },
        Stores::new(dir.path().to_path_buf()),
    );
    let entry = FakeEntry::default();
    let lifecycle = Lifecycle::new(LifecycleRecorder::default(), entry.clone());
    lifecycle.set_tray(if tray {
        TrayState::Available
    } else {
        TrayState::Unavailable {
            detail: "The system has no tray area.".into(),
        }
    });
    let window = FakeWindow::default();
    lifecycle.show(&shell, &window).await;
    Setup {
        shell,
        lifecycle,
        entry,
        window,
        exit: FakeExit::default(),
        dir,
    }
}

fn stable(mut view: LifecycleView) -> LifecycleView {
    view.folder = FOLDER.into();
    view
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

async fn with_effects(shell: &Shell<Silent>) {
    shell.start(DemoBackend::new()).await.expect("demo");
    tokio::time::sleep(Duration::from_millis(200)).await;
    for device in ["demo-graphics-card", "demo-motherboard"] {
        shell
            .apply_effect(whole(device), rainbow())
            .await
            .expect("effect");
    }
}

fn error<T: std::fmt::Debug>(result: Result<T, CommandError>) -> CommandError {
    result.expect_err("a refusal")
}

async fn choices(views: &mut Module, errors: &mut Module) {
    let s = setup("life-fresh", true, SessionConfig::default()).await;
    views.add(
        "lifecycleFresh",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
    errors.add(
        "settingNoAddress",
        &error(
            s.lifecycle
                .change(&s.shell, AppSettingChange::ConnectAtStart(true))
                .await,
        ),
    );
    s.lifecycle
        .close_requested(&s.shell, &s.window, &s.exit)
        .await;
    views.add(
        "lifecycleAsking",
        &stable(s.lifecycle.publish(&s.shell).await),
    );

    let s = setup("life-chosen", true, SessionConfig::default()).await;
    s.lifecycle
        .remember_address(&s.shell, "192.168.1.20", 6742)
        .await
        .expect("address");
    for change in [
        AppSettingChange::OnClose(CloseAction::Background),
        AppSettingChange::StartWithSystem(true),
        AppSettingChange::StartHidden(true),
        AppSettingChange::ConnectAtStart(true),
    ] {
        s.lifecycle.change(&s.shell, change).await.expect("change");
    }
    views.add(
        "lifecycleChosen",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
    s.lifecycle.hide(&s.shell, &s.window).await;
    views.add(
        "lifecycleHidden",
        &stable(s.lifecycle.publish(&s.shell).await),
    );

    let s = setup("life-quit", true, SessionConfig::default()).await;
    s.lifecycle
        .change(&s.shell, AppSettingChange::OnClose(CloseAction::Quit))
        .await
        .expect("quit");
    views.add(
        "lifecycleQuitChosen",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
}

async fn problems(views: &mut Module, errors: &mut Module) {
    let s = setup("life-no-tray", false, SessionConfig::default()).await;
    views.add(
        "lifecycleNoTray",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
    errors.add(
        "settingTrayUnavailable",
        &error(
            s.lifecycle
                .change(&s.shell, AppSettingChange::OnClose(CloseAction::Background))
                .await,
        ),
    );

    let s = setup("life-entry", true, SessionConfig::default()).await;
    *s.entry.refuse.lock().expect("entry") = Some("Access is denied.".into());
    errors.add(
        "settingStartEntryRefused",
        &error(
            s.lifecycle
                .change(&s.shell, AppSettingChange::StartWithSystem(true))
                .await,
        ),
    );
    *s.entry.refuse.lock().expect("entry") = None;
    s.entry
        .ignore
        .store(true, std::sync::atomic::Ordering::SeqCst);
    errors.add(
        "settingStartEntryNotKept",
        &error(
            s.lifecycle
                .change(&s.shell, AppSettingChange::StartWithSystem(true))
                .await,
        ),
    );
    *s.entry.unreadable.lock().expect("entry") = Some("No home folder was found.".into());
    views.add(
        "lifecycleEntryUnknown",
        &stable(s.lifecycle.publish(&s.shell).await),
    );

    let s = setup("life-machine", true, SessionConfig::default()).await;
    s.entry
        .machine
        .store(true, std::sync::atomic::Ordering::SeqCst);
    views.add(
        "lifecycleMachineEntry",
        &stable(s.lifecycle.publish(&s.shell).await),
    );

    let s = setup("life-quitting", true, SessionConfig::default()).await;
    s.lifecycle.quit(&s.shell, &s.window, &s.exit).await;
    views.add(
        "lifecycleQuitting",
        &stable(s.lifecycle.publish(&s.shell).await),
    );

    let s = setup("life-in-use", true, SessionConfig::default()).await;
    let second = Shell::new(
        Silent,
        ShellConfig::default(),
        Stores::new(s.dir.path().to_path_buf()),
    );
    let lifecycle = Lifecycle::new(LifecycleRecorder::default(), FakeEntry::default());
    lifecycle.set_tray(TrayState::Available);
    lifecycle.show(&second, &FakeWindow::default()).await;
    views.add("lifecycleInUse", &stable(lifecycle.publish(&second).await));
    let (window, exit) = (FakeWindow::default(), FakeExit::default());
    lifecycle.close_requested(&second, &window, &exit).await;
    errors.add(
        "settingsInUse",
        &error(
            lifecycle
                .answer_close(&second, Some(CloseAction::Background), &window, &exit)
                .await,
        ),
    );

    let s = setup("life-recovered", true, SessionConfig::default()).await;
    s.lifecycle
        .change(&s.shell, AppSettingChange::OnClose(CloseAction::Background))
        .await
        .expect("first");
    s.lifecycle
        .change(&s.shell, AppSettingChange::OnClose(CloseAction::Quit))
        .await
        .expect("second");
    std::fs::write(s.dir.path().join("app-settings.json"), b"{ \"format\"").expect("damage");
    drop(s.shell);
    let reopened = Shell::new(
        Silent,
        ShellConfig::default(),
        Stores::new(s.dir.path().to_path_buf()),
    );
    let lifecycle = Lifecycle::new(LifecycleRecorder::default(), FakeEntry::default());
    lifecycle.set_tray(TrayState::Available);
    lifecycle.show(&reopened, &FakeWindow::default()).await;
    views.add(
        "lifecycleRecovered",
        &stable(lifecycle.publish(&reopened).await),
    );
}

async fn tray_stops(views: &mut Module) {
    let s = setup("life-stop", true, SessionConfig::default()).await;
    with_effects(&s.shell).await;
    s.lifecycle.hide(&s.shell, &s.window).await;
    s.lifecycle.stop_all(&s.shell).await;
    views.add(
        "lifecycleTrayStopped",
        &stable(s.lifecycle.publish(&s.shell).await),
    );

    let busy = SessionConfig {
        command_capacity: 1,
        ..SessionConfig::default()
    };
    let s = setup("life-stop-busy", true, busy).await;
    let backend = DemoBackend::new();
    let controls = backend.controls();
    s.shell.start(backend).await.expect("demo");
    tokio::time::sleep(Duration::from_millis(200)).await;
    s.shell
        .apply_effect(whole("demo-graphics-card"), rainbow())
        .await
        .expect("effect");
    controls.set_call_delay(Duration::from_secs(10));
    let (_, report) = tokio::join!(s.shell.refresh(), s.lifecycle.stop_all(&s.shell));
    assert!(
        report.stops.iter().any(|stop| matches!(
            &stop.result,
            prismforge_desktop::StopResult::Failed {
                error: CommandError::Session(_)
            }
        )),
        "{report:?}"
    );
    views.add(
        "lifecycleTrayStopBusy",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
}

async fn start_path(views: &mut Module) {
    let s = setup("life-path", true, SessionConfig::default()).await;
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    s.shell
        .start(OpenRgbBackend::new(
            server.connector(),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let id = s.shell.create_profile("Evening").await.expect("save").id;
    s.shell.set_apply_on_connect(Some(id)).await.expect("mark");
    s.shell.stop().await;
    s.lifecycle
        .remember_address(&s.shell, "127.0.0.1", 6742)
        .await
        .expect("address");
    for change in [
        AppSettingChange::StartWithSystem(true),
        AppSettingChange::ConnectAtStart(true),
    ] {
        s.lifecycle.change(&s.shell, change).await.expect("change");
    }
    views.add(
        "lifecycleStartPath",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
    prismforge_desktop::Automation::new(prismforge_desktop::EventSupport::NotOnThisSystem)
        .set_rule(
            &s.shell,
            prismforge_core::automation::Trigger::Starts,
            true,
            prismforge_core::automation::RuleAction::LightsOff,
        )
        .await
        .expect("starts rule");
    views.add(
        "lifecycleStartLightsOff",
        &stable(s.lifecycle.publish(&s.shell).await),
    );
}

pub async fn generate(views: &mut Module, errors: &mut Module) {
    start_path(views).await;
    choices(views, errors).await;
    problems(views, errors).await;
    tray_stops(views).await;
}
