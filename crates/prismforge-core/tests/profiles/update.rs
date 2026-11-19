use std::time::Duration;

use prismforge_core::profile::{Change, Profile, ProfileId, ProfileItem, SettingChange};
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{
    ApplyOutcome, ImportError, ProfileStore, Profiles, STORE_FILE_LIMIT, StdFileOps, StoreError,
};
use prismforge_core::{DeviceRef, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};
use prismforge_testkit::{FakeServer, FixtureDevice, FixtureMode};
use serde_json::json;
use tokio::time::sleep;

use crate::session::{connected, demo, openrgb, reference, start, wait_until, whole};
use crate::support::{TempDir, effect, extra, golden};

const PLUGIN: &str = "OpenRGB Effects plugin";
const GPU: &str = "demo-graphics-card";
const MEMORY: &str = "demo-memory-1";
const LIGHT_BAR: &str = "demo-light-bar";

fn solid(color: Rgb) -> EffectInstance {
    EffectInstance {
        effect: "solid".into(),
        values: [("color".to_owned(), ControlValue::Color(color))].into(),
    }
}

fn rainbow(values: &[(&str, ControlValue)]) -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: values
            .iter()
            .map(|(control, value)| ((*control).to_owned(), value.clone()))
            .collect(),
    }
}

fn gpu() -> DeviceRef {
    DeviceRef::new(GPU)
}

async fn open(dir: &TempDir) -> Profiles {
    Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open store")
}

async fn apply(profiles: &Profiles, handle: &SessionHandle, id: &ProfileId) {
    match profiles.apply(handle, id).await.expect("apply") {
        ApplyOutcome::Applied(_) => {}
        other => panic!("{other:?}"),
    }
}

fn effect_item(zone: Option<usize>, effect: &str) -> ProfileItem {
    ProfileItem::Effect {
        zone,
        effect: effect.into(),
    }
}

fn modes() -> Vec<FixtureMode> {
    vec![
        FixtureMode::direct(),
        FixtureMode::static_color(),
        FixtureMode::breathing(),
    ]
}

fn twin(label: &str) -> FixtureDevice {
    FixtureDevice::new(label, "Twin strip").zone("Strip", 2)
}

/// A demo session with a solid effect on the graphics card, saved as the active profile `Base`.
async fn active_profile(
    dir: &TempDir,
) -> (
    Profiles,
    SessionHandle,
    prismforge_testkit::DemoControls,
    ProfileId,
) {
    let profiles = open(dir).await;
    let (handle, controls) = demo().await;
    handle
        .apply_effect(whole(&gpu()), solid(Rgb::new(9, 9, 9)))
        .await
        .expect("gpu");
    let id = profiles
        .create_from_session(&handle, "Base")
        .await
        .expect("create")
        .id;
    (profiles, handle, controls, id)
}

#[tokio::test(start_paused = true)]
async fn update_keeps_a_setting_that_apply_skipped() {
    let dir = TempDir::new("update-skipped");
    let server = FakeServer::new();
    server.add_device(0, crate::session::strip("alpha", 3));
    server.add_device(1, crate::session::strip("beta", 2));
    let profiles = open(&dir).await;
    let first = connected(&server, Vec::new()).await;
    let alpha = reference(&first.snapshot(), "SN-alpha");
    let beta = reference(&first.snapshot(), "SN-beta");
    first
        .apply_effect(whole(&alpha), solid(Rgb::new(255, 0, 0)))
        .await
        .expect("alpha");
    first
        .apply_effect(whole(&beta), solid(Rgb::new(0, 0, 255)))
        .await
        .expect("beta");
    let id = profiles
        .create_from_session(&first, "Desk")
        .await
        .expect("capture")
        .id;
    let stored = profiles.profile(&id).await.expect("stored");
    drop(first);
    sleep(Duration::from_millis(100)).await;

    server.restart(vec![
        (0, crate::session::strip("alpha", 3).into()),
        (
            1,
            crate::session::strip("beta", 2)
                .without_direct_mode()
                .into(),
        ),
    ]);
    let handle = connected(&server, Vec::new()).await;
    apply(&profiles, &handle, &id).await;
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    assert_eq!(profiles.profile(&id).await.expect("saved"), stored);
    assert_eq!(saved.changes, []);
}

