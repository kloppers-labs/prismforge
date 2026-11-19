mod support;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use prismforge_core::profile::{ItemOutcome, ProfileId, SkipReason};
use prismforge_core::session::{ReconnectPolicy, SessionConfig, SessionSnapshot};
use prismforge_core::store::{ApplyOutcome, IMPORT_LIMIT, StoreError};
use prismforge_core::{DeviceRef, LossCause, OutputTarget};
use prismforge_desktop::{
    CommandError, ConnectOutcome, FileRule, Shell, ShellConfig, StoreKind, StoreUnavailable, Stores,
};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, DemoControls, DemoOutput, FakeServer, FixtureDevice};
use tokio::time::sleep;

use support::temp::TempDir;
use support::{Recorder, is_connected, is_retrying};

const GPU: &str = "demo-graphics-card";
const LIGHT_BAR: &str = "demo-light-bar";

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

async fn demo(shell: &Shell<Recorder>, recorder: &Recorder) -> DemoControls {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start demo");
    recorder.wait_for(is_connected).await;
    controls
}

fn openrgb(server: &FakeServer) -> OpenRgbBackend<impl Connector + use<>> {
    OpenRgbBackend::new(server.connector(), ClientConfig::default())
}

fn strip(label: &str) -> FixtureDevice {
    FixtureDevice::new(label, "Fixture Strip")
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .zone("Ring", 8)
}

fn reference(snapshot: &SessionSnapshot, serial: &str) -> DeviceRef {
    snapshot
        .devices
        .iter()
        .find(|device| device.descriptor.serial == serial)
        .and_then(|device| device.identity.reference())
        .cloned()
        .expect("resolved device")
}

fn whole(device: &str) -> OutputTarget {
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

fn mode_calls(controls: &DemoControls, device: &str) -> usize {
    controls
        .outputs()
        .iter()
        .filter(|output| matches!(output, DemoOutput::ModeReapplied { device: written, .. } if written.as_str() == device))
        .count()
}

async fn settle() {
    sleep(Duration::from_millis(500)).await;
}

async fn released(shell: &Shell<Recorder>, recorder: &Recorder) {
    recorder.wait_view(|view| view.applying.is_none()).await;
    assert_eq!(shell.set_frame_rate(30).await, Ok(30));
}

#[tokio::test(start_paused = true)]
async fn the_registry_is_seeded_and_kept_across_an_app_restart() {
    let dir = TempDir::new("restart");
    let other = TempDir::new("restart-unseeded");
    let first = FakeServer::new();
    first.add_device(0, strip("alpha"));
    let (shell, recorder) = shell_on(&dir);
    shell.start(openrgb(&first)).await.expect("start");
    let view = recorder.wait_for(is_connected).await;
    let alpha = reference(view.session.as_ref().expect("session"), "SN-alpha");
    shell.stop().await;
    drop(shell);

    let moved = FakeServer::new();
    moved.add_device(0, strip("alpha").location("usb:moved"));
    let (unseeded, unseeded_recorder) = shell_on(&other);
    unseeded.start(openrgb(&moved)).await.expect("start");
    let unseeded_view = unseeded_recorder.wait_for(is_connected).await;
    let (restarted, recorder) = shell_on(&dir);
    restarted.start(openrgb(&moved)).await.expect("start");
    let view = recorder.wait_for(is_connected).await;

    assert!(dir.files("store").contains(&"devices.json".to_owned()));
    assert_ne!(
        reference(unseeded_view.session.as_ref().expect("session"), "SN-alpha"),
        alpha
    );
    assert_eq!(
        reference(view.session.as_ref().expect("session"), "SN-alpha"),
        alpha
    );
}

#[tokio::test(start_paused = true)]
async fn apply_on_connect_is_off_by_default_and_applies_the_marked_profile_once_per_connect() {
    let dir = TempDir::new("on-connect");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    shell
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("breathing");
    let bar = shell.create_profile("Bar").await.expect("save").id;
    shell
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("effect");
    shell.create_profile("Other").await.expect("save other");
    shell.stop().await;

    let unmarked = demo(&shell, &recorder).await;
    settle().await;
    let default_mark = shell
        .store_view()
        .await
        .expect("store")
        .summary
        .apply_on_connect;
    shell
        .set_apply_on_connect(Some(bar.clone()))
        .await
        .expect("mark");
    settle().await;
    let unmarked_calls = mode_calls(&unmarked, LIGHT_BAR);
    shell.stop().await;

    let backend = DemoBackend::new();
    let controls = backend.controls();
    controls.fail_connect(true);
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_retrying).await;
    settle().await;
    let before_connect = mode_calls(&controls, LIGHT_BAR);
    let report_before_connect = shell.view().connect_apply;
    controls.fail_connect(false);
    let view = recorder
        .wait_view(|view| view.connect_apply.is_some())
        .await;
    for _ in 0..3 {
        shell.refresh().await.expect("refresh");
        settle().await;
    }
    let after_first = mode_calls(&controls, LIGHT_BAR);
    controls.lose_connection(LossCause::Timeout);
    recorder.wait_for(is_retrying).await;
    recorder.wait_for(is_connected).await;
    settle().await;
    let after_reconnect = mode_calls(&controls, LIGHT_BAR);
    let next = demo(&shell, &recorder).await;
    settle().await;

    assert_eq!(default_mark, None);
    assert_eq!(unmarked_calls, 0);
    assert_eq!(before_connect, 0);
    assert_eq!(report_before_connect, None);
    assert_eq!(after_first, 1);
    assert_eq!(after_reconnect, 1);
    assert_eq!(mode_calls(&next, LIGHT_BAR), 1);
    let applied = view.connect_apply.expect("report");
    assert_eq!(applied.name, "Bar");
    assert!(matches!(applied.outcome, ConnectOutcome::Applied { .. }));
    assert_eq!(
        shell.store_view().await.expect("store").summary.active,
        Some(bar)
    );
}

