use std::time::Duration;

use prismforge_core::profile::{Change, KeepReason, Profile, SettingChange, SkipReason};
use prismforge_core::session::{OutputError, SessionError, SessionHandle};
use prismforge_core::store::{
    ApplyOutcome, ImportError, ProfileStore, Profiles, STORE_FILE_LIMIT, StdFileOps, StoreError,
};
use prismforge_core::{DeviceRef, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};
use prismforge_testkit::{FakeServer, FixtureDevice, FixtureMode};
use serde_json::json;
use tokio::time::sleep;

use crate::session::{connected, demo, openrgb, reference, start, wait_until, whole};
use crate::support::{TempDir, effect, extra, golden};

const PLUGIN: &str = "OpenRGB Effects plugin";
const GPU: &str = "demo-graphics-card";
const MEMORY: &str = "demo-memory-1";

fn solid(color: Rgb) -> EffectInstance {
    EffectInstance {
        effect: "solid".into(),
        values: [("color".to_owned(), ControlValue::Color(color))].into(),
    }
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

async fn open(dir: &TempDir) -> Profiles {
    Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open store")
}

async fn apply(
    profiles: &Profiles,
    handle: &SessionHandle,
    id: &prismforge_core::profile::ProfileId,
) {
    match profiles.apply(handle, id).await.expect("apply") {
        ApplyOutcome::Applied(_) => {}
        other => panic!("{other:?}"),
    }
}

fn change_of(changes: &[SettingChange], label: &str) -> Change {
    changes
        .iter()
        .find(|change| change.label == label)
        .unwrap_or_else(|| panic!("no change for {label}: {changes:?}"))
        .change
        .clone()
}

fn not_running(reason: SkipReason) -> Change {
    Change::Kept {
        reason: KeepReason::NotRunning {
            skipped: Some(reason),
        },
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
    assert_eq!(change_of(&saved.changes, "Strip alpha"), Change::Unchanged);
    assert_eq!(
        change_of(&saved.changes, "Strip beta"),
        not_running(SkipReason::NoDirectMode)
    );
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
    let changes = &saved.changes;
    assert_eq!(change_of(changes, "Strip alpha"), Change::Unchanged);
    assert_eq!(
        change_of(changes, "Strip beta"),
        not_running(SkipReason::NoDirectMode)
    );
    assert_eq!(
        change_of(changes, "Strip gamma"),
        not_running(SkipReason::ModeListChanged {
            found: Some("Static".into())
        })
    );
    assert_eq!(
        change_of(changes, "Strip delta"),
        Change::Kept {
            reason: KeepReason::Missing
        }
    );
    assert_eq!(
        change_of(changes, "Strip eps"),
        not_running(SkipReason::DeclaredExternal {
            label: PLUGIN.into()
        })
    );
    assert!(matches!(
        change_of(changes, "Strip zeta"),
        Change::Kept {
            reason: KeepReason::NotRunning {
                skipped: Some(SkipReason::NotAvailable { .. })
            }
        }
    ));
    assert_eq!(
        change_of(changes, "Strip eta"),
        not_running(SkipReason::Refused {
            error: OutputError::Session(SessionError::IdentifyInProgress {
                device: reference(&snapshot, "SN-eta")
            })
        })
    );
    assert_eq!(
        change_of(changes, "Twin strip"),
        Change::Kept {
            reason: KeepReason::Ambiguous
        }
    );
    assert_eq!(
        change_of(changes, "Strip iota"),
        Change::Kept {
            reason: KeepReason::Unavailable
        }
    );
}

#[tokio::test(start_paused = true)]
async fn update_removes_what_the_user_stopped_and_replaces_what_the_user_changed() {
    let dir = TempDir::new("update-user");
    let profiles = open(&dir).await;
    let (handle, _) = demo().await;
    let gpu = DeviceRef::new(GPU);
    let memory = DeviceRef::new(MEMORY);
    handle
        .apply_effect(whole(&gpu), solid(Rgb::new(9, 9, 9)))
        .await
        .expect("gpu");
    handle
        .apply_effect(whole(&memory), solid(Rgb::new(8, 8, 8)))
        .await
        .expect("memory");
    let created = profiles
        .create_from_session(&handle, "Mine")
        .await
        .expect("capture");
    handle.stop_effect(whole(&gpu)).await.expect("stop");
    handle
        .apply_effect(whole(&memory), rainbow())
        .await
        .expect("change");

    let saved = profiles
        .update_from_session(&handle, &created.id)
        .await
        .expect("update");

    let profile = profiles.profile(&created.id).await.expect("saved");
    assert_eq!(
        change_of(&saved.changes, "Demo Graphics Card"),
        Change::Removed
    );
    let memory_changes: Vec<&Change> = saved
        .changes
        .iter()
        .filter(|change| change.device == memory)
        .map(|change| &change.change)
        .collect();
    assert_eq!(memory_changes, [&Change::Replaced, &Change::Added]);
    assert!(profile.device(&gpu).is_none());
    assert_eq!(profile.assignments.len(), 1);
    assert_eq!(profile.assignments[0].effect.effect, "rainbow-wave");
    assert_eq!(
        profiles.summary(Some(&handle)).await.active,
        Some(created.id.clone())
    );
}

#[tokio::test(start_paused = true)]
async fn update_of_a_profile_not_applied_in_this_session_is_refused() {
    let dir = TempDir::new("update-inactive");
    let profiles = open(&dir).await;
    let (first, _) = demo().await;
    first
        .apply_effect(whole(&DeviceRef::new(GPU)), solid(Rgb::new(9, 9, 9)))
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
    let zone = |device: &DeviceRef, zone| prismforge_core::OutputTarget {
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
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");
    drop(profiles);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");

    assert_eq!(reopened.profile(&id).expect("saved"), &edited);
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

    let refused = store.import(&compact, &[]);

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
