use std::time::Duration;

use prismforge_core::profile::{
    ApplyError, ApplyReport, DeviceStatus, EffectProblem, ItemOutcome, ProfileId, ProfileItem,
    SkipReason,
};
use prismforge_core::store::{ApplyOutcome, Profiles, StdFileOps};
use prismforge_core::{DeviceRef, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};
use prismforge_testkit::{DemoOutput, FakeServer, FixtureDevice, FixtureMode};
use tokio::time::sleep;

use crate::session::{
    connected, demo, is_connected, output_packets, reference, strip, wait_until, whole,
};
use crate::support::{TempDir, effect};

const RED: Rgb = Rgb::new(255, 0, 0);
const BLUE: Rgb = Rgb::new(0, 0, 255);
const PLUGIN: &str = "OpenRGB Effects plugin";

fn solid(color: Rgb) -> EffectInstance {
    EffectInstance {
        effect: "solid".into(),
        values: [("color".to_owned(), ControlValue::Color(color))].into(),
    }
}

async fn open(dir: &TempDir) -> Profiles {
    Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open store")
}

async fn applied(
    profiles: &Profiles,
    handle: &prismforge_core::session::SessionHandle,
    id: &ProfileId,
) -> ApplyReport {
    match profiles.apply(handle, id).await.expect("apply") {
        ApplyOutcome::Applied(report) => report,
        other => panic!("not applied: {other:?}"),
    }
}

fn outcome_for(report: &ApplyReport, label: &str) -> ItemOutcome {
    report
        .items
        .iter()
        .find(|item| item.label == label)
        .unwrap_or_else(|| panic!("no item for {label}: {report:?}"))
        .outcome
        .clone()
}

fn skipped(reason: SkipReason) -> ItemOutcome {
    ItemOutcome::Skipped { reason }
}

fn colour_writes(server: &FakeServer, session: u32, label: &str) -> Vec<Vec<[u8; 3]>> {
    server
        .color_writes()
        .into_iter()
        .filter(|write| write.session == session && write.target.as_deref() == Some(label))
        .map(|write| write.colors)
        .collect()
}

#[tokio::test(start_paused = true)]
async fn preserves_missing_device_settings() {
    let dir = TempDir::new("missing-device");
    let yesterday = FakeServer::new();
    yesterday.add_device(0, strip("alpha", 3));
    yesterday.add_device(1, strip("beta", 2));
    let profiles = open(&dir).await;
    let first = connected(&yesterday, profiles.known_devices().await).await;
    let alpha = reference(&first.snapshot(), "SN-alpha");
    let beta = reference(&first.snapshot(), "SN-beta");
    first
        .apply_effect(whole(&alpha), solid(RED))
        .await
        .expect("alpha");
    first
        .apply_effect(whole(&beta), solid(BLUE))
        .await
        .expect("beta");
    let id = profiles
        .create_from_session(&first, "Desk")
        .await
        .expect("capture");
    let stored = profiles.profile(&id).await.expect("stored");
    drop(first);
    drop(profiles);

    let today = FakeServer::new();
    today.add_device(0, strip("alpha", 3));
    let profiles = open(&dir).await;
    let handle = connected(&today, profiles.known_devices().await).await;
    let report = applied(&profiles, &handle, &id).await;
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("save");
    drop(profiles);
    let profiles = open(&dir).await;
    let saved = profiles.profile(&id).await.expect("saved");

    assert_eq!(outcome_for(&report, "Strip alpha"), ItemOutcome::Applied);
    assert_eq!(
        outcome_for(&report, "Strip beta"),
        skipped(SkipReason::Missing)
    );
    assert_eq!(saved.device(&beta), stored.device(&beta));
    let beta_settings = |profile: &prismforge_core::profile::Profile| {
        profile
            .assignments
            .iter()
            .filter(|assignment| assignment.device == beta)
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(beta_settings(&saved), beta_settings(&stored));
    assert_eq!(beta_settings(&saved).len(), 1);

    today.add_device(1, strip("beta", 2));
    today.notify_device_list_updated();
    wait_until(&handle, |snapshot| {
        snapshot
            .devices
            .iter()
            .any(|device| device.descriptor.serial == "SN-beta" && device.available)
    })
    .await;
    let report = applied(&profiles, &handle, &id).await;
    sleep(Duration::from_millis(100)).await;

    assert_eq!(outcome_for(&report, "Strip beta"), ItemOutcome::Applied);
    let beta_frames = colour_writes(&today, 0, "beta");
    assert!(!beta_frames.is_empty());
    assert!(beta_frames.iter().all(|colors| colors == &[[0, 0, 255]; 2]));
}

#[tokio::test(start_paused = true)]
async fn applying_a_profile_skips_a_device_declared_external() {
    let dir = TempDir::new("external");
    let profiles = open(&dir).await;
    let (handle, controls) = demo().await;
    let gpu = DeviceRef::new("demo-graphics-card");
    handle
        .apply_effect(whole(&gpu), solid(RED))
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&handle, "Gaming")
        .await
        .expect("capture");
    handle.stop_effect(whole(&gpu)).await.expect("stop");
    handle
        .declare_external(gpu.clone(), PLUGIN.into())
        .await
        .expect("declare");
    let before = controls.outputs().len();

    let report = applied(&profiles, &handle, &id).await;
    sleep(Duration::from_millis(200)).await;

    assert_eq!(
        outcome_for(&report, "Demo Graphics Card"),
        skipped(SkipReason::DeclaredExternal {
            label: PLUGIN.into()
        })
    );
    let written: Vec<DemoOutput> = controls.outputs()[before..].to_vec();
    assert!(written.is_empty(), "{written:?}");
}