#[tokio::test(start_paused = true)]
async fn update_keeps_the_setting_for_every_skip_reason() {
    let dir = TempDir::new("update-every-skip");
    let server = FakeServer::new();
    let strip = crate::session::strip;
    let all = |modes_of_gamma: Vec<FixtureMode>, beta_direct: bool, twins: bool, iota: bool| {
        let beta = if beta_direct {
            strip("beta", 2)
        } else {
            strip("beta", 2).without_direct_mode()
        };
        let mut devices: Vec<(u32, prismforge_testkit::ControllerBlock)> = vec![
            (0, strip("alpha", 3).into()),
            (1, beta.into()),
            (2, strip("gamma", 2).modes(modes_of_gamma).into()),
            (3, strip("eps", 2).into()),
            (4, strip("zeta", 2).into()),
            (5, strip("eta", 2).into()),
            (6, twin("left").into()),
        ];
        if twins {
            devices.push((7, twin("right").into()));
        }
        if iota {
            devices.push((8, strip("iota", 2).into()));
        }
        devices
    };
    for (id, block) in all(modes(), true, false, true) {
        server.add_controller(id, &format!("c{id}"), block);
    }
    server.add_device(9, strip("delta", 2));
    let profiles = open(&dir).await;
    let first = connected(&server, Vec::new()).await;
    let snapshot = first.snapshot();
    for serial in [
        "SN-alpha", "SN-beta", "SN-delta", "SN-eps", "SN-zeta", "SN-eta", "SN-iota",
    ] {
        first
            .apply_effect(
                whole(&reference(&snapshot, serial)),
                solid(Rgb::new(1, 2, 3)),
            )
            .await
            .expect(serial);
    }
    let twin_ref = snapshot
        .devices
        .iter()
        .find(|device| device.descriptor.name == "Twin strip")
        .and_then(|device| device.identity.reference().cloned())
        .expect("twin");
    first
        .apply_effect(whole(&twin_ref), solid(Rgb::new(1, 2, 3)))
        .await
        .expect("twin");
    first
        .select_mode(reference(&snapshot, "SN-gamma"), 2)
        .await
        .expect("mode");
    let id = profiles
        .create_from_session(&first, "Everything")
        .await
        .expect("capture")
        .id;
    let mut stored = profiles.profile(&id).await.expect("stored");
    let zeta = reference(&snapshot, "SN-zeta");
    let zeta_effect = stored
        .assignments
        .iter_mut()
        .find(|assignment| assignment.device == zeta)
        .expect("zeta");
    zeta_effect.effect = effect("fire", &[("heat", ControlValue::Number(3))]);
    profiles.replace(&id, stored.clone()).await.expect("edit");
    drop(first);
    sleep(Duration::from_millis(100)).await;

    let reordered = vec![
        FixtureMode::direct(),
        FixtureMode::breathing(),
        FixtureMode::static_color(),
    ];
    server.restart(all(reordered.clone(), false, true, true));
    let handle = connected(&server, Vec::new()).await;
    let iota = reference(&handle.snapshot(), "SN-iota");
    server.restart(all(reordered, false, true, false));
    wait_until(&handle, |snapshot| {
        snapshot.connection
            == prismforge_core::ConnectionState::Degraded {
                reasons: vec![prismforge_core::StateReason::AmbiguousDevices { devices: 2 }],
            }
            && snapshot
                .devices
                .iter()
                .any(|device| device.identity.reference() == Some(&iota) && !device.available)
    })
    .await;
    let snapshot = handle.snapshot();
    handle
        .declare_external(reference(&snapshot, "SN-eps"), PLUGIN.into())
        .await
        .expect("declare");
    let identify = {
        let (handle, eta) = (handle.clone(), reference(&snapshot, "SN-eta"));
        tokio::spawn(async move { handle.identify(eta, None, Duration::from_secs(20)).await })
    };
    sleep(Duration::from_millis(10)).await;
    apply(&profiles, &handle, &id).await;
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");
    identify.abort();

    assert_eq!(profiles.profile(&id).await.expect("saved"), stored);
    assert_eq!(saved.changes, []);
    assert_eq!(saved.frame_rate, None);
}

