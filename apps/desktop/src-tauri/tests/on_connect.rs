mod support;

use std::time::Duration;

use prismforge_core::profile::{ApplyError, ItemOutcome};
use prismforge_core::session::{ReconnectPolicy, SessionConfig};
use prismforge_core::store::StoreError;
use prismforge_core::{DeviceRef, LossCause, OutputTarget};
use prismforge_desktop::{
    ConnectOutcome, MarkProblem, Marked, OnConnect, Shell, ShellConfig, ShellView, Stores,
};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, DemoControls, FakeServer, FixtureDevice, wire};

use support::temp::TempDir;
use support::{Recorder, is_connected, is_retrying};

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

fn shell_on(dir: &TempDir) -> (Shell<Recorder>, Recorder) {
    let recorder = Recorder::default();
    let stores = Stores::new(dir.path().to_path_buf());
    (Shell::new(recorder.clone(), config(), stores), recorder)
}

fn openrgb(server: &FakeServer) -> OpenRgbBackend<impl Connector + use<>> {
    OpenRgbBackend::new(server.connector(), ClientConfig::default())
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn armed(name: &str) -> impl Fn(&ShellView) -> bool + use<> {
    let name = name.to_owned();
    move |view| matches!(&view.on_connect, OnConnect::Armed { name: armed, .. } if *armed == name)
}

fn ready(name: &str) -> Marked {
    Marked::Ready { name: name.into() }
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(500)).await;
}

async fn demo_failing(shell: &Shell<Recorder>, recorder: &Recorder) -> DemoControls {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    controls.fail_connect(true);
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_retrying).await;
    controls
}

#[tokio::test(start_paused = true)]
async fn a_connect_whose_first_device_scan_fails_stays_armed_until_the_devices_are_read() {
    let dir = TempDir::new("armed-scan");
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    let (shell, recorder) = shell_on(&dir);
    shell.start(openrgb(&server)).await.expect("start");
    let view = recorder.wait_for(is_connected).await;
    let desk = view.session.expect("session").devices[0]
        .identity
        .reference()
        .cloned()
        .expect("reference");
    shell
        .apply_effect(
            OutputTarget {
                device: desk,
                zone: None,
            },
            rainbow(),
        )
        .await
        .expect("effect");
    let id = shell.create_profile("Desk").await.expect("save").id;
    shell.set_apply_on_connect(Some(id)).await.expect("mark");
    shell.stop().await;
    let stopped = shell.view().on_connect;

    server.silence(wire::REQUEST_CONTROLLER_DATA);
    shell.start(openrgb(&server)).await.expect("start");
    recorder.wait_for(is_retrying).await;
    let retrying = recorder.wait_view(armed("Desk")).await;
    server.unsilence(wire::REQUEST_CONTROLLER_DATA);
    let applied = recorder
        .wait_view(|view| view.connect_apply.is_some())
        .await;
    settle().await;

    assert_eq!(stopped, OnConnect::NotApplicable);
    assert!(retrying.connect_apply.is_none());
    assert_eq!(applied.connect_apply.expect("result").name, "Desk");
    let OnConnect::Fired {
        ran: Some(ran),
        next,
    } = shell.view().on_connect
    else {
        panic!("not fired: {:?}", shell.view().on_connect);
    };
    assert_eq!(ran.name, "Desk");
    assert_eq!(next, Some(ready("Desk")));
}

#[tokio::test(start_paused = true)]
async fn after_it_fired_a_lost_connection_keeps_it_fired() {
    let dir = TempDir::new("fired-loss");
    let (shell, recorder) = shell_on(&dir);
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell
        .select_mode(DeviceRef::new("demo-light-bar"), 1)
        .await
        .expect("mode");
    let id = shell.create_profile("Bar").await.expect("save").id;
    shell.set_apply_on_connect(Some(id)).await.expect("mark");
    settle().await;
    let marked_while_live = shell.view().on_connect;

    controls.fail_connect(true);
    controls.lose_connection(LossCause::Timeout);
    recorder.wait_for(is_retrying).await;
    settle().await;

    assert_eq!(
        marked_while_live,
        OnConnect::Fired {
            ran: None,
            next: Some(ready("Bar"))
        }
    );
    assert_eq!(shell.view().on_connect, marked_while_live);
}

