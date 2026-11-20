use prismforge_core::automation::Trigger;
use std::time::Duration;

use prismforge_core::profile::{DeviceStatus, EffectProblem, ItemOutcome, SkipReason};
use prismforge_core::store::{
    ApplyOutcome, FileProblem, IMPORT_LIMIT, ImportError, ProfileStore, Profiles, StdFileOps,
    UnsupportedEffect,
};
use prismforge_core::{ApplyMode, DeviceRef, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};
use prismforge_testkit::{FakeServer, FixtureDevice};
use tokio::time::sleep;

use crate::session::{connected, output_packets, reference, whole};
use crate::support::{TempDir, connects_rule, golden};

const SECRET_SERIAL: &str = "SERIAL-1234-PRIVATE";
const SECRET_LOCATION: &str = "HID: /dev/hidraw7 usb-0000:00:14.0-3";
const FRIENDLY: &str = "Kitchen strip of Alex";

fn private_strip(label: &str, serial: &str) -> FixtureDevice {
    FixtureDevice::new(label, "Strip twin")
        .serial(serial)
        .location(&format!("{SECRET_LOCATION}-{label}"))
        .zone("Strip", 3)
}

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

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

#[test]
fn export_pins_the_portable_format() {
    let dir = TempDir::new("export-golden");
    dir.write("profiles/p-golden.json", &golden("profile-v1.json"));
    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let id = prismforge_core::profile::ProfileId::parse("p-golden").expect("id");
    let destination = dir.file("Evening.prismforge.json");

    let report = store.export_to(&id, &destination).expect("export");

    assert_eq!(report.omitted_fields, 7);
    assert_eq!(
        String::from_utf8(dir.read("Evening.prismforge.json")).expect("utf-8"),
        String::from_utf8(golden("export-v2.json")).expect("utf-8")
    );
}

#[tokio::test(start_paused = true)]
async fn export_contains_no_private_machine_data() {
    let dir = TempDir::new("export-private");
    let server = FakeServer::new();
    server.add_device(0, private_strip("left", SECRET_SERIAL));
    let profiles = open(&dir).await;
    let handle = connected(&server, Vec::new()).await;
    let device = reference(&handle.snapshot(), SECRET_SERIAL);
    profiles
        .record_known_devices(handle.known_devices().borrow().clone())
        .await
        .expect("registry");
    profiles
        .set_device_name(&device, Some(FRIENDLY.into()))
        .await
        .expect("name");
    handle
        .apply_effect(whole(&device), solid(Rgb::new(1, 2, 3)))
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&handle, "Shared")
        .await
        .expect("capture")
        .id;
    let local = dir.read(&format!("profiles/{id}.json"));
    let destination = dir.file("Shared.prismforge.json");

    profiles.export_to(&id, destination).await.expect("export");
    let exported = dir.read("Shared.prismforge.json");

    assert!(contains(&local, SECRET_SERIAL) && contains(&local, FRIENDLY));
    let text = String::from_utf8(exported.clone()).expect("utf-8");
    for private in [
        SECRET_SERIAL,
        "hidraw",
        "usb-0000",
        FRIENDLY,
        device.as_str(),
        id.as_str(),
        &dir.path().to_string_lossy(),
        "dev-",
    ] {
        assert!(!contains(&exported, private), "{private} in {text}");
    }
    assert!(contains(
        &exported,
        "\"format\": \"prismforge.profile-export\""
    ));
    assert!(contains(&exported, "\"label\": \"Strip twin\""));
    assert!(contains(&exported, "\"reference\": \"device-1\""));
}

#[tokio::test(start_paused = true)]
async fn import_matches_by_model_and_never_picks_one_of_two_identical_devices() {
    let source = TempDir::new("import-source");
    let server = FakeServer::new();
    server.add_device(0, private_strip("left", SECRET_SERIAL));
    let profiles = open(&source).await;
    let first = connected(&server, Vec::new()).await;
    let device = reference(&first.snapshot(), SECRET_SERIAL);
    first
        .apply_effect(whole(&device), solid(Rgb::new(9, 9, 9)))
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&first, "Twin")
        .await
        .expect("capture")
        .id;
    let file = source.file("twin.json");
    profiles.export_to(&id, file.clone()).await.expect("export");
    drop(first);
    sleep(Duration::from_millis(100)).await;

    let single = FakeServer::new();
    single.add_device(0, private_strip("other", "SERIAL-OTHER"));
    let twins = FakeServer::new();
    twins.add_device(0, private_strip("left", "SERIAL-A"));
    twins.add_device(1, private_strip("right", "SERIAL-B"));
    let target = TempDir::new("import-target");
    let imported = open(&target).await;
    let alone = connected(&single, Vec::new()).await;
    let report = imported
        .import_from(file.clone(), Some(&alone))
        .await
        .expect("import");
    let pair = connected(&twins, Vec::new()).await;
    let details = imported
        .details(&report.details.id, Some(&pair))
        .await
        .expect("details");
    let outcome = imported.apply(&pair, &report.details.id).await;
    sleep(Duration::from_millis(100)).await;

    assert!(matches!(
        report.details.devices[0].status,
        DeviceStatus::Resolved { .. }
    ));
    assert_eq!(details.devices[0].status, DeviceStatus::Ambiguous);
    let Ok(ApplyOutcome::Applied(applied)) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(
        applied.items[0].outcome,
        ItemOutcome::Skipped {
            reason: SkipReason::Ambiguous
        }
    );
    assert!(output_packets(&twins).is_empty());
    assert!(output_packets(&single).is_empty());
}

