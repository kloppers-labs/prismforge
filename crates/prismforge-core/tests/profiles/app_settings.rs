use prismforge_core::store::{
    AppSettings, AppSettingsFile, CloseAction, FileProblem, NoticeEvent, OpenRgbAddress,
    StdFileOps, StoreError, WriteStep,
};
use serde_json::{Value, json};

use crate::support::{FaultyOps, Step, TempDir};

const FILE: &str = "app-settings.json";

fn address() -> OpenRgbAddress {
    OpenRgbAddress {
        host: "192.168.1.20".into(),
        port: 6742,
    }
}

fn stored(dir: &TempDir) -> Value {
    serde_json::from_slice(&dir.read(FILE)).expect("json")
}

#[test]
fn a_missing_file_gives_every_setting_off_and_writes_nothing() {
    let dir = TempDir::new("app-settings-fresh");

    let file = AppSettingsFile::open(dir.path(), StdFileOps);

    assert_eq!(file.settings(), &AppSettings::default());
    assert_eq!(file.settings().on_close, None);
    assert!(!file.settings().start_hidden);
    assert!(!file.settings().connect_at_start);
    assert_eq!(file.settings().last_address, None);
    assert!(file.notices().is_empty());
    assert!(dir.files().is_empty(), "{:?}", dir.files());
}

#[test]
fn a_change_is_written_with_its_format_and_version_and_read_back() {
    let dir = TempDir::new("app-settings-write");
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);

    file.change(|settings| {
        settings.on_close = Some(CloseAction::Background);
        settings.last_address = Some(address());
        settings.connect_at_start = true;
    })
    .expect("change");

    let json = stored(&dir);
    assert_eq!(json["format"], "prismforge.app-settings");
    assert_eq!(json["schema_version"], json!({ "major": 1, "minor": 0 }));
    assert_eq!(json["on_close"], "Background");
    assert_eq!(
        json["last_address"],
        json!({ "host": "192.168.1.20", "port": 6742 })
    );
    let reopened = AppSettingsFile::open(dir.path(), StdFileOps);
    assert_eq!(reopened.settings(), file.settings());
    assert!(reopened.notices().is_empty());
}

#[test]
fn an_unchanged_setting_writes_no_file_and_keeps_the_backup() {
    let dir = TempDir::new("app-settings-quiet");
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);
    file.change(|settings| settings.start_hidden = true)
        .expect("first");
    file.change(|settings| settings.connect_at_start = false)
        .expect("same");

    assert_eq!(dir.files(), [FILE]);

    file.change(|settings| settings.start_hidden = false)
        .expect("second");
    let backup: Value = serde_json::from_slice(&dir.read(&format!("{FILE}.bak"))).expect("json");
    assert_eq!(backup["start_hidden"], true);
}

#[test]
fn a_failed_write_keeps_the_previous_file_and_value() {
    let dir = TempDir::new("app-settings-failed");
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);
    file.change(|settings| settings.on_close = Some(CloseAction::Quit))
        .expect("first");
    let before = dir.read(FILE);
    let ops = FaultyOps::default();
    let mut file = AppSettingsFile::open(dir.path(), ops.clone());
    let cases = [
        (Step::Write, format!("{FILE}.tmp"), WriteStep::Temporary),
        (Step::Sync, format!("{FILE}.tmp"), WriteStep::Sync),
        (Step::Write, format!("{FILE}.bak.tmp"), WriteStep::Backup),
        (Step::Replace, FILE.to_owned(), WriteStep::Replace),
    ];

    for (step, suffix, expected) in cases {
        ops.fail(step, &suffix);
        let result = file.change(|settings| settings.on_close = Some(CloseAction::Background));
        ops.heal();

        match result {
            Err(StoreError::Write(failure)) => {
                assert_eq!(failure.step, expected);
                assert_eq!(failure.file, FILE);
            }
            other => panic!("{step:?} on {suffix}: {other:?}"),
        }
        assert_eq!(dir.read(FILE), before);
        assert_eq!(file.settings().on_close, Some(CloseAction::Quit));
        assert!(!dir.files().iter().any(|name| name.ends_with(".tmp")));
    }
}