#[tokio::test(start_paused = true)]
async fn marking_before_the_first_live_connection_arms_it_and_unmarking_disarms_it() {
    let dir = TempDir::new("mark-retrying");
    let (shell, recorder) = shell_on(&dir);
    let backend = DemoBackend::new();
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell
        .apply_effect(
            OutputTarget {
                device: DeviceRef::new("demo-graphics-card"),
                zone: None,
            },
            rainbow(),
        )
        .await
        .expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    shell.stop().await;

    let controls = demo_failing(&shell, &recorder).await;
    settle().await;
    let unmarked = shell.view().on_connect;
    shell
        .set_apply_on_connect(Some(id.clone()))
        .await
        .expect("mark");
    let marked = recorder.wait_view(armed("Glow")).await.on_connect;
    shell.set_apply_on_connect(None).await.expect("unmark");
    settle().await;
    let unmarked_again = shell.view().on_connect;
    controls.fail_connect(false);
    recorder.wait_for(is_connected).await;
    settle().await;

    assert_eq!(unmarked, OnConnect::NotArmed);
    assert!(matches!(marked, OnConnect::Armed { .. }));
    assert_eq!(unmarked_again, OnConnect::NotArmed);
    assert_eq!(shell.view().connect_apply, None);
    assert_eq!(
        shell.view().on_connect,
        OnConnect::Fired {
            ran: None,
            next: None
        }
    );
}

#[tokio::test(start_paused = true)]
async fn a_shell_without_its_store_publishes_not_applicable() {
    let dir = TempDir::new("on-connect-no-store");
    let (first, _) = shell_on(&dir);
    let (second, recorder) = shell_on(&dir);
    first.store_view().await.expect("first holds the folder");

    second.start(DemoBackend::new()).await.expect("start");
    recorder.wait_for(is_connected).await;

    assert_eq!(second.view().on_connect, OnConnect::NotApplicable);
}

fn strip(label: &str) -> FixtureDevice {
    FixtureDevice::new(label, &format!("Strip {label}"))
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .zone("Ring", 8)
}

/// Saves a profile with an effect on each of `labels` and marks it to apply on connect.
async fn marked_profile(dir: &TempDir, labels: &[&str]) {
    let server = FakeServer::new();
    for (index, label) in labels.iter().enumerate() {
        server.add_device(index as u32, strip(label));
    }
    let (shell, recorder) = shell_on(dir);
    shell.start(openrgb(&server)).await.expect("start");
    let view = recorder.wait_for(is_connected).await;
    for device in &view.session.expect("session").devices {
        let reference = device.identity.reference().cloned().expect("reference");
        shell
            .apply_effect(
                OutputTarget {
                    device: reference,
                    zone: None,
                },
                rainbow(),
            )
            .await
            .expect("effect");
    }
    let id = shell.create_profile("Pair").await.expect("save").id;
    shell.set_apply_on_connect(Some(id)).await.expect("mark");
    shell.stop().await;
}

async fn fired_with(dir: &TempDir, labels: &[&str]) -> OnConnect {
    let server = FakeServer::new();
    for (index, label) in labels.iter().enumerate() {
        server.add_device(index as u32, strip(label));
    }
    let (shell, recorder) = shell_on(dir);
    shell.start(openrgb(&server)).await.expect("start");
    recorder
        .wait_view(|view| matches!(view.on_connect, OnConnect::Fired { .. }))
        .await
        .on_connect
}

fn skipped_of(state: &OnConnect) -> (usize, usize) {
    let OnConnect::Fired { ran: Some(ran), .. } = state else {
        panic!("not fired with a result: {state:?}");
    };
    let ConnectOutcome::Applied { report } = &ran.outcome else {
        panic!("not applied: {ran:?}");
    };
    let skipped = report
        .items
        .iter()
        .filter(|item| item.outcome != ItemOutcome::Applied)
        .count();
    (skipped, report.items.len())
}

#[tokio::test(start_paused = true)]
async fn the_fired_state_carries_a_partial_result() {
    let dir = TempDir::new("fired-partial");
    marked_profile(&dir, &["alpha", "beta"]).await;

    let state = fired_with(&dir, &["alpha"]).await;

    assert_eq!(skipped_of(&state), (1, 2));
}

