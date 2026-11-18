use prismforge_core::profile::ProfileId;
use prismforge_core::store::{
    FileProblem, NoticeEvent, ProfileStore, StdFileOps, StoreError, StoreNotice, WriteStep,
};
use prismforge_core::{DeviceFingerprint, DeviceKind, DeviceRef, KnownDevice};

use crate::support::{FaultyOps, Step, TempDir, profile};

const DAMAGED: &[u8] = b"{\"format\": \"prismforge.profile\", \"schema_vers";

fn created(dir: &TempDir, name: &str) -> ProfileId {
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.create(profile(name, Vec::new())).expect("create")
}

fn main_file(id: &ProfileId) -> String {
    format!("profiles/{id}.json")
}

fn not_json() -> FileProblem {
    FileProblem::NotJson {
        detail: "EOF while parsing a string at line 1 column 45".into(),
    }
}

#[test]
fn keeps_valid_file_after_failed_write() {
    let dir = TempDir::new("failed-write");
    let id = created(&dir, "Original");
    let main = main_file(&id);
    let before = dir.read(&main);
    let ops = FaultyOps::default();
    let mut store = ProfileStore::open(dir.path(), ops.clone()).expect("open");
    let cases = [
        (Step::Write, format!("{id}.json.tmp"), WriteStep::Temporary),
        (Step::Sync, format!("{id}.json.tmp"), WriteStep::Sync),
        (Step::Write, format!("{id}.json.bak.tmp"), WriteStep::Backup),
        (Step::Replace, format!("{id}.json"), WriteStep::Replace),
    ];

    for (step, suffix, expected) in cases {
        ops.fail(step, &suffix);
        let result = store.rename(&id, "Changed");
        ops.heal();

        match result {
            Err(StoreError::Write(failure)) => {
                assert_eq!(failure.step, expected, "{failure:?}");
                assert_eq!(failure.file, main);
            }
            other => panic!("{step:?} on {suffix}: {other:?}"),
        }
        assert_eq!(dir.read(&main), before, "{step:?} on {suffix}");
        assert_eq!(store.profile(&id).expect("kept").name, "Original");
        let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
        assert_eq!(reopened.profile(&id).expect("readable").name, "Original");
        assert!(reopened.notices().is_empty(), "{:?}", reopened.notices());
        assert!(
            !dir.files().iter().any(|name| name.ends_with(".tmp")),
            "{:?}",
            dir.files()
        );
    }

    ops.fail(Step::Write, &format!("{id}.json.tmp"));
    ops.fail(Step::Remove, ".tmp");
    for _ in 0..5 {
        assert!(store.rename(&id, "Changed").is_err());
    }
    ops.heal();
    let temporary: Vec<String> = dir
        .files()
        .into_iter()
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert_eq!(temporary, [format!("{main}.tmp")]);
    assert_eq!(dir.read(&main), before);

    store
        .rename(&id, "Changed")
        .expect("rename after the faults");
    assert_eq!(dir.read(&format!("{main}.bak")), before);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert_eq!(reopened.profile(&id).expect("renamed").name, "Changed");
    assert!(!dir.files().iter().any(|name| name.ends_with(".tmp")));
}

#[test]
fn damaged_profile_is_restored_from_its_backup_with_a_notice() {
    let dir = TempDir::new("backup");
    let id = created(&dir, "First");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.rename(&id, "Second").expect("rename");
    drop(store);
    let main = main_file(&id);
    dir.write(&main, DAMAGED);

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open with damage");

    assert_eq!(store.profile(&id).expect("from backup").name, "First");
    let aside = format!("{id}.json.unreadable-1");
    assert_eq!(
        store.notices(),
        [StoreNotice {
            file: main.clone(),
            event: NoticeEvent::RestoredFromBackup {
                problem: Some(not_json()),
                set_aside: Some(aside.clone()),
                rewritten: true,
            },
        }]
    );
    assert_eq!(dir.read(&format!("profiles/{aside}")), DAMAGED);
    assert_eq!(dir.read(&main), dir.read(&format!("{main}.bak")));
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert!(reopened.notices().is_empty());
    assert_eq!(reopened.profile(&id).expect("restored").name, "First");
}

#[test]
fn missing_profile_file_is_restored_from_its_backup() {
    let dir = TempDir::new("missing-main");
    let id = created(&dir, "First");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.rename(&id, "Second").expect("rename");
    drop(store);
    std::fs::remove_file(dir.file(&main_file(&id))).expect("remove main");

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    assert_eq!(store.profile(&id).expect("from backup").name, "First");
    assert_eq!(
        store.notices()[0].event,
        NoticeEvent::RestoredFromBackup {
            problem: None,
            set_aside: None,
            rewritten: true,
        }
    );
}

#[test]
fn starts_empty_and_keeps_both_files_when_file_and_backup_are_damaged() {
    let dir = TempDir::new("both-damaged");
    dir.write("devices.json", DAMAGED);
    dir.write("devices.json.bak", b"[]");

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    assert!(store.known_devices().is_empty());
    assert_eq!(
        store.notices(),
        [StoreNotice {
            file: "devices.json".into(),
            event: NoticeEvent::StartedEmpty {
                main: Some(not_json()),
                backup: Some(FileProblem::WrongFormat {
                    expected: "prismforge.devices".into(),
                    found: None,
                }),
                set_aside: vec![
                    "devices.json.unreadable-1".into(),
                    "devices.json.bak.unreadable-1".into()
                ],
            },
        }]
    );
    assert_eq!(dir.read("devices.json.unreadable-1"), DAMAGED);
    assert_eq!(dir.read("devices.json.bak.unreadable-1"), b"[]");
    assert!(!dir.file("devices.json").exists());
}

#[test]
fn damaged_profile_does_not_block_the_others() {
    let dir = TempDir::new("one-damaged");
    let damaged = created(&dir, "Damaged");
    let healthy = created(&dir, "Healthy");
    dir.write(&main_file(&damaged), DAMAGED);

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    assert_eq!(store.profile(&healthy).expect("healthy").name, "Healthy");
    assert_eq!(
        store.profile(&damaged),
        Err(StoreError::UnknownProfile {
            id: damaged.clone()
        })
    );
    assert_eq!(store.summary(None).profiles.len(), 1);
    assert_eq!(store.notices().len(), 1);
    assert_eq!(store.notices()[0].file, main_file(&damaged));
    assert_eq!(
        dir.read(&format!("profiles/{damaged}.json.unreadable-1")),
        DAMAGED
    );
}

#[test]
fn registry_write_keeps_its_previous_file_as_backup() {
    let dir = TempDir::new("registry-backup");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let device = |location: &str| KnownDevice {
        reference: DeviceRef::new("dev-alpha"),
        fingerprint: DeviceFingerprint {
            kind: DeviceKind::LedStrip,
            vendor: "Fixture Vendor".into(),
            model: "Strip alpha".into(),
            serial: Some("SN-alpha".into()),
            location: Some(location.into()),
            zones: Vec::new(),
        },
    };
    assert!(
        store
            .record_known_devices(&[device("usb:1")])
            .expect("first")
    );
    assert!(
        !store
            .record_known_devices(&[device("usb:1")])
            .expect("same")
    );
    let first = dir.read("devices.json");
    assert!(
        store
            .record_known_devices(&[device("usb:2")])
            .expect("moved")
    );

    assert_eq!(dir.read("devices.json.bak"), first);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert_eq!(reopened.known_devices(), [device("usb:2")]);
}