#[tokio::test(start_paused = true)]
async fn reused_runtime_ids_after_a_server_restart_keep_each_setting_on_its_device() {
    let dir = TempDir::new("reused-ids");
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 3));
    let profiles = open(&dir).await;
    let first = connected(&server, profiles.known_devices().await).await;
    let alpha = reference(&first.snapshot(), "SN-alpha");
    let beta = reference(&first.snapshot(), "SN-beta");
    first
        .apply_effect(whole(&alpha), solid(RED))
        .await
        .expect("alpha");
    first
        .apply_effect(whole(&beta), solid(BLUE))
        .await
        .expect("beta");
    let id = profiles
        .create_from_session(&first, "Pair")
        .await
        .expect("capture");
    drop(first);
    drop(profiles);
    sleep(Duration::from_millis(100)).await;

    server.restart(vec![
        (0, strip("beta", 3).into()),
        (1, strip("alpha", 3).into()),
    ]);
    let profiles = open(&dir).await;
    let handle = connected(&server, profiles.known_devices().await).await;
    let report = applied(&profiles, &handle, &id).await;
    sleep(Duration::from_millis(100)).await;

    assert!(
        report
            .items
            .iter()
            .all(|item| item.outcome == ItemOutcome::Applied),
        "{report:?}"
    );
    let alpha_frames = colour_writes(&server, 1, "alpha");
    let beta_frames = colour_writes(&server, 1, "beta");
    assert!(!alpha_frames.is_empty() && !beta_frames.is_empty());
    assert!(
        alpha_frames
            .iter()
            .all(|colors| colors == &[[255, 0, 0]; 3])
    );
    assert!(beta_frames.iter().all(|colors| colors == &[[0, 0, 255]; 3]));
}

#[tokio::test(start_paused = true)]
async fn indistinguishable_devices_stay_unresolved_and_get_nothing() {
    let dir = TempDir::new("ambiguous");
    let server = FakeServer::new();
    let twin = |label: &str| FixtureDevice::new(label, "Twin strip").zone("Strip", 2);
    server.add_device(0, twin("left"));
    let profiles = open(&dir).await;
    let first = connected(&server, Vec::new()).await;
    let device = first.snapshot().devices[0]
        .identity
        .reference()
        .cloned()
        .expect("one twin resolves");
    first
        .apply_effect(whole(&device), solid(RED))
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&first, "Twin")
        .await
        .expect("capture");
    drop(first);
    sleep(Duration::from_millis(100)).await;

    server.restart(vec![(0, twin("left").into()), (1, twin("right").into())]);
    let handle = connected(&server, Vec::new()).await;
    let details = profiles.details(&id, Some(&handle)).await.expect("details");
    let report = applied(&profiles, &handle, &id).await;
    sleep(Duration::from_millis(100)).await;

    assert_eq!(details.devices[0].status, DeviceStatus::Ambiguous);
    assert_eq!(
        outcome_for(&report, "Twin strip"),
        skipped(SkipReason::Ambiguous)
    );
    assert!(
        output_packets(&server)
            .iter()
            .all(|packet| packet.session == 0)
    );
}