#[tokio::test(start_paused = true)]
async fn a_setting_that_never_ran_survives_an_update_after_an_update() {
    let dir = TempDir::new("update-twice");
    let (profiles, handle, _, id) = active_profile(&dir).await;
    let mut stored = profiles.profile(&id).await.expect("stored");
    let light_bar = handle
        .snapshot()
        .devices
        .iter()
        .find(|device| device.identity.reference().map(DeviceRef::as_str) == Some(LIGHT_BAR))
        .map(|device| prismforge_core::profile::ProfileDevice {
            reference: DeviceRef::new(LIGHT_BAR),
            label: "Demo Light Bar".into(),
            identity: prismforge_core::profile::StoredIdentity::from_fingerprint(
                &device.descriptor.fingerprint(),
            ),
            extra: Default::default(),
        })
        .expect("light bar");
    stored.devices.push(light_bar);
    stored
        .assignments
        .push(prismforge_core::profile::ProfileAssignment {
            device: DeviceRef::new(LIGHT_BAR),
            zone: None,
            effect: prismforge_core::profile::StoredEffect::from_instance(&solid(Rgb::new(
                1, 1, 1,
            ))),
            extra: Default::default(),
        });
    profiles.replace(&id, stored.clone()).await.expect("edit");
    apply(&profiles, &handle, &id).await;

    let first = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("first");
    handle.stop_effect(whole(&gpu())).await.expect("stop");
    let second = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("second");
    let third = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("third");

    assert_eq!(first.changes, []);
    assert_eq!(
        second.changes,
        [SettingChange {
            device: gpu(),
            label: "Demo Graphics Card".into(),
            change: Change::Removed {
                was: effect_item(None, "solid")
            },
        }]
    );
    assert_eq!(third.changes, []);
    let saved = profiles.profile(&id).await.expect("saved");
    assert_eq!(saved.assignments.len(), 1);
    assert_eq!(saved.assignments[0].device.as_str(), LIGHT_BAR);
    assert_eq!(saved.devices, stored.devices);
}

#[tokio::test(start_paused = true)]
async fn a_disconnect_or_reconnect_is_not_a_change() {
    let dir = TempDir::new("update-disconnect");
    let (profiles, handle, controls, id) = active_profile(&dir).await;
    let stored = profiles.profile(&id).await.expect("stored");
    apply(&profiles, &handle, &id).await;

    handle.disconnect().await.expect("disconnect");
    let while_down = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("down");
    handle.connect().await.expect("connect");
    controls.lose_connection(prismforge_core::LossCause::ServerClosed);
    wait_until(&handle, |snapshot| {
        snapshot.diagnostics.counters.connect_attempts >= 3
            && snapshot.connection == prismforge_core::ConnectionState::Connected
    })
    .await;
    let after = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("after");

    assert_eq!(while_down.changes, []);
    assert_eq!(after.changes, []);
    assert_eq!(profiles.profile(&id).await.expect("saved"), stored);
}

