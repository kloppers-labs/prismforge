use std::sync::Arc;
use std::time::Duration;

use prismforge_core::session::{
    BatchReport, MAX_BATCH_TARGETS, OutputError, SessionError, SessionHandle, TargetOutcome,
};
use prismforge_core::setup::{
    AcrossPlan, Component, ComponentId, FirstLed, Geometry, OutputId, Placement, Setup,
    SetupOutput, Strip, View, ViewId, plan_across,
};
use prismforge_core::{ApplyMode, BackendError, DeviceRef, OutputOwner, OutputTarget, Rgb};
use prismforge_effects::{Flow, prepare};
use prismforge_testkit::{DemoOutput, FakeServer, FixtureDevice};
use serde_json::Map;
use tokio::time::{Instant, sleep};

use crate::support::{
    self, GPU, GPU_LEDS, MEMORY, config, demo, device_frames, effect, rainbow, reference, strip,
    wait_until, whole,
};

const MEMORY_2: &str = "demo-memory-2";
const RIGHT: Flow = Flow::Along { angle: 0 };
const BLACK: Rgb = Rgb::new(0, 0, 0);

fn view_id() -> ViewId {
    ViewId::parse("v-desk").expect("view id")
}

fn fast_rainbow() -> prismforge_effects::EffectInstance {
    effect(
        "rainbow-wave",
        &[("speed", prismforge_effects::ControlValue::Number(240))],
    )
}

/// One straight strip of `leds` per device, wired to the whole device and placed left to right.
fn strips(devices: &[(&str, u32)]) -> Setup {
    let mut setup = Setup::default();
    let mut placements = Vec::new();
    for (index, (device, leds)) in devices.iter().enumerate() {
        let component = ComponentId::parse(&format!("c-{index}")).expect("component id");
        setup.components.push(Component {
            id: component.clone(),
            name: format!("Strip {index}"),
            geometry: Geometry::Strip(Strip {
                leds: *leds,
                first: FirstLed::AtStart,
                extra: Map::new(),
            }),
            extra: Map::new(),
        });
        setup.outputs.push(SetupOutput {
            id: OutputId::parse(&format!("o-{index}")).expect("output id"),
            device: DeviceRef::new(*device),
            zone: None,
            label: (*device).into(),
            zone_name: None,
            first_led: 0,
            branches: vec![vec![component.clone()]],
            extra: Map::new(),
        });
        placements.push(Placement {
            component,
            x: i32::try_from(index).expect("small") * 100,
            y: 0,
            size: 80,
            rotation: 0,
            flipped: false,
            extra: Map::new(),
        });
    }
    setup.views.push(View {
        id: view_id(),
        name: "Desk".into(),
        placements,
        extra: Map::new(),
    });
    setup
}

fn plan(handle: &SessionHandle, setup: &Setup, flow: Flow) -> AcrossPlan {
    plan_across(setup, &view_id(), flow, &handle.snapshot().devices).expect("view")
}

fn outcomes(report: &BatchReport) -> Vec<TargetOutcome> {
    report
        .results
        .iter()
        .map(|result| result.outcome.clone())
        .collect()
}

fn device_frames_after(
    outputs: Vec<(Instant, DemoOutput)>,
    from: Instant,
) -> Vec<(Instant, String, Vec<Rgb>)> {
    outputs
        .into_iter()
        .filter(|(at, _)| *at >= from)
        .filter_map(|(at, output)| match output {
            DemoOutput::DeviceFrame { device, colors } => Some((at, device.to_string(), colors)),
            _ => None,
        })
        .collect()
}