#[tokio::test(start_paused = true)]
async fn a_changed_mode_list_or_lost_direct_mode_is_reported_not_misapplied() {
    let dir = TempDir::new("changed-modes");
    let modes = vec![
        FixtureMode::direct(),
        FixtureMode::static_color(),
        FixtureMode::breathing(),
    ];
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3).modes(modes));
    server.add_device(1, strip("beta", 3));
    let profiles = open(&dir).await;
    let first = connected(&server, Vec::new()).await;
    let alpha = reference(&first.snapshot(), "SN-alpha");
    let beta = reference(&first.snapshot(), "SN-beta");
    first.select_mode(alpha, 2).await.expect("breathing");
    first
        .apply_effect(whole(&beta), solid(BLUE))
        .await
        .expect("beta");
    let id = profiles
        .create_from_session(&first, "Modes")
        .await
        .expect("capture");
    drop(first);
    sleep(Duration::from_millis(100)).await;

    let reordered = vec![
        FixtureMode::direct(),
        FixtureMode::breathing(),
        FixtureMode::static_color(),
    ];
    server.restart(vec![
        (0, strip("alpha", 3).modes(reordered).into()),
        (1, strip("beta", 3).without_direct_mode().into()),
    ]);
    let handle = connected(&server, Vec::new()).await;
    let report = applied(&profiles, &handle, &id).await;
    sleep(Duration::from_millis(100)).await;

    assert_eq!(
        outcome_for(&report, "Strip alpha"),
        skipped(SkipReason::ModeListChanged {
            found: Some("Static".into())
        })
    );
    assert_eq!(
        outcome_for(&report, "Strip beta"),
        skipped(SkipReason::NoDirectMode)
    );
    assert!(
        output_packets(&server)
            .iter()
            .all(|packet| packet.session == 0)
    );
}

#[tokio::test(start_paused = true)]
async fn an_effect_this_version_does_not_know_is_reported_and_kept() {
    let dir = TempDir::new("unknown-effect");
    let profiles = open(&dir).await;
    let (handle, controls) = demo().await;
    let gpu = DeviceRef::new("demo-graphics-card");
    handle
        .apply_effect(whole(&gpu), solid(RED))
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&handle, "Future")
        .await
        .expect("capture");
    handle.stop_effect(whole(&gpu)).await.expect("stop");
    let mut edited = profiles.profile(&id).await.expect("stored");
    edited.assignments[0].effect = effect("fire", &[("heat", ControlValue::Number(3))]);
    profiles.replace(&id, edited.clone()).await.expect("edit");
    let before = controls.outputs().len();

    let report = applied(&profiles, &handle, &id).await;
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("save");
    sleep(Duration::from_millis(100)).await;

    assert_eq!(
        outcome_for(&report, "Demo Graphics Card"),
        skipped(SkipReason::NotAvailable {
            problem: EffectProblem::EffectNotAvailable {
                effect: "fire".into()
            }
        })
    );
    assert_eq!(
        report.items[0].item,
        ProfileItem::Effect {
            zone: None,
            effect: "fire".into()
        }
    );
    assert_eq!(profiles.profile(&id).await.expect("kept"), edited);
    assert!(controls.outputs()[before..].is_empty());
}

#[tokio::test(start_paused = true)]
async fn applying_while_not_connected_is_refused_unless_marked_for_connect() {
    let dir = TempDir::new("not-connected");
    let profiles = open(&dir).await;
    let (handle, _) = demo().await;
    let id = profiles
        .create_from_session(&handle, "Any")
        .await
        .expect("capture");
    handle.disconnect().await.expect("disconnect");
    wait_until(&handle, |snapshot| !is_connected(snapshot)).await;

    let refused = profiles.apply(&handle, &id).await;
    profiles
        .set_apply_on_connect(Some(id.clone()))
        .await
        .expect("mark");
    let waiting = profiles.apply(&handle, &id).await;

    assert_eq!(refused, Err(ApplyError::NotConnected));
    assert_eq!(waiting, Ok(ApplyOutcome::WaitingForConnection));
}
