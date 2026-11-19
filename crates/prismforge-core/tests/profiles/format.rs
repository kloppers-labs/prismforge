use prismforge_core::ApplyMode;
use prismforge_core::profile::ProfileId;
use prismforge_core::setup::ViewId;
use prismforge_core::store::{
    FileProblem, NoticeEvent, ProfileStore, StdFileOps, StoreError, UnreadableProfile,
};
use prismforge_effects::Flow;
use serde_json::json;

use crate::support::{TempDir, full_profile, golden, profile};

const GOLDEN_ID: &str = "p-golden";
const GOLDEN_FILE: &str = "profiles/p-golden.json";

fn id(value: &str) -> ProfileId {
    ProfileId::parse(value).expect("valid id")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("utf-8")
}

#[test]
fn roundtrips_profile_v2() {
    let dir = TempDir::new("roundtrip");
    dir.write(GOLDEN_FILE, &golden("profile-v2.json"));

    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let loaded = store.profile(&id(GOLDEN_ID)).expect("loaded").clone();
    assert_eq!(loaded, full_profile());
    store
        .rename(&id(GOLDEN_ID), "Evening, renamed")
        .expect("save");
    store.rename(&id(GOLDEN_ID), "Evening").expect("save back");
    let first = dir.read(GOLDEN_FILE);

    let mut reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    let again = reopened.profile(&id(GOLDEN_ID)).expect("reloaded").clone();
    reopened
        .rename(&id(GOLDEN_ID), "Evening, renamed")
        .expect("save again");
    reopened
        .rename(&id(GOLDEN_ID), "Evening")
        .expect("save back again");
    let second = dir.read(GOLDEN_FILE);

    assert_eq!(text(first.clone()), text(golden("profile-v2.json")));
    assert_eq!(second, first);
    assert_eq!(again, full_profile());
    assert!(store.notices().is_empty());
}

#[test]
fn migrates_profile_v1_to_v2_without_loss() {
    for sample in ["profile-v1.json", "profile-v1.1-newer-writer.json"] {
        let dir = TempDir::new("migrate");
        let original = golden(sample);
        dir.write(GOLDEN_FILE, &original);

        let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

        assert_eq!(
            store.profile(&id(GOLDEN_ID)).expect("loaded"),
            &full_profile(),
            "{sample}"
        );
        assert!(store.notices().is_empty(), "{sample}");
        assert_eq!(
            dir.read(GOLDEN_FILE),
            original,
            "{sample}: loading writes nothing"
        );
        store
            .rename(&id(GOLDEN_ID), "Evening, renamed")
            .expect("first write");
        assert_eq!(
            dir.read(&format!("{GOLDEN_FILE}.bak")),
            original,
            "{sample}: the version 1 file is the backup of the first write"
        );
        store.rename(&id(GOLDEN_ID), "Evening").expect("save back");
        assert_eq!(
            text(dir.read(GOLDEN_FILE)),
            text(golden("profile-v2.json")),
            "{sample}"
        );
        let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
        assert_eq!(
            reopened.profile(&id(GOLDEN_ID)).expect("v2"),
            &full_profile()
        );
    }
}

#[test]
fn stores_the_apply_mode_of_each_effect() {
    let dir = TempDir::new("apply-mode");
    let mut stored = full_profile();
    let across = ApplyMode::AcrossSetup {
        view: ViewId::parse("v-pc").expect("view id"),
        flow: Flow::Along { angle: 90 },
    };
    stored.assignments[1].mode = across.clone();
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    let created = store.create(stored.clone()).expect("create");
    let file: serde_json::Value =
        serde_json::from_slice(&dir.read(&format!("profiles/{created}.json"))).expect("json");
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");

    assert_eq!(file["assignments"][0]["mode"], json!("EachDevice"));
    assert_eq!(
        file["assignments"][1]["mode"],
        json!({ "AcrossSetup": { "view": "v-pc", "flow": { "Along": { "angle": 90 } } } })
    );
    assert_eq!(reopened.profile(&created).expect("stored"), &stored);
}

#[test]
fn a_version_2_profile_without_a_mode_is_not_valid() {
    let dir = TempDir::new("no-mode");
    let without = text(golden("profile-v2.json")).replacen("\"mode\": \"EachDevice\",\n", "", 1);
    dir.write(GOLDEN_FILE, without.as_bytes());

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    assert!(store.profile(&id(GOLDEN_ID)).is_err());
    assert!(matches!(
        &store.notices()[0].event,
        NoticeEvent::StartedEmpty {
            main: Some(FileProblem::Invalid { .. }),
            ..
        }
    ));
}

#[test]
fn rejects_future_profile_version() {
    let dir = TempDir::new("future");
    let future = golden("profile-v3.json");
    dir.write("profiles/p-future.json", &future);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let current = store
        .create(profile("Current", Vec::new()))
        .expect("create");

    let problem = FileProblem::NewerVersion {
        found: 3,
        supported: 2,
    };
    let refused = StoreError::NewerFile {
        file: "profiles/p-future.json".into(),
        problem: problem.clone(),
    };
    assert_eq!(store.profile(&id("p-future")), Err(refused.clone()));
    assert_eq!(store.rename(&id("p-future"), "Mine"), Err(refused.clone()));
    assert_eq!(store.delete(&id("p-future")), Err(refused.clone()));
    assert_eq!(
        store.set_apply_on_connect(Some(id("p-future"))),
        Err(refused)
    );
    let summary = store.summary(None);
    assert_eq!(
        summary.unreadable,
        [UnreadableProfile {
            id: id("p-future"),
            problem: problem.clone()
        }]
    );
    assert_eq!(summary.profiles.len(), 1);
    assert_eq!(summary.profiles[0].id, current);
    assert_eq!(summary.notices.len(), 1);
    assert_eq!(summary.notices[0].file, "profiles/p-future.json");
    assert_eq!(
        summary.notices[0].event,
        NoticeEvent::LeftUntouched { problem }
    );
    drop(store);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert_eq!(reopened.profile(&current).expect("current").name, "Current");
    assert_eq!(dir.read("profiles/p-future.json"), future);
    assert!(
        !dir.files()
            .iter()
            .any(|name| name.starts_with("profiles/p-future.json.")),
        "{:?}",
        dir.files()
    );
}