#[tokio::test(start_paused = true)]
async fn output_commands_are_refused_while_a_profile_applies() {
    let dir = TempDir::new("lock");
    let (shell, recorder) = shell_on(&dir);
    let controls = demo(&shell, &recorder).await;
    shell
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    controls.set_call_delay(Duration::from_secs(2));

    let (outcome, refusals) = tokio::join!(shell.apply_profile(&id), async {
        let applying = recorder.wait_view(|view| view.applying.is_some()).await;
        let refusals = vec![
            shell.apply_effect(whole(GPU), rainbow()).await.err(),
            shell.stop_effect(whole(GPU)).await.err(),
            shell.select_mode(DeviceRef::new(LIGHT_BAR), 1).await.err(),
            shell
                .declare_external(DeviceRef::new(GPU), "Plugin".into())
                .await
                .err(),
            shell.clear_external(DeviceRef::new(GPU)).await.err(),
            shell.set_frame_rate(20).await.err(),
            shell.identify(DeviceRef::new(GPU), None).await.err(),
            shell.apply_profile(&id).await.err(),
            shell.create_profile("During").await.err(),
            shell.update_profile(&id).await.err(),
            shell.delete_profile(&id).await.err(),
        ];
        (applying.applying, refusals)
    });
    controls.set_call_delay(Duration::ZERO);

    assert!(
        matches!(outcome, Ok(ApplyOutcome::Applied(_))),
        "{outcome:?}"
    );
    let (applying, refusals) = refusals;
    let applying = applying.expect("applying");
    assert_eq!(
        (applying.name.as_str(), applying.automatic),
        ("Glow", false)
    );
    assert!(
        refusals
            .iter()
            .all(|refusal| refusal == &Some(CommandError::ProfileApplying)),
        "{refusals:?}"
    );
    released(&shell, &recorder).await;
}

#[tokio::test(start_paused = true)]
async fn the_apply_lock_is_released_after_a_partial_apply_an_error_a_cancel_and_a_session_end() {
    let dir = TempDir::new("release");
    let (shell, recorder) = shell_on(&dir);
    let controls = demo(&shell, &recorder).await;
    shell
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;

    shell
        .declare_external(DeviceRef::new(GPU), "Plugin".into())
        .await
        .expect("declare");
    let partial = shell.apply_profile(&id).await;
    released(&shell, &recorder).await;
    shell
        .clear_external(DeviceRef::new(GPU))
        .await
        .expect("hand back");

    let unknown = ProfileId::parse("p-0000000000000000").expect("id");
    let missing = shell.apply_profile(&unknown).await;
    released(&shell, &recorder).await;

    controls.set_call_delay(Duration::from_secs(2));
    let cancelled =
        tokio::time::timeout(Duration::from_millis(100), shell.apply_profile(&id)).await;
    controls.set_call_delay(Duration::ZERO);
    released(&shell, &recorder).await;

    controls.set_call_delay(Duration::from_secs(2));
    let (_, stopped) = tokio::join!(shell.apply_profile(&id), async {
        recorder.wait_view(|view| view.applying.is_some()).await;
        controls.lose_connection(LossCause::Timeout);
        shell.stop().await
    });
    demo(&shell, &recorder).await;
    released(&shell, &recorder).await;

    let Ok(ApplyOutcome::Applied(report)) = partial else {
        panic!("{partial:?}");
    };
    assert!(report.items.iter().any(|item| item.outcome
        == ItemOutcome::Skipped {
            reason: SkipReason::DeclaredExternal {
                label: "Plugin".into()
            }
        }));
    assert!(matches!(
        missing,
        Err(CommandError::Store(StoreError::UnknownProfile { .. }))
    ));
    assert!(cancelled.is_err());
    assert_eq!(stopped.session, None);
}