#[test]
fn imports_a_version_1_export_with_every_effect_on_each_device() {
    let dir = TempDir::new("import-v1");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    let old = store
        .import(&golden("export-v1.json"), None)
        .expect("version 1 import");
    let new = store
        .import(&golden("export-v2.json"), None)
        .expect("version 2 import");

    let old = store.profile(&old.details.id).expect("stored").clone();
    let new = store.profile(&new.details.id).expect("stored").clone();
    assert_eq!(old.assignments.len(), 5);
    assert!(
        old.assignments
            .iter()
            .all(|assignment| assignment.mode == ApplyMode::EachDevice)
    );
    assert_eq!(old.assignments, new.assignments);
    assert_eq!(old.devices, new.devices);
}

#[test]
fn import_keeps_and_reports_what_this_version_cannot_run_and_never_overwrites() {
    let source = TempDir::new("import-unknown");
    source.write("profiles/p-golden.json", &golden("profile-v1.json"));
    let store = ProfileStore::open(source.path(), StdFileOps).expect("open");
    let id = prismforge_core::profile::ProfileId::parse("p-golden").expect("id");
    let exported = store.export(&id).expect("export").bytes;
    let target = TempDir::new("import-target");
    let mut store = ProfileStore::open(target.path(), StdFileOps).expect("open");

    let first = store.import(&exported, None).expect("first import");
    let second = store.import(&exported, None).expect("second import");

    assert_eq!(first.renamed_from, None);
    assert_eq!(first.details.name, "Evening");
    assert_eq!(second.renamed_from.as_deref(), Some("Evening"));
    assert_eq!(second.details.name, "Evening (2)");
    assert_ne!(first.details.id, second.details.id);
    assert!(
        first
            .details
            .devices
            .iter()
            .all(|device| device.status == DeviceStatus::Missing)
    );
    assert_eq!(
        first.details.unsupported,
        [
            UnsupportedEffect {
                device: DeviceRef::new("device-3"),
                zone: None,
                problem: EffectProblem::ControlsNotAvailable {
                    controls: vec!["future_control".into()]
                },
            },
            UnsupportedEffect {
                device: DeviceRef::new("device-4"),
                zone: Some(0),
                problem: EffectProblem::EffectNotAvailable {
                    effect: "fire".into()
                },
            },
        ]
    );
    let stored = store.profile(&first.details.id).expect("stored");
    assert_eq!(stored.assignments[4].effect.effect, "fire");
    assert!(
        stored.assignments[3]
            .effect
            .values
            .contains_key("future_control")
    );
    assert_eq!(store.summary(None).profiles.len(), 2);
}

#[test]
fn import_refuses_damaged_foreign_oversized_and_future_files_before_storing_anything() {
    let dir = TempDir::new("import-refused");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let local = golden("profile-v1.json");
    let future = String::from_utf8(golden("export-v1.json"))
        .expect("utf-8")
        .replacen("\"major\": 1", "\"major\": 3", 1);
    let invalid = String::from_utf8(golden("export-v1.json"))
        .expect("utf-8")
        .replacen("\"frame_rate\": 45", "\"frame_rate\": 0", 1);
    let oversized = vec![b' '; IMPORT_LIMIT as usize + 1];
    let cases: Vec<(&[u8], FileProblem)> = vec![
        (
            b"{ not json",
            FileProblem::NotJson {
                detail: "key must be a string at line 1 column 3".into(),
            },
        ),
        (
            &local,
            FileProblem::WrongFormat {
                expected: "prismforge.profile-export".into(),
                found: Some("prismforge.profile".into()),
            },
        ),
        (
            b"[1, 2]",
            FileProblem::WrongFormat {
                expected: "prismforge.profile-export".into(),
                found: None,
            },
        ),
        (
            future.as_bytes(),
            FileProblem::NewerVersion {
                found: 3,
                supported: 2,
            },
        ),
        (
            invalid.as_bytes(),
            FileProblem::Invalid {
                detail: "frame rate 0 is outside 1 to 60".into(),
            },
        ),
        (
            &oversized,
            FileProblem::TooLarge {
                limit: IMPORT_LIMIT,
            },
        ),
    ];

    for (bytes, problem) in cases {
        assert_eq!(
            store.import(bytes, None),
            Err(ImportError::Unreadable { problem })
        );
    }
    assert!(store.summary(None).profiles.is_empty());
    assert_eq!(dir.files(), Vec::<String>::new());
}

#[tokio::test(start_paused = true)]
async fn loading_and_importing_write_nothing_to_hardware() {
    let dir = TempDir::new("no-writes");
    let server = FakeServer::new();
    server.add_device(0, private_strip("left", SECRET_SERIAL));
    let profiles = open(&dir).await;
    let first = connected(&server, Vec::new()).await;
    let device = reference(&first.snapshot(), SECRET_SERIAL);
    first
        .apply_effect(whole(&device), solid(Rgb::new(5, 5, 5)))
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&first, "Marked")
        .await
        .expect("capture")
        .id;
    profiles
        .set_rule(Trigger::Connects, connects_rule(&id))
        .await
        .expect("mark");
    let file = dir.file("marked.json");
    profiles.export_to(&id, file.clone()).await.expect("export");
    drop(first);
    drop(profiles);
    sleep(Duration::from_millis(100)).await;
    let written_before = output_packets(&server).len();

    let handle = connected(&server, Vec::new()).await;
    let profiles = open(&dir).await;
    let report = profiles
        .import_from(file, Some(&handle))
        .await
        .expect("import");
    sleep(Duration::from_secs(2)).await;

    assert!(matches!(
        report.details.devices[0].status,
        DeviceStatus::Resolved { .. }
    ));
    assert_eq!(profiles.summary(None).await.apply_on_connect, Some(id));
    assert_eq!(output_packets(&server).len(), written_before);
}
