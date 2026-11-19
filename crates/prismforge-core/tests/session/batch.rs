use std::time::Duration;

use prismforge_core::session::{
    BatchReport, MAX_BATCH_TARGETS, OutputError, RestoreOutcome, SessionError, TargetOutcome,
    TargetResult,
};
use prismforge_core::{BackendError, DeviceRef, OutputOwner, OutputTarget, Rgb};
use prismforge_effects::prepare;
use prismforge_testkit::DemoOutput;
use tokio::time::{Instant, sleep};

use crate::support::{
    GPU, LIGHT_BAR, MEMORY, MOTHERBOARD, config, demo, effect, output_of, rainbow, solid, whole,
};

const RED: Rgb = Rgb::new(255, 0, 0);
const MEMORY_2: &str = "demo-memory-2";

fn zone(device: &str, zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: Some(zone),
    }
}

fn outcomes(report: &BatchReport) -> Vec<TargetOutcome> {
    report
        .results
        .iter()
        .map(|result| result.outcome.clone())
        .collect()
}

/// Every frame written after `from`, with the instant it was written and the LEDs it covers.
fn frames_after(
    outputs: Vec<(Instant, DemoOutput)>,
    from: Instant,
) -> Vec<(Instant, String, Vec<Rgb>)> {
    outputs
        .into_iter()
        .filter(|(at, _)| *at >= from)
        .filter_map(|(at, output)| match output {
            DemoOutput::DeviceFrame { device, colors } => Some((at, device.to_string(), colors)),
            DemoOutput::ZoneFrame {
                device,
                zone,
                colors,
            } => Some((at, format!("{device}:{zone}"), colors)),
            _ => None,
        })
        .collect()
}

