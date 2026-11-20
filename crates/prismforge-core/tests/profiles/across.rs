use std::time::Duration;

use prismforge_core::profile::{Change, ItemOutcome, SkipReason};
use prismforge_core::session::SessionHandle;
use prismforge_core::setup::{
    Component, ComponentId, FirstLed, Geometry, OutputId, Placement, Setup, SetupOutput, Strip,
    View, ViewId, plan_across,
};
use prismforge_core::store::{ApplyOutcome, Profiles, Readiness, StdFileOps, StoreError};
use prismforge_core::{ApplyMode, DeviceRef, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, EffectInstance, Flow, prepare};
use prismforge_testkit::DemoOutput;
use serde_json::Map;
use tokio::time::{Instant, sleep};

use crate::session::{demo, whole};
use crate::support::TempDir;

const MEMORY_1: &str = "demo-memory-1";
const MEMORY_2: &str = "demo-memory-2";
const DOWN: Flow = Flow::Along { angle: 90 };

fn view() -> ViewId {
    ViewId::parse("v-desk").expect("view id")
}

fn across() -> ApplyMode {
    ApplyMode::AcrossSetup {
        view: view(),
        flow: DOWN,
    }
}

fn wave() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: [("speed".to_owned(), ControlValue::Number(200))].into(),
    }
}

/// The two demo memory modules as two 8-LED strips, one above the other.
fn desk() -> Setup {
    let mut setup = Setup::default();
    let mut placements = Vec::new();
    for (index, device) in [MEMORY_1, MEMORY_2].into_iter().enumerate() {
        let component = ComponentId::parse(&format!("c-{index}")).expect("component id");
        setup.components.push(Component {
            id: component.clone(),
            name: format!("Module {index}"),
            geometry: Geometry::Strip(Strip {
                leds: 8,
                first: FirstLed::AtStart,
                extra: Map::new(),
            }),
            frame: None,
            extra: Map::new(),
        });
        setup.outputs.push(SetupOutput {
            id: OutputId::parse(&format!("o-{index}")).expect("output id"),
            device: DeviceRef::new(device),
            zone: None,
            label: "Demo Memory Module".into(),
            zone_name: None,
            first_led: 0,
            branches: vec![vec![component.clone()]],
            extra: Map::new(),
        });
        placements.push(Placement {
            component,
            x: 0,
            y: 40 * i32::try_from(index).expect("small"),
            size: 80,
            rotation: 0,
            flipped: false,
            extra: Map::new(),
        });
    }
    setup.views.push(View {
        id: view(),
        name: "Desk".into(),
        placements,
        outlines: Vec::new(),
        extra: Map::new(),
    });
    setup
}

async fn store_with_desk(dir: &TempDir, handle: &SessionHandle) -> Profiles {
    let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open");
    profiles
        .save_setup(desk(), Some(handle))
        .await
        .expect("setup");
    profiles
}

async fn apply_across(handle: &SessionHandle) {
    let plan = plan_across(&desk(), &view(), DOWN, &handle.snapshot().devices).expect("view");
    handle.apply_across(plan, wave()).await.expect("across");
}

fn modes(profile: &prismforge_core::profile::Profile) -> Vec<(String, ApplyMode)> {
    profile
        .assignments
        .iter()
        .map(|assignment| (assignment.device.to_string(), assignment.mode.clone()))
        .collect()
}

#[tokio::test(start_paused = true)]
async fn an_update_saves_the_mode_each_effect_was_applied_with() {
    let dir = TempDir::new("across-update");
    let (handle, _) = demo().await;
    let profiles = store_with_desk(&dir, &handle).await;
    let id = profiles
        .create_from_session(&handle, "Desk")
        .await
        .expect("create")
        .id;

    apply_across(&handle).await;
    profiles
        .update_from_session(&handle, &id)
        .await
        .expect("update");
    let stored = profiles.profile(&id).await.expect("stored");
    handle
        .apply_effect(whole(&DeviceRef::new(MEMORY_1)), wave())
        .await
        .expect("each device");
    let saved = profiles
        .update_from_session(&handle, &id)
        .await
        .expect("second update");
    let again = profiles.profile(&id).await.expect("stored again");

    assert_eq!(
        modes(&stored),
        vec![(MEMORY_1.into(), across()), (MEMORY_2.into(), across())]
    );
    assert_eq!(
        modes(&again),
        vec![
            (MEMORY_1.into(), ApplyMode::EachDevice),
            (MEMORY_2.into(), across())
        ]
    );
    assert_eq!(saved.changes.len(), 1);
    assert!(matches!(saved.changes[0].change, Change::Replaced { .. }));
}