#[tokio::test(start_paused = true)]
async fn the_fired_state_carries_a_result_where_every_setting_was_skipped() {
    let dir = TempDir::new("fired-skipped");
    marked_profile(&dir, &["alpha", "beta"]).await;

    let state = fired_with(&dir, &["gamma"]).await;

    assert_eq!(skipped_of(&state), (2, 2));
}

fn profile_file(dir: &TempDir) -> std::path::PathBuf {
    let folder = dir.path().join("store").join("profiles");
    std::fs::read_dir(&folder)
        .expect("profiles")
        .map(|entry| entry.expect("entry").path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("a profile file")
}

#[tokio::test(start_paused = true)]
async fn a_marked_profile_from_a_newer_version_is_unreadable_and_fails_without_writing() {
    let dir = TempDir::new("marked-newer");
    marked_profile(&dir, &["alpha"]).await;
    let file = profile_file(&dir);
    let text = std::fs::read_to_string(&file).expect("profile");
    std::fs::write(&file, text.replacen("\"major\": 1", "\"major\": 2", 1)).expect("newer");

    let server = FakeServer::new();
    server.add_device(0, strip("alpha"));
    server.silence(wire::REQUEST_CONTROLLER_DATA);
    let (shell, recorder) = shell_on(&dir);
    shell.start(openrgb(&server)).await.expect("start");
    let before = recorder
        .wait_view(|view| matches!(view.on_connect, OnConnect::Unreadable { .. }))
        .await
        .on_connect;
    server.unsilence(wire::REQUEST_CONTROLLER_DATA);
    let fired = recorder
        .wait_view(|view| matches!(view.on_connect, OnConnect::Fired { .. }))
        .await
        .on_connect;

    assert!(matches!(
        before,
        OnConnect::Unreadable {
            problem: MarkProblem::NewerVersion,
            ..
        }
    ));
    let OnConnect::Fired { ran: Some(ran), .. } = fired else {
        panic!("{fired:?}");
    };
    assert!(matches!(
        ran.outcome,
        ConnectOutcome::Failed {
            error: ApplyError::Store(StoreError::NewerFile { .. })
        }
    ));
}

#[tokio::test(start_paused = true)]
async fn a_marked_profile_that_is_gone_is_unreadable_as_missing() {
    let dir = TempDir::new("marked-missing");
    marked_profile(&dir, &["alpha"]).await;
    let file = profile_file(&dir);
    std::fs::write(&file, b"damaged").expect("damage");
    let _ = std::fs::remove_file(file.with_extension("json.bak"));

    let server = FakeServer::new();
    server.add_device(0, strip("alpha"));
    server.silence(wire::REQUEST_CONTROLLER_DATA);
    let (shell, recorder) = shell_on(&dir);
    shell.start(openrgb(&server)).await.expect("start");
    let before = recorder
        .wait_view(|view| matches!(view.on_connect, OnConnect::Unreadable { .. }))
        .await
        .on_connect;

    assert!(matches!(
        before,
        OnConnect::Unreadable {
            problem: MarkProblem::Missing,
            ..
        }
    ));
}

#[cfg(unix)]
#[tokio::test(start_paused = true)]
async fn a_failed_delete_of_the_marked_profile_publishes_the_cleared_mark() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new("delete-fails");
    let (shell, recorder) = shell_on(&dir);
    let controls = demo_failing(&shell, &recorder).await;
    controls.fail_connect(false);
    recorder.wait_for(is_connected).await;
    shell
        .apply_effect(
            OutputTarget {
                device: DeviceRef::new("demo-graphics-card"),
                zone: None,
            },
            rainbow(),
        )
        .await
        .expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    shell.stop().await;
    let controls = demo_failing(&shell, &recorder).await;
    shell
        .set_apply_on_connect(Some(id.clone()))
        .await
        .expect("mark");
    recorder.wait_view(armed("Glow")).await;
    let folder = dir.path().join("demo-store").join("profiles");
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).expect("lock");

    let deleted = shell.delete_profile(&id).await;
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).expect("unlock");
    settle().await;

    assert!(deleted.is_err(), "{deleted:?}");
    assert_eq!(shell.view().on_connect, OnConnect::NotArmed);
    drop(controls);
}