#[tokio::test(start_paused = true)]
async fn applies_one_picture_across_several_devices_with_one_start() {
    let (handle, controls) = demo(config()).await;
    let setup = strips(&[(MEMORY, 8), (MEMORY_2, 8)]);
    let plan = plan(&handle, &setup, RIGHT);

    let start = Instant::now();
    let report = handle
        .apply_across(plan.clone(), fast_rainbow())
        .await
        .expect("across");
    sleep(Duration::from_millis(400)).await;

    assert_eq!(outcomes(&report), vec![TargetOutcome::Applied; 2]);
    let effect = prepare(&fast_rainbow()).expect("valid");
    let frames = device_frames_after(controls.timed_outputs(), start);
    for device in [MEMORY, MEMORY_2] {
        assert!(frames.iter().filter(|(_, name, _)| name == device).count() > 5);
    }
    for (at, device, colors) in &frames {
        let placed = plan
            .targets
            .iter()
            .find(|placed| placed.target.device.as_str() == device)
            .expect("planned");
        assert_eq!(
            colors,
            &effect.render_across(*at - start, RIGHT, plan.plane, &placed.points, 0),
            "{device} at {:?} is not on the shared picture",
            *at - start
        );
    }
    let mode = ApplyMode::AcrossSetup {
        view: view_id(),
        flow: RIGHT,
    };
    let modes: Vec<(OutputTarget, ApplyMode)> = handle
        .snapshot()
        .output
        .scene
        .assignments
        .into_iter()
        .map(|assignment| (assignment.target, assignment.mode))
        .collect();
    assert_eq!(
        modes,
        vec![(whole(MEMORY), mode.clone()), (whole(MEMORY_2), mode)]
    );
}

#[tokio::test(start_paused = true)]
async fn the_mode_each_device_keeps_its_frames_beside_a_picture_across_the_setup() {
    let (handle, controls) = demo(config()).await;
    let start = Instant::now();
    handle
        .apply_batch(vec![whole(GPU)], rainbow())
        .await
        .expect("batch");
    let setup = strips(&[(MEMORY, 8), (MEMORY_2, 8)]);
    handle
        .apply_across(plan(&handle, &setup, RIGHT), rainbow())
        .await
        .expect("across");
    sleep(Duration::from_millis(300)).await;

    let strip_effect = prepare(&rainbow()).expect("valid");
    let frames = device_frames_after(controls.timed_outputs(), start);
    let gpu: Vec<_> = frames.iter().filter(|(_, name, _)| name == GPU).collect();
    assert!(gpu.len() > 5);
    for (at, _, colors) in gpu {
        assert_eq!(colors, &strip_effect.render_strip(*at - start, GPU_LEDS));
    }
    assert_eq!(
        handle.snapshot().output.scene.assignments[0].mode,
        ApplyMode::EachDevice
    );
}

#[tokio::test(start_paused = true)]
async fn leds_without_a_place_are_written_black_in_full_frames() {
    let (handle, controls) = demo(config()).await;
    let setup = strips(&[(GPU, 4)]);

    handle
        .apply_across(
            plan(&handle, &setup, RIGHT),
            support::solid(Rgb::new(9, 9, 9)),
        )
        .await
        .expect("across");
    sleep(Duration::from_millis(100)).await;

    let frames = device_frames(&controls.outputs(), GPU);
    let last = frames.last().expect("a frame");
    assert_eq!(last.len(), GPU_LEDS);
    assert_eq!(&last[..4], &[Rgb::new(9, 9, 9); 4]);
    assert_eq!(&last[4..], &[BLACK; GPU_LEDS - 4]);
}