#[tokio::test(start_paused = true)]
async fn applying_a_profile_runs_its_effects_across_the_setup_in_one_batch() {
    let dir = TempDir::new("across-apply");
    let (handle, controls) = demo().await;
    let profiles = store_with_desk(&dir, &handle).await;
    apply_across(&handle).await;
    let id = profiles
        .create_from_session(&handle, "Desk")
        .await
        .expect("create")
        .id;
    handle.stop_all_effects().await.expect("stop");
    let first = Instant::now();
    handle
        .apply_effect(whole(&DeviceRef::new(MEMORY_1)), wave())
        .await
        .expect("running alone");
    sleep(Duration::from_millis(500)).await;

    let report = match profiles.apply(&handle, &id).await.expect("apply") {
        ApplyOutcome::Applied(report) => report,
        other => panic!("{other:?}"),
    };
    let applied = Instant::now();
    sleep(Duration::from_millis(300)).await;

    assert!(
        report
            .items
            .iter()
            .all(|item| item.outcome == ItemOutcome::Applied)
    );
    let modes: Vec<ApplyMode> = handle
        .snapshot()
        .output
        .scene
        .assignments
        .into_iter()
        .map(|assignment| assignment.mode)
        .collect();
    assert_eq!(modes, vec![across(), across()]);
    let plan = plan_across(&desk(), &view(), DOWN, &handle.snapshot().devices).expect("view");
    let effect = prepare(&wave()).expect("valid");
    let frames: Vec<(Instant, String, Vec<Rgb>)> = controls
        .timed_outputs()
        .into_iter()
        .filter(|(at, _)| *at > applied)
        .filter_map(|(at, output)| match output {
            DemoOutput::DeviceFrame { device, colors } => Some((at, device.to_string(), colors)),
            _ => None,
        })
        .collect();
    assert!(frames.iter().any(|(_, device, _)| device == MEMORY_2));
    for (at, device, colors) in frames {
        let target = OutputTarget {
            device: DeviceRef::new(&device),
            zone: None,
        };
        let placed = plan
            .targets
            .iter()
            .find(|placed| placed.target == target)
            .expect("planned");
        assert_eq!(
            colors,
            effect.render_across(at - first, DOWN, plan.plane(), &placed.points),
            "{device} joins the one picture on the shared start"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn an_effect_across_a_view_the_setup_no_longer_has_is_skipped() {
    let dir = TempDir::new("across-gone");
    let (handle, controls) = demo().await;
    let profiles = store_with_desk(&dir, &handle).await;
    apply_across(&handle).await;
    let id = profiles
        .create_from_session(&handle, "Desk")
        .await
        .expect("create")
        .id;
    handle.stop_all_effects().await.expect("stop");
    let mut without = desk();
    without.views.clear();
    profiles
        .save_setup(without, Some(&handle))
        .await
        .expect("view removed");
    let before = controls.outputs().len();

    let details = profiles.details(&id, Some(&handle)).await.expect("details");
    let report = match profiles.apply(&handle, &id).await.expect("apply") {
        ApplyOutcome::Applied(report) => report,
        other => panic!("{other:?}"),
    };
    sleep(Duration::from_millis(200)).await;

    let skipped = ItemOutcome::Skipped {
        reason: SkipReason::SetupChanged,
    };
    assert_eq!(report.items.len(), 2);
    assert!(report.items.iter().all(|item| item.outcome == skipped));
    assert!(details.settings.iter().all(|setting| setting.today
        == Readiness::Skipped {
            reason: SkipReason::SetupChanged
        }));
    let written = controls.outputs().split_off(before);
    assert!(
        written
            .iter()
            .all(|output| !matches!(output, DemoOutput::DeviceFrame { .. })),
        "{written:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn effects_across_a_setup_that_needs_a_newer_prismforge_say_so() {
    let dir = TempDir::new("across-newer");
    let (handle, _) = demo().await;
    let profiles = store_with_desk(&dir, &handle).await;
    apply_across(&handle).await;
    let id = profiles
        .create_from_session(&handle, "Desk")
        .await
        .expect("create")
        .id;
    handle.stop_all_effects().await.expect("stop");
    drop(profiles);
    let newer = serde_json::to_vec(&serde_json::json!({
        "format": "prismforge.setup",
        "schema_version": { "major": 2, "minor": 0 }
    }))
    .expect("encode");
    dir.write("setup.json", &newer);
    let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("reopen");

    let details = profiles.details(&id, Some(&handle)).await.expect("details");
    let report = match profiles.apply(&handle, &id).await.expect("apply") {
        ApplyOutcome::Applied(report) => report,
        other => panic!("{other:?}"),
    };

    let reason = SkipReason::NewerSetup;
    assert!(report.items.iter().all(|item| item.outcome
        == ItemOutcome::Skipped {
            reason: reason.clone()
        }));
    assert!(details.settings.iter().all(|setting| setting.today
        == Readiness::Skipped {
            reason: reason.clone()
        }));
    assert!(matches!(
        profiles.setup().await,
        Err(StoreError::NewerFile { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn apply_and_details_give_one_reason_for_an_external_device_that_lost_its_place() {
    let dir = TempDir::new("across-order");
    let (handle, _) = demo().await;
    let profiles = store_with_desk(&dir, &handle).await;
    apply_across(&handle).await;
    let id = profiles
        .create_from_session(&handle, "Desk")
        .await
        .expect("create")
        .id;
    handle.stop_all_effects().await.expect("stop");
    handle
        .declare_external(DeviceRef::new(MEMORY_1), "Other app".into())
        .await
        .expect("declare");
    let mut without = desk();
    without.views.clear();
    profiles
        .save_setup(without, Some(&handle))
        .await
        .expect("view removed");

    let details = profiles.details(&id, Some(&handle)).await.expect("details");
    let report = match profiles.apply(&handle, &id).await.expect("apply") {
        ApplyOutcome::Applied(report) => report,
        other => panic!("{other:?}"),
    };

    let memory = DeviceRef::new(MEMORY_1);
    let applied = report
        .items
        .iter()
        .find(|item| item.device == memory)
        .expect("item");
    let checked = details
        .settings
        .iter()
        .find(|setting| setting.device == memory)
        .expect("setting");
    assert_eq!(
        applied.outcome,
        ItemOutcome::Skipped {
            reason: SkipReason::SetupChanged
        }
    );
    assert_eq!(
        checked.today,
        Readiness::Skipped {
            reason: SkipReason::SetupChanged
        }
    );
}