#[tokio::test(start_paused = true)]
async fn what_prismforge_does_by_itself_is_not_a_change() {
    let dir = TempDir::new("update-internal");
    let (profiles, handle, controls, id) = active_profile(&dir).await;
    let stored = profiles.profile(&id).await.expect("stored");
    apply(&profiles, &handle, &id).await;

    controls.refuse_writes(&gpu());
    sleep(Duration::from_secs(3)).await;
    handle
        .identify(DeviceRef::new(MEMORY), None, Duration::from_secs(1))
        .await
        .expect("identify");
    controls.change_inventory();
    handle.refresh().await.expect("refresh");
    sleep(Duration::from_secs(1)).await;
    let summary = profiles.summary(Some(&handle)).await;
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    assert_eq!(summary.unsaved_changes, Some(0));
    assert_eq!(saved.changes, []);
    assert_eq!(profiles.profile(&id).await.expect("saved"), stored);
}

#[tokio::test(start_paused = true)]
async fn an_unavailable_device_is_not_a_change() {
    let dir = TempDir::new("update-unavailable");
    let server = FakeServer::new();
    server.add_device(0, crate::session::strip("alpha", 3));
    server.add_device(1, crate::session::strip("beta", 2));
    let profiles = open(&dir).await;
    let handle = connected(&server, Vec::new()).await;
    let beta = reference(&handle.snapshot(), "SN-beta");
    handle
        .apply_effect(whole(&beta), solid(Rgb::new(0, 0, 255)))
        .await
        .expect("beta");
    let id = profiles
        .create_from_session(&handle, "Pair")
        .await
        .expect("capture")
        .id;
    let stored = profiles.profile(&id).await.expect("stored");

    server.restart(vec![(0, crate::session::strip("alpha", 3).into())]);
    wait_until(&handle, |snapshot| {
        snapshot.connection == prismforge_core::ConnectionState::Connected
            && snapshot
                .devices
                .iter()
                .any(|device| device.identity.reference() == Some(&beta) && !device.available)
    })
    .await;
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    assert_eq!(saved.changes, []);
    assert_eq!(profiles.profile(&id).await.expect("saved"), stored);
}

#[tokio::test(start_paused = true)]
async fn a_control_change_saves_the_complete_new_values() {
    let dir = TempDir::new("update-control");
    let (profiles, handle, _, id) = active_profile(&dir).await;
    let mut stored = profiles.profile(&id).await.expect("stored");
    stored.assignments[0].effect = effect("rainbow-wave", &[]);
    profiles.replace(&id, stored).await.expect("edit");
    apply(&profiles, &handle, &id).await;

    let bright = rainbow(&[("brightness", ControlValue::Number(37))]);
    handle
        .apply_effect(whole(&gpu()), bright)
        .await
        .expect("brightness");
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    let profile = profiles.profile(&id).await.expect("saved");
    let complete =
        prismforge_effects::prepare(&rainbow(&[("brightness", ControlValue::Number(37))]))
            .expect("valid")
            .instance()
            .clone();
    assert_eq!(
        profile.assignments[0].effect,
        prismforge_core::profile::StoredEffect::from_instance(&complete)
    );
    assert_eq!(
        saved.changes,
        [SettingChange {
            device: gpu(),
            label: "Demo Graphics Card".into(),
            change: Change::Replaced {
                was: effect_item(None, "rainbow-wave"),
                now: effect_item(None, "rainbow-wave"),
            },
        }]
    );
}

#[tokio::test(start_paused = true)]
async fn a_stop_then_a_new_apply_saves_the_new_effect() {
    let dir = TempDir::new("update-reapply");
    let (profiles, handle, _, id) = active_profile(&dir).await;
    apply(&profiles, &handle, &id).await;

    handle.stop_effect(whole(&gpu())).await.expect("stop");
    handle
        .apply_effect(whole(&gpu()), solid(Rgb::new(200, 100, 50)))
        .await
        .expect("again");
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    let profile = profiles.profile(&id).await.expect("saved");
    assert_eq!(
        profile.assignments[0].effect.values["color"],
        json!({"Color": {"r": 200, "g": 100, "b": 50}})
    );
    assert_eq!(
        saved.changes[0].change,
        Change::Replaced {
            was: effect_item(None, "solid"),
            now: effect_item(None, "solid"),
        }
    );
}