#[tokio::test(start_paused = true)]
async fn demo_profiles_are_kept_apart_from_real_ones() {
    let dir = TempDir::new("apart");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    shell
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("effect");
    let demo_id = shell.create_profile("Demo look").await.expect("save").id;
    let demo_view = shell.store_view().await.expect("demo store");
    shell.stop().await;
    let idle_view = shell.store_view().await.expect("real store");
    let server = FakeServer::new();
    server.add_device(0, strip("alpha"));
    shell.start(openrgb(&server)).await.expect("start");
    recorder.wait_for(is_connected).await;
    let real_view = shell.store_view().await.expect("real store");
    let refused = shell.apply_profile(&demo_id).await;
    let details = shell.profile_details(&demo_id).await;

    assert_eq!(demo_view.kind, StoreKind::Demo);
    assert_eq!(demo_view.summary.profiles.len(), 1);
    assert!(demo_view.folder.ends_with("demo-store"));
    assert_eq!(idle_view.kind, StoreKind::Real);
    assert!(idle_view.summary.profiles.is_empty());
    assert_eq!(real_view.kind, StoreKind::Real);
    assert!(real_view.summary.profiles.is_empty());
    assert!(matches!(
        refused,
        Err(CommandError::Store(StoreError::UnknownProfile { .. }))
    ));
    assert!(matches!(
        details,
        Err(CommandError::Store(StoreError::UnknownProfile { .. }))
    ));
    assert_eq!(dir.files("demo-store/profiles").len(), 1);
    assert!(dir.files("store/profiles").is_empty());
}

#[tokio::test(start_paused = true)]
async fn export_and_import_keep_to_the_file_rules_and_write_nothing_to_hardware() {
    let dir = TempDir::new("files");
    let outside = TempDir::new("files-outside");
    let (shell, recorder) = shell_on(&dir);
    let controls = demo(&shell, &recorder).await;
    shell
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("breathing");
    let id = shell.create_profile("Evening").await.expect("save").id;
    settle().await;
    let before = controls.outputs().len();
    let stored = dir
        .path()
        .join("demo-store")
        .join("profiles")
        .join(format!("{id}.json"));
    let large = outside.path().join("large.json");
    fs::write(&large, vec![b' '; IMPORT_LIMIT as usize + 1]).expect("large file");
    let target = outside.path().join("evening.json");

    let relative = shell
        .export_profile(&id, PathBuf::from("evening.json"))
        .await;
    let inside = shell
        .export_profile(&id, dir.path().join("evening.json"))
        .await;
    let exported = shell.export_profile(&id, target.clone()).await;
    let too_large = shell.import_profile(large).await;
    let from_store = shell.import_profile(stored).await;
    let imported = shell.import_profile(target.clone()).await.expect("import");
    let name = shell.export_file_name(&id).await;
    settle().await;

    assert_eq!(relative, Err(CommandError::File(FileRule::NotAbsolute)));
    assert_eq!(inside, Err(CommandError::File(FileRule::InStoreFolder)));
    assert_eq!(exported.map(|report| report.omitted_fields), Ok(0));
    let text = fs::read_to_string(&target).expect("exported file");
    assert!(!text.contains("SIMULATED-BAR-01"));
    assert!(!text.contains("Simulated USB port 1"));
    assert_eq!(
        too_large,
        Err(CommandError::File(FileRule::TooLarge {
            limit: IMPORT_LIMIT
        }))
    );
    assert_eq!(from_store, Err(CommandError::File(FileRule::InStoreFolder)));
    assert_eq!(imported.renamed_from.as_deref(), Some("Evening"));
    assert_eq!(name, Ok("Evening.json".to_owned()));
    assert_eq!(controls.outputs().len(), before);
}

#[tokio::test(start_paused = true)]
async fn a_second_instance_on_the_same_folder_does_not_touch_the_store() {
    let dir = TempDir::new("twice");
    let (first, _) = shell_on(&dir);
    let (second, second_recorder) = shell_on(&dir);

    let opened = first.store_view().await;
    let refused = second.store_view().await;
    demo(&second, &second_recorder).await;
    second
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("output still works");
    let saved = second.create_profile("Second").await;
    settle().await;
    let demo_files = dir.files("demo-store");
    drop(first);
    let later = second.store_view().await;

    assert!(opened.is_ok());
    assert_eq!(
        refused.map(|view| view.kind),
        Err(CommandError::StoreUnavailable(StoreUnavailable::InUse))
    );
    assert_eq!(
        saved.map(|saved| saved.id),
        Err(CommandError::StoreUnavailable(StoreUnavailable::InUse))
    );
    assert!(demo_files.is_empty(), "{demo_files:?}");
    assert_eq!(later.map(|view| view.kind), Ok(StoreKind::Demo));
}