fn assert_in_phase(frames: &[(Instant, String, Vec<Rgb>)], start: Instant) {
    let effect = prepare(&rainbow()).expect("valid");
    for (at, target, colors) in frames {
        let time = *at - start;
        assert_eq!(
            colors,
            &effect.render_strip(time, colors.len()),
            "{target} at {time:?} is not on the shared clock"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn applies_one_effect_to_several_targets_in_phase() {
    let (handle, controls) = demo(config()).await;
    let targets = vec![whole(GPU), whole(MEMORY), zone(MOTHERBOARD, 1)];

    let start = Instant::now();
    let report = handle
        .apply_batch(targets.clone(), rainbow())
        .await
        .expect("batch");
    sleep(Duration::from_millis(500)).await;

    assert_eq!(outcomes(&report), vec![TargetOutcome::Applied; 3]);
    let frames = frames_after(controls.timed_outputs(), start);
    for name in [GPU, MEMORY, &format!("{MOTHERBOARD}:1")] {
        assert!(
            frames
                .iter()
                .filter(|(_, target, _)| target == name)
                .count()
                > 10,
            "{name} gets frames"
        );
    }
    assert_in_phase(&frames, start);
    let scene = handle.snapshot().output.scene;
    assert_eq!(
        scene
            .assignments
            .iter()
            .map(|assignment| assignment.target.clone())
            .collect::<Vec<_>>(),
        targets
    );
}

#[tokio::test(start_paused = true)]
async fn a_batch_joins_targets_that_already_run_the_effect_and_keeps_their_start() {
    let (handle, controls) = demo(config()).await;
    let first = Instant::now();
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(700)).await;

    let joined = Instant::now();
    handle
        .apply_batch(vec![whole(GPU), whole(MEMORY)], rainbow())
        .await
        .expect("batch");
    sleep(Duration::from_millis(300)).await;

    let frames = frames_after(controls.timed_outputs(), joined);
    assert!(frames.iter().any(|(_, target, _)| target == MEMORY));
    assert_in_phase(&frames, first);
}

#[tokio::test(start_paused = true)]
async fn applying_the_same_batch_again_keeps_the_shared_start() {
    let (handle, controls) = demo(config()).await;
    let targets = vec![whole(GPU), whole(MEMORY)];
    let first = Instant::now();
    handle
        .apply_batch(targets.clone(), rainbow())
        .await
        .expect("batch");
    sleep(Duration::from_millis(400)).await;

    let again = Instant::now();
    let slower = effect("rainbow-wave", &[]);
    handle.apply_batch(targets, slower).await.expect("again");
    sleep(Duration::from_millis(200)).await;

    assert_in_phase(&frames_after(controls.timed_outputs(), again), first);
}

#[tokio::test(start_paused = true)]
async fn reports_each_target_of_a_batch() {
    let (handle, _) = demo(config()).await;
    let targets = vec![
        whole(GPU),
        whole(LIGHT_BAR),
        whole("no-such-device"),
        zone(MOTHERBOARD, 7),
        whole(MEMORY),
    ];

    let report = handle
        .apply_batch(targets.clone(), solid(RED))
        .await
        .expect("batch");

    assert_eq!(
        report.results,
        vec![
            TargetResult {
                target: whole(GPU),
                outcome: TargetOutcome::Applied,
            },
            TargetResult {
                target: whole(LIGHT_BAR),
                outcome: TargetOutcome::Skipped {
                    error: OutputError::Session(SessionError::Backend(
                        BackendError::DirectModeUnavailable {
                            device: DeviceRef::new(LIGHT_BAR),
                        }
                    )),
                },
            },
            TargetResult {
                target: whole("no-such-device"),
                outcome: TargetOutcome::Skipped {
                    error: OutputError::Session(SessionError::Backend(
                        BackendError::UnknownDevice {
                            device: DeviceRef::new("no-such-device"),
                        }
                    )),
                },
            },
            TargetResult {
                target: zone(MOTHERBOARD, 7),
                outcome: TargetOutcome::Skipped {
                    error: OutputError::Session(SessionError::Backend(
                        BackendError::ZoneOutOfRange {
                            device: DeviceRef::new(MOTHERBOARD),
                            zone: 7,
                            zone_count: 3,
                        }
                    )),
                },
            },
            TargetResult {
                target: whole(MEMORY),
                outcome: TargetOutcome::Applied,
            },
        ]
    );
    let scene = handle.snapshot().output.scene;
    assert_eq!(scene.assignments.len(), 2, "a partial result is kept");
}

#[tokio::test(start_paused = true)]
async fn skips_external_devices_in_a_batch() {
    let (handle, controls) = demo(config()).await;
    handle
        .declare_external(DeviceRef::new(MEMORY), "Other app".into())
        .await
        .expect("declare");
    let before = controls.outputs().len();

    let report = handle
        .apply_batch(vec![whole(MEMORY), whole(GPU)], solid(RED))
        .await
        .expect("batch");
    sleep(Duration::from_millis(200)).await;

    assert_eq!(
        outcomes(&report),
        vec![
            TargetOutcome::Skipped {
                error: OutputError::Session(SessionError::OutputConflict {
                    device: DeviceRef::new(MEMORY),
                    owner: OutputOwner::External {
                        label: "Other app".into(),
                    },
                }),
            },
            TargetOutcome::Applied,
        ]
    );
    let written: Vec<DemoOutput> = controls.outputs().split_off(before);
    assert!(
        written.iter().all(|output| !matches!(output,
            DemoOutput::DirectMode { device } | DemoOutput::DeviceFrame { device, .. } | DemoOutput::ZoneFrame { device, .. }
                if device.as_str() == MEMORY)),
        "nothing reaches the external device: {written:?}"
    );
    assert_eq!(
        output_of(&handle.snapshot(), MEMORY).expect("memory").owner,
        OutputOwner::External {
            label: "Other app".into()
        }
    );
}

#[tokio::test(start_paused = true)]
async fn a_whole_device_covers_its_zones_and_a_repeat_in_one_batch() {
    let (handle, _) = demo(config()).await;
    let targets = vec![
        zone(MOTHERBOARD, 0),
        whole(MOTHERBOARD),
        whole(GPU),
        zone(MOTHERBOARD, 2),
        whole(GPU),
    ];

    let report = handle
        .apply_batch(targets, solid(RED))
        .await
        .expect("batch");

    assert_eq!(
        outcomes(&report),
        vec![
            TargetOutcome::Covered {
                by: whole(MOTHERBOARD)
            },
            TargetOutcome::Applied,
            TargetOutcome::Applied,
            TargetOutcome::Covered {
                by: whole(MOTHERBOARD)
            },
            TargetOutcome::Covered { by: whole(GPU) },
        ]
    );
    let targets: Vec<OutputTarget> = handle
        .snapshot()
        .output
        .scene
        .assignments
        .into_iter()
        .map(|assignment| assignment.target)
        .collect();
    assert_eq!(targets, vec![whole(GPU), whole(MOTHERBOARD)]);
}

#[tokio::test(start_paused = true)]
async fn an_invalid_effect_refuses_the_whole_batch() {
    let (handle, controls) = demo(config()).await;

    let result = handle
        .apply_batch(vec![whole(GPU)], effect("solid", &[]))
        .await;

    assert!(matches!(result, Err(OutputError::InvalidEffect(_))));
    assert!(controls.outputs().is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_batch_has_a_bounded_size() {
    let (handle, controls) = demo(config()).await;
    let targets = vec![whole(GPU); MAX_BATCH_TARGETS + 1];

    let result = handle.apply_batch(targets.clone(), solid(RED)).await;
    let stop = handle.stop_batch(targets).await;

    let limit = MAX_BATCH_TARGETS;
    assert_eq!(result, Err(OutputError::TooManyTargets { limit }));
    assert_eq!(stop, Err(OutputError::TooManyTargets { limit }));
    assert!(controls.outputs().is_empty());
}

#[tokio::test(start_paused = true)]
async fn stops_a_batch_and_reports_each_target() {
    let (handle, _) = demo(config()).await;
    handle
        .apply_batch(
            vec![whole(GPU), zone(MOTHERBOARD, 0), zone(MOTHERBOARD, 1)],
            rainbow(),
        )
        .await
        .expect("batch");

    let report = handle
        .stop_batch(vec![
            zone(MOTHERBOARD, 0),
            whole(GPU),
            whole(MEMORY),
            zone(MOTHERBOARD, 1),
        ])
        .await
        .expect("stop");

    assert_eq!(
        outcomes(&report),
        vec![
            TargetOutcome::Stopped { restore: None },
            TargetOutcome::Stopped {
                restore: Some(RestoreOutcome::Restored)
            },
            TargetOutcome::Skipped {
                error: OutputError::NoEffect {
                    target: whole(MEMORY)
                },
            },
            TargetOutcome::Stopped {
                restore: Some(RestoreOutcome::Restored)
            },
        ]
    );
    assert!(handle.snapshot().output.scene.assignments.is_empty());
}

#[tokio::test(start_paused = true)]
async fn stop_all_stops_every_device_with_the_batch_form() {
    let (handle, _) = demo(config()).await;
    handle
        .apply_batch(
            vec![zone(MOTHERBOARD, 2), whole(GPU), whole(MEMORY_2)],
            rainbow(),
        )
        .await
        .expect("batch");
    handle
        .select_mode(DeviceRef::new(MEMORY), 2)
        .await
        .expect("mode");

    let report = handle.stop_all_effects().await.expect("stop all");

    let mut stopped: Vec<OutputTarget> = report
        .results
        .iter()
        .map(|result| {
            assert!(
                matches!(result.outcome, TargetOutcome::Stopped { .. }),
                "{result:?}"
            );
            result.target.clone()
        })
        .collect();
    stopped.sort_by(|left, right| left.device.cmp(&right.device));
    assert_eq!(
        stopped,
        vec![whole(GPU), whole(MEMORY_2), whole(MOTHERBOARD)]
    );
    let snapshot = handle.snapshot();
    assert!(snapshot.output.scene.assignments.is_empty());
    assert_eq!(
        output_of(&snapshot, MEMORY).expect("memory").owner,
        OutputOwner::HardwareMode { mode: 2 },
        "stop all ends effects only"
    );
}
