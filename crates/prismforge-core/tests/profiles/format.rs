use prismforge_core::profile::ProfileId;
use prismforge_core::store::{
    FileProblem, NoticeEvent, ProfileStore, StdFileOps, StoreError, UnreadableProfile,
};

use crate::support::{TempDir, full_profile, golden, profile};

const GOLDEN_ID: &str = "p-golden";
const GOLDEN_FILE: &str = "profiles/p-golden.json";

fn id(value: &str) -> ProfileId {
    ProfileId::parse(value).expect("valid id")
}

#[test]
fn roundtrips_profile_v1() {
    let dir = TempDir::new("roundtrip");
    dir.write(GOLDEN_FILE, &golden("profile-v1.1-newer-writer.json"));

    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let loaded = store.profile(&id(GOLDEN_ID)).expect("loaded").clone();
    assert_eq!(loaded, full_profile());
    store.update(&id(GOLDEN_ID), loaded).expect("save");
    let first = dir.read(GOLDEN_FILE);

    let mut reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    let again = reopened.profile(&id(GOLDEN_ID)).expect("reloaded").clone();
    reopened
        .update(&id(GOLDEN_ID), again.clone())
        .expect("save again");
    let second = dir.read(GOLDEN_FILE);

    assert_eq!(
        String::from_utf8(first.clone()).expect("utf-8"),
        String::from_utf8(golden("profile-v1.json")).expect("utf-8")
    );
    assert_eq!(second, first);
    assert_eq!(again, full_profile());
    assert!(store.notices().is_empty());
}

#[test]
fn rejects_future_profile_version() {
    let dir = TempDir::new("future");
    let future = golden("profile-v2.json");
    dir.write("profiles/p-future.json", &future);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let current = store
        .create(profile("Current", Vec::new()))
        .expect("create");

    let problem = FileProblem::NewerVersion {
        found: 2,
        supported: 1,
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
