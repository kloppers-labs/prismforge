use prismforge_core::automation::Trigger;
use std::time::Duration;

use prismforge_core::store::{ApplyOutcome, Profiles, StdFileOps};
use prismforge_core::{DeviceRef, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};
use tokio::time::sleep;

use crate::session::{demo, whole};
use crate::support::{TempDir, connects_rule};

#[tokio::test(start_paused = true)]
async fn read_only_queries_do_not_wake_snapshot_subscribers() {
    let dir = TempDir::new("quiet");
    let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open");
    let (handle, _) = demo().await;
    handle
        .select_mode(DeviceRef::new("demo-light-bar"), 1)
        .await
        .expect("mode");
    let id = profiles
        .create_from_session(&handle, "Quiet")
        .await
        .expect("create")
        .id;
    match profiles.apply(&handle, &id).await.expect("apply") {
        ApplyOutcome::Applied(_) => {}
        other => panic!("{other:?}"),
    }
    sleep(Duration::from_secs(1)).await;
    let mut updates = handle.subscribe();
    updates.mark_unchanged();

    profiles.summary(Some(&handle)).await;
    let after_summary = updates.has_changed().expect("session runs");
    profiles
        .preview_update(&handle, &id)
        .await
        .expect("preview");
    let after_preview = updates.has_changed().expect("session runs");
    profiles.details(&id, Some(&handle)).await.expect("details");
    let after_details = updates.has_changed().expect("session runs");
    handle
        .apply_effect(
            whole(&DeviceRef::new("demo-memory-1")),
            EffectInstance {
                effect: "solid".into(),
                values: [("color".to_owned(), ControlValue::Color(Rgb::new(1, 1, 1)))].into(),
            },
        )
        .await
        .expect("a state change");
    let after_change = updates.has_changed().expect("session runs");

    assert!(!after_summary);
    assert!(!after_preview);
    assert!(!after_details);
    assert!(after_change);
}

#[test]
fn saves_that_change_nothing_write_nothing() {
    let dir = TempDir::new("no-write");
    let mut store =
        prismforge_core::store::ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let id = store
        .create(crate::support::profile("Same", Vec::new()))
        .expect("create");
    store.rename(&id, "Other").expect("rename");
    store.rename(&id, "Same").expect("rename back");
    store
        .set_rule(Trigger::Connects, connects_rule(&id))
        .expect("mark");
    let files = dir.files();
    let main = format!("profiles/{id}.json");
    let before: Vec<Vec<u8>> = files.iter().map(|name| dir.read(name)).collect();

    store.rename(&id, "Same").expect("same name");
    let current = store.profile(&id).expect("profile").clone();
    store.update(&id, current).expect("same content");
    store
        .set_rule(Trigger::Connects, connects_rule(&id))
        .expect("same mark");
    assert!(!store.record_known_devices(&[]).expect("same registry"));

    assert_eq!(dir.files(), files);
    let after: Vec<Vec<u8>> = files.iter().map(|name| dir.read(name)).collect();
    assert_eq!(after, before);
    assert_ne!(dir.read(&main), dir.read(&format!("{main}.bak")));
}