#[tokio::test(start_paused = true)]
async fn an_external_device_in_the_setup_is_skipped_and_gets_nothing() {
    let (handle, controls) = demo(config()).await;
    handle
        .declare_external(DeviceRef::new(MEMORY_2), "Other app".into())
        .await
        .expect("declare");
    let before = controls.outputs().len();
    let setup = strips(&[(MEMORY, 8), (MEMORY_2, 8)]);

    let report = handle
        .apply_across(plan(&handle, &setup, RIGHT), rainbow())
        .await
        .expect("across");
    sleep(Duration::from_millis(200)).await;

    assert_eq!(
        outcomes(&report),
        vec![
            TargetOutcome::Applied,
            TargetOutcome::Skipped {
                error: OutputError::Session(SessionError::OutputConflict {
                    device: DeviceRef::new(MEMORY_2),
                    owner: OutputOwner::External {
                        label: "Other app".into(),
                    },
                }),
            },
        ]
    );
    let written = controls.outputs().split_off(before);
    assert!(
        written.iter().all(|output| !matches!(output,
            DemoOutput::DirectMode { device } | DemoOutput::DeviceFrame { device, .. } | DemoOutput::ZoneFrame { device, .. }
                if device.as_str() == MEMORY_2)),
        "nothing reaches the external device: {written:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn an_ambiguous_device_in_the_setup_is_skipped_and_gets_nothing() {
    let twin = |label: &str| FixtureDevice::new(label, "Twin Fan").zone("Ring", 4);
    let server = FakeServer::new();
    server.add_device(0, twin("left"));
    server.add_device(1, strip("alpha", 3));
    let handle = support::connected_openrgb(&server).await;
    let left = handle.snapshot().devices[0]
        .identity
        .reference()
        .cloned()
        .expect("single twin resolves");
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    let setup = strips(&[(left.as_str(), 4), (alpha.as_str(), 3)]);
    server.restart(vec![
        (0, twin("left").into()),
        (1, twin("right").into()),
        (2, strip("alpha", 3).into()),
    ]);
    wait_until(&handle, support::is_retrying).await;
    wait_until(&handle, support::is_live).await;
    let writes_before = server.color_writes().len();

    let report = handle
        .apply_across(plan(&handle, &setup, RIGHT), rainbow())
        .await
        .expect("across");
    sleep(Duration::from_millis(200)).await;

    assert_eq!(
        outcomes(&report),
        vec![
            TargetOutcome::Skipped {
                error: OutputError::Session(SessionError::Backend(BackendError::AmbiguousDevice {
                    device: left
                })),
            },
            TargetOutcome::Applied,
        ]
    );
    let new_writes = server.color_writes().split_off(writes_before);
    assert!(!new_writes.is_empty());
    assert!(
        new_writes.iter().all(|write| write.device_id == 2),
        "only the strip gets frames: {new_writes:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn the_same_effect_keeps_its_start_when_the_mode_changes() {
    let (handle, controls) = demo(config()).await;
    let first = Instant::now();
    handle
        .apply_effect(whole(MEMORY), fast_rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(700)).await;
    let setup = strips(&[(MEMORY, 8), (MEMORY_2, 8)]);
    let plan = plan(&handle, &setup, RIGHT);

    let switched = Instant::now();
    handle
        .apply_across(plan.clone(), fast_rainbow())
        .await
        .expect("across");
    sleep(Duration::from_millis(300)).await;

    let effect = prepare(&fast_rainbow()).expect("valid");
    for (at, device, colors) in device_frames_after(controls.timed_outputs(), switched) {
        let placed = plan
            .targets
            .iter()
            .find(|placed| placed.target.device.as_str() == device)
            .expect("planned");
        assert_eq!(
            colors,
            effect.render_across(at - first, RIGHT, plan.plane, &placed.points, 0),
            "{device} joins the running animation"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn stopping_the_targets_of_a_picture_restores_each_device() {
    let (handle, _) = demo(config()).await;
    let setup = strips(&[(MEMORY, 8), (GPU, 12)]);
    let plan = plan(&handle, &setup, RIGHT);
    handle
        .apply_across(plan.clone(), rainbow())
        .await
        .expect("across");

    let targets: Vec<OutputTarget> = plan
        .targets
        .iter()
        .map(|placed| placed.target.clone())
        .collect();
    let report = handle.stop_batch(targets).await.expect("stop");

    assert!(
        outcomes(&report)
            .iter()
            .all(|outcome| matches!(outcome, TargetOutcome::Stopped { restore: Some(_) })),
        "{report:?}"
    );
    assert!(handle.snapshot().output.scene.assignments.is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_picture_has_a_bounded_number_of_targets() {
    let (handle, controls) = demo(config()).await;
    let setup = strips(&[(MEMORY, 8)]);
    let mut plan = plan(&handle, &setup, RIGHT);
    let one = plan.targets[0].clone();
    plan.targets = (0..=MAX_BATCH_TARGETS)
        .map(|_| prismforge_core::setup::PlacedTarget {
            target: one.target.clone(),
            points: Arc::clone(&one.points),
        })
        .collect();
    let before = controls.outputs().len();

    let refused = handle.apply_across(plan, rainbow()).await;

    assert_eq!(
        refused,
        Err(OutputError::TooManyTargets {
            limit: MAX_BATCH_TARGETS
        })
    );
    assert_eq!(controls.outputs().len(), before);
}
