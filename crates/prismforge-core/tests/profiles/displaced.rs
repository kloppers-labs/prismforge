use prismforge_core::profile::{ProfileId, ProfileItem};
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{ApplyOutcome, Profiles, StdFileOps};
use prismforge_core::{DeviceRef, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};

use crate::session::{demo, whole};
use crate::support::TempDir;

const BOARD: &str = "demo-motherboard";

fn solid(color: Rgb) -> EffectInstance {
    EffectInstance {
        effect: "solid".into(),
        values: [("color".to_owned(), ControlValue::Color(color))].into(),
    }
}

fn board() -> DeviceRef {
    DeviceRef::new(BOARD)
}

fn zone(index: usize) -> OutputTarget {
    OutputTarget {
        device: board(),
        zone: Some(index),
    }
}

/// A demo session whose motherboard runs `target`, saved and applied as the active profile.
async fn applied_with(dir: &TempDir, target: OutputTarget) -> (Profiles, SessionHandle, ProfileId) {
    let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open");
    let (handle, _) = demo().await;
    handle
        .apply_effect(target, solid(Rgb::new(5, 5, 5)))
        .await
        .expect("stored effect");
    let id = profiles
        .create_from_session(&handle, "Board")
        .await
        .expect("create")
        .id;
    match profiles.apply(&handle, &id).await.expect("apply") {
        ApplyOutcome::Applied(_) => {}
        other => panic!("{other:?}"),
    }
    (profiles, handle, id)
}

fn effects_on_board(profile: &prismforge_core::profile::Profile) -> Vec<Option<usize>> {
    profile
        .assignments
        .iter()
        .filter(|assignment| assignment.device == board())
        .map(|assignment| assignment.zone)
        .collect()
}

#[tokio::test(start_paused = true)]
async fn a_zone_apply_that_displaced_the_whole_device_effect_is_saved_after_its_stop() {
    let dir = TempDir::new("displaced-whole");
    let (profiles, handle, id) = applied_with(&dir, whole(&board())).await;

    handle
        .apply_effect(zone(0), solid(Rgb::new(1, 1, 1)))
        .await
        .expect("zone");
    handle.stop_effect(zone(0)).await.expect("stop zone");
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    assert!(handle.snapshot().output.scene.assignments.is_empty());
    assert_eq!(
        effects_on_board(&profiles.profile(&id).await.expect("saved")),
        []
    );
    assert_eq!(saved.changes.len(), 1);
    assert_eq!(
        saved.changes[0].change,
        prismforge_core::profile::Change::Removed {
            was: ProfileItem::Effect {
                zone: None,
                effect: "solid".into()
            }
        }
    );
}

#[tokio::test(start_paused = true)]
async fn a_whole_device_effect_displaced_in_turn_is_not_saved() {
    let dir = TempDir::new("displaced-chain");
    let (profiles, handle, id) = applied_with(&dir, zone(1)).await;

    handle
        .apply_effect(whole(&board()), solid(Rgb::new(2, 2, 2)))
        .await
        .expect("whole");
    handle
        .apply_effect(zone(0), solid(Rgb::new(3, 3, 3)))
        .await
        .expect("zone 0");
    handle.stop_effect(zone(0)).await.expect("stop zone 0");
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");

    assert!(handle.snapshot().output.scene.assignments.is_empty());
    assert_eq!(
        effects_on_board(&profiles.profile(&id).await.expect("saved")),
        []
    );
}