#[tokio::test(start_paused = true)]
async fn user_changes_are_saved_and_reported_and_previewed() {
    let dir = TempDir::new("update-user");
    let (profiles, handle, _, id) = active_profile(&dir).await;
    let memory = DeviceRef::new(MEMORY);
    handle
        .apply_effect(whole(&memory), solid(Rgb::new(8, 8, 8)))
        .await
        .expect("memory");
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("add memory");
    handle.stop_effect(whole(&gpu())).await.expect("stop");
    handle
        .apply_effect(whole(&memory), rainbow(&[]))
        .await
        .expect("change");
    handle
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("mode");
    handle.set_frame_rate(45).await.expect("rate");

    let summary = profiles.summary(Some(&handle)).await;
    let preview = profiles
        .preview_update(&handle, &id)
        .await
        .expect("preview");
    let unchanged = profiles.profile(&id).await.expect("not yet saved");
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    assert_eq!(summary.active, Some(id.clone()));
    assert_eq!(summary.unsaved_changes, Some(4));
    assert_eq!(preview, saved);
    assert_eq!(unchanged.assignments.len(), 2);
    let changes: Vec<(&str, &Change)> = saved
        .changes
        .iter()
        .map(|change| (change.device.as_str(), &change.change))
        .collect();
    assert_eq!(
        changes,
        [
            (
                GPU,
                &Change::Removed {
                    was: effect_item(None, "solid")
                }
            ),
            (
                MEMORY,
                &Change::Replaced {
                    was: effect_item(None, "solid"),
                    now: effect_item(None, "rainbow-wave"),
                }
            ),
            (
                LIGHT_BAR,
                &Change::Added {
                    now: ProfileItem::HardwareMode {
                        index: 1,
                        name: "Breathing".into()
                    }
                }
            ),
        ]
    );
    assert_eq!(
        saved.frame_rate,
        Some(prismforge_core::profile::FrameRateChange { was: 30, now: 45 })
    );
    let profile = profiles.profile(&id).await.expect("saved");
    assert_eq!(profile.frame_rate, 45);
    assert!(
        profile
            .assignments
            .iter()
            .all(|assignment| assignment.device == memory)
    );
    assert_eq!(
        profiles.summary(Some(&handle)).await.unsaved_changes,
        Some(0)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_stop_right_after_an_apply_is_always_saved() {
    let dir = TempDir::new("update-race");
    let profiles = open(&dir).await;
    let (handle, _) = demo().await;
    for round in 0..40 {
        handle
            .apply_effect(whole(&gpu()), solid(Rgb::new(round, 0, 0)))
            .await
            .expect("gpu");
        let id = profiles
            .create_from_session(&handle, &format!("Round {round}"))
            .await
            .expect("create")
            .id;
        apply(&profiles, &handle, &id).await;
        handle.stop_effect(whole(&gpu())).await.expect("stop");
        let saved = profiles
            .update_from_session(&handle, &id)
            .await
            .expect("update");
        assert_eq!(
            saved
                .changes
                .iter()
                .map(|change| &change.change)
                .collect::<Vec<_>>(),
            [&Change::Removed {
                was: effect_item(None, "solid")
            }],
            "round {round}"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn save_as_from_the_active_profile_keeps_skipped_and_unresolved_settings() {
    let dir = TempDir::new("save-as");
    let (profiles, handle, _, id) = active_profile(&dir).await;
    let mut stored = profiles.profile(&id).await.expect("stored");
    let mut absent = stored.devices[0].clone();
    absent.reference = DeviceRef::new("dev-absent");
    absent.label = "Absent strip".into();
    absent.identity.model = "Absent strip".into();
    absent.identity.serial = Some("SN-absent".into());
    stored.devices.push(absent);
    stored
        .assignments
        .push(prismforge_core::profile::ProfileAssignment {
            device: DeviceRef::new("dev-absent"),
            zone: None,
            effect: effect("fire", &[]),
            extra: Default::default(),
        });
    profiles.replace(&id, stored.clone()).await.expect("edit");
    apply(&profiles, &handle, &id).await;
    handle
        .apply_effect(whole(&gpu()), solid(Rgb::new(1, 2, 3)))
        .await
        .expect("change");

    let copy = profiles
        .create_from_session(&handle, "Copy")
        .await
        .expect("save as");

    let saved = profiles.profile(&copy.id).await.expect("copy");
    assert_eq!(saved.name, "Copy");
    assert_eq!(saved.devices, stored.devices);
    assert_eq!(saved.assignments[1], stored.assignments[1]);
    assert_eq!(
        saved.assignments[0].effect.values["color"],
        json!({"Color": {"r": 1, "g": 2, "b": 3}})
    );
    assert_eq!(profiles.profile(&id).await.expect("original"), stored);
    assert_eq!(profiles.summary(Some(&handle)).await.active, Some(copy.id));
}

#[tokio::test(start_paused = true)]
async fn update_of_a_profile_not_applied_in_this_session_is_refused() {
    let dir = TempDir::new("update-inactive");
    let profiles = open(&dir).await;
    let (first, _) = demo().await;
    first
        .apply_effect(whole(&gpu()), solid(Rgb::new(9, 9, 9)))
        .await
        .expect("gpu");
    let one = profiles
        .create_from_session(&first, "One")
        .await
        .expect("one")
        .id;
    let two = profiles
        .create_from_session(&first, "Two")
        .await
        .expect("two")
        .id;
    let stored = profiles.profile(&one).await.expect("stored");

    let other_profile = profiles.update_from_session(&first, &one).await;
    let (second, _) = demo().await;
    let other_session = profiles.update_from_session(&second, &two).await;

    assert_eq!(
        other_profile,
        Err(StoreError::NotActive {
            id: one.clone(),
            active: Some(two.clone())
        })
    );
    assert_eq!(
        other_session,
        Err(StoreError::NotActive {
            id: two.clone(),
            active: None
        })
    );
    assert_eq!(profiles.summary(Some(&second)).await.active, None);
    assert_eq!(profiles.profile(&one).await.expect("one"), stored);
}

#[tokio::test(start_paused = true)]
async fn unknown_fields_at_every_level_survive_an_update_for_present_and_absent_devices() {
    let dir = TempDir::new("update-extras");
    let strip = crate::session::strip;
    let two_zones = |label: &str| {
        FixtureDevice::new(label, &format!("Strip {label}"))
            .serial(&format!("SN-{label}"))
            .location(&format!("usb:{label}"))
            .zone("Front", 2)
            .zone("Back", 2)
    };
    let server = FakeServer::new();
    server.add_device(0, two_zones("alpha"));
    server.add_device(1, strip("beta", 2).modes(modes()));
    server.add_device(2, strip("gamma", 2));
    server.add_device(3, two_zones("delta"));
    let profiles = open(&dir).await;
    let first = connected(&server, Vec::new()).await;
    let snapshot = first.snapshot();
    let [alpha, beta, gamma, delta] =
        ["SN-alpha", "SN-beta", "SN-gamma", "SN-delta"].map(|serial| reference(&snapshot, serial));
    let zone = |device: &DeviceRef, zone| OutputTarget {
        device: device.clone(),
        zone: Some(zone),
    };
    first
        .apply_effect(zone(&alpha, 0), solid(Rgb::new(1, 1, 1)))
        .await
        .expect("alpha");
    first.select_mode(beta.clone(), 2).await.expect("beta");
    first
        .declare_external(gamma.clone(), PLUGIN.into())
        .await
        .expect("gamma");
    first
        .apply_effect(zone(&delta, 1), solid(Rgb::new(2, 2, 2)))
        .await
        .expect("delta");
    let id = profiles
        .create_from_session(&first, "Extras")
        .await
        .expect("capture")
        .id;
    let mut edited: Profile = profiles.profile(&id).await.expect("stored");
    edited.extra = extra(&[("future_profile", json!(1))]);
    for device in &mut edited.devices {
        device.extra = extra(&[("future_device", json!(device.reference.as_str()))]);
        for zone in &mut device.identity.zones {
            zone.extra = extra(&[("future_zone", json!([zone.name.clone()]))]);
        }
    }
    for assignment in &mut edited.assignments {
        assignment.extra = extra(&[("future_assignment", json!(true))]);
        assignment.effect.extra = extra(&[("future_effect", json!({"seed": 4}))]);
        if assignment.device == delta {
            assignment
                .effect
                .values
                .insert("future_control".into(), json!({"Gradient": [1]}));
        }
    }
    let position = edited
        .assignments
        .iter()
        .position(|assignment| assignment.device == alpha)
        .expect("alpha effect");
    let mut unrunnable = edited.assignments[position].clone();
    unrunnable.zone = Some(1);
    unrunnable
        .effect
        .values
        .insert("future_control".into(), json!({"Gradient": [2]}));
    edited.assignments.insert(position + 1, unrunnable);
    edited.modes[0].extra = extra(&[("future_mode", json!("m"))]);
    edited.external[0].extra = extra(&[("future_external", json!("e"))]);
    profiles.replace(&id, edited.clone()).await.expect("edit");
    drop(first);
    sleep(Duration::from_millis(100)).await;

    server.restart(vec![
        (0, two_zones("alpha").into()),
        (1, strip("beta", 2).modes(modes()).into()),
        (2, strip("gamma", 2).into()),
    ]);
    let handle = connected(&server, Vec::new()).await;
    apply(&profiles, &handle, &id).await;
    let untouched = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");
    handle
        .apply_effect(zone(&alpha, 0), rainbow(&[]))
        .await
        .expect("replace alpha front");
    let replaced = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update again");
    drop(profiles);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    let saved = reopened.profile(&id).expect("saved");

    assert_eq!(untouched.changes, []);
    assert_eq!(replaced.changes.len(), 1);
    assert_eq!(saved.devices, edited.devices);
    assert_eq!(saved.extra, edited.extra);
    assert_eq!(saved.modes, edited.modes);
    assert_eq!(saved.external, edited.external);
    let mut expected = edited.assignments.clone();
    expected[position].effect.effect = "rainbow-wave".into();
    expected[position].effect.values = saved.assignments[position].effect.values.clone();
    expected[position].effect.extra = Default::default();
    assert_eq!(saved.assignments, expected);
    assert_eq!(
        saved.assignments[position].extra,
        edited.assignments[position].extra
    );
}

#[tokio::test(start_paused = true)]
async fn registry_is_stored_when_the_keeper_starts_after_the_first_refresh() {
    let dir = TempDir::new("late-keeper");
    let server = FakeServer::new();
    server.add_device(0, crate::session::strip("alpha", 3));
    let profiles = open(&dir).await;
    let handle = start(openrgb(&server, Vec::new()));
    let known = handle.known_devices();
    handle.connect().await.expect("connect");
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    let keeper = {
        let profiles = profiles.clone();
        tokio::spawn(async move { profiles.keep_registry(known).await })
    };
    let mut stored = Vec::new();
    for _ in 0..100 {
        stored = profiles.known_devices().await;
        if !stored.is_empty() {
            break;
        }
        tokio::task::yield_now().await;
    }
    keeper.abort();

    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].reference, alpha);
    assert!(dir.file("devices.json").exists());
}

#[test]
fn the_store_never_writes_a_file_its_loader_would_refuse() {
    let dir = TempDir::new("too-large");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let id = store
        .create(crate::support::profile("Small", Vec::new()))
        .expect("create");
    let before = dir.read(&format!("profiles/{id}.json"));
    let mut huge = store.profile(&id).expect("small").clone();
    huge.extra = extra(&[("future", json!(vec![0; 700_000]))]);

    let refused = store.update(&id, huge);

    assert!(
        matches!(&refused, Err(StoreError::TooLarge { limit, .. }) if *limit == STORE_FILE_LIMIT),
        "{refused:?}"
    );
    assert_eq!(dir.read(&format!("profiles/{id}.json")), before);
    assert_eq!(store.profile(&id).expect("kept").name, "Small");
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert!(reopened.notices().is_empty());
}

#[test]
fn import_refuses_a_file_that_would_grow_past_the_store_limit() {
    let dir = TempDir::new("import-grows");
    let export = String::from_utf8(golden("export-v1.json")).expect("utf-8");
    let mut value: serde_json::Value = serde_json::from_str(&export).expect("json");
    value["assignments"][0]["effect"]["values"]["future_control"] = json!(vec![0; 300_000]);
    let compact = serde_json::to_vec(&value).expect("encode");
    assert!(compact.len() < 1024 * 1024);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    let refused = store.import(&compact, None);

    assert!(
        matches!(
            &refused,
            Err(ImportError::Store(StoreError::TooLarge { .. }))
        ),
        "{refused:?}"
    );
    assert!(store.summary(None).profiles.is_empty());
    assert_eq!(dir.files(), Vec::<String>::new());
}

fn zone(device: &str, zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: Some(zone),
    }
}

#[tokio::test(start_paused = true)]
async fn a_batch_records_every_target_it_changed_and_nothing_it_skipped() {
    let dir = TempDir::new("update-batch");
    let (profiles, handle, _controls, id) = active_profile(&dir).await;
    handle
        .declare_external(DeviceRef::new(LIGHT_BAR), PLUGIN.into())
        .await
        .expect("declare");
    let before = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("save the declaration");
    assert_eq!(before.changes.len(), 1);

    let report = handle
        .apply_batch(
            vec![
                whole(&gpu()),
                whole(&DeviceRef::new(MEMORY)),
                whole(&DeviceRef::new(LIGHT_BAR)),
                zone("demo-motherboard", 1),
                whole(&DeviceRef::new("demo-motherboard")),
            ],
            rainbow(&[]),
        )
        .await
        .expect("batch");
    let saved = profiles
        .preview_update(&handle, &id)
        .await
        .expect("preview");

    assert_eq!(report.results.len(), 5);
    let mut changed: Vec<(String, Change)> = saved
        .changes
        .into_iter()
        .map(|change| (change.device.to_string(), change.change))
        .collect();
    changed.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        changed,
        vec![
            (
                GPU.to_owned(),
                Change::Replaced {
                    was: effect_item(None, "solid"),
                    now: effect_item(None, "rainbow-wave"),
                },
            ),
            (
                MEMORY.to_owned(),
                Change::Added {
                    now: effect_item(None, "rainbow-wave"),
                },
            ),
            (
                "demo-motherboard".to_owned(),
                Change::Added {
                    now: effect_item(None, "rainbow-wave"),
                },
            ),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn a_batch_stop_records_each_stopped_target_and_its_side_effects() {
    let dir = TempDir::new("update-batch-stop");
    let (profiles, handle, _controls, id) = active_profile(&dir).await;
    handle
        .apply_batch(
            vec![
                zone("demo-motherboard", 0),
                zone("demo-motherboard", 2),
                whole(&DeviceRef::new(MEMORY)),
            ],
            solid(Rgb::new(1, 2, 3)),
        )
        .await
        .expect("batch");
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("save");

    handle
        .stop_batch(vec![
            whole(&DeviceRef::new("demo-motherboard")),
            whole(&gpu()),
        ])
        .await
        .expect("stop");
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    let after = profiles.profile(&id).await.expect("stored");
    assert_eq!(saved.changes.len(), 3, "{saved:?}");
    let devices: Vec<&str> = after
        .assignments
        .iter()
        .map(|assignment| assignment.device.as_str())
        .collect();
    assert_eq!(devices, vec![MEMORY]);
}