#[test]
fn a_damaged_file_is_restored_from_its_backup_with_a_notice() {
    let dir = TempDir::new("app-settings-backup");
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);
    file.change(|settings| settings.start_hidden = true)
        .expect("first");
    file.change(|settings| settings.connect_at_start = true)
        .expect("second");
    dir.write(FILE, b"{\"format\": \"prismforge.app-se");

    let reopened = AppSettingsFile::open(dir.path(), StdFileOps);

    assert!(reopened.settings().start_hidden);
    assert!(!reopened.settings().connect_at_start);
    let [notice] = reopened.notices() else {
        panic!("{:?}", reopened.notices());
    };
    assert_eq!(notice.file, FILE);
    assert!(matches!(
        &notice.event,
        NoticeEvent::RestoredFromBackup {
            problem: Some(FileProblem::NotJson { .. }),
            set_aside: Some(aside),
            rewritten: true,
        } if aside == "app-settings.json.unreadable-1"
    ));
    assert!(
        dir.files()
            .contains(&"app-settings.json.unreadable-1".to_owned())
    );
}

#[test]
fn a_damaged_file_and_backup_start_with_every_setting_off_and_are_kept_aside() {
    let dir = TempDir::new("app-settings-empty");
    dir.write(FILE, b"not json");
    dir.write(&format!("{FILE}.bak"), b"not json either");

    let file = AppSettingsFile::open(dir.path(), StdFileOps);

    assert_eq!(file.settings(), &AppSettings::default());
    let [notice] = file.notices() else {
        panic!("{:?}", file.notices());
    };
    assert!(matches!(
        &notice.event,
        NoticeEvent::StartedEmpty { set_aside, .. } if set_aside.len() == 2
    ));
}

#[test]
fn a_newer_file_is_left_as_it_is_and_never_written() {
    let dir = TempDir::new("app-settings-newer");
    let newer = br#"{ "format": "prismforge.app-settings", "schema_version": { "major": 2, "minor": 0 }, "start_hidden": true }"#;
    dir.write(FILE, newer);
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);

    let changed = file.change(|settings| settings.connect_at_start = true);

    assert_eq!(file.settings(), &AppSettings::default());
    assert!(matches!(
        file.notices(),
        [notice] if matches!(notice.event, NoticeEvent::LeftUntouched { .. })
    ));
    assert!(matches!(changed, Err(StoreError::NewerFile { .. })));
    assert_eq!(dir.read(FILE), newer);
}

#[test]
fn fields_from_a_newer_minor_version_are_kept_on_write() {
    let dir = TempDir::new("app-settings-minor");
    dir.write(
        FILE,
        br#"{ "format": "prismforge.app-settings", "schema_version": { "major": 1, "minor": 4 }, "start_hidden": true, "theme": "dusk" }"#,
    );
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);

    file.change(|settings| settings.connect_at_start = true)
        .expect("change");

    let json = stored(&dir);
    assert_eq!(json["theme"], "dusk");
    assert_eq!(json["start_hidden"], true);
    assert_eq!(json["schema_version"], json!({ "major": 1, "minor": 0 }));
}

#[test]
fn an_address_the_loader_would_refuse_is_not_written() {
    let dir = TempDir::new("app-settings-invalid");
    let mut file = AppSettingsFile::open(dir.path(), StdFileOps);

    let empty = file.change(|settings| {
        settings.last_address = Some(OpenRgbAddress {
            host: " ".into(),
            port: 6742,
        })
    });
    let port = file.change(|settings| {
        settings.last_address = Some(OpenRgbAddress {
            host: "localhost".into(),
            port: 0,
        })
    });

    assert!(matches!(empty, Err(StoreError::InvalidSettings { .. })));
    assert!(matches!(port, Err(StoreError::InvalidSettings { .. })));
    assert_eq!(file.settings().last_address, None);
    assert!(dir.files().is_empty());
}
