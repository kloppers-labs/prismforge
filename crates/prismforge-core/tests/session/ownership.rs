use std::time::Duration;

use prismforge_core::session::{
    IDENTIFY_COLOR, OutputCondition, OutputError, PauseReason, RestoreOutcome, Session,
    SessionError,
};
use prismforge_core::{BackendError, DeviceRef, OutputOwner, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, InstanceError, ValueError, ValueProblem};
use prismforge_testkit::{DemoBackend, DemoOutput, FakeServer, FixtureDevice, wire};
use tokio::time::{Instant, sleep};

use crate::support::{
    self, GPU, GPU_LEDS, LIGHT_BAR, MEMORY, MOTHERBOARD, config, demo, device_frames, effect,
    is_connected, is_retrying, output_of, output_packets, rainbow, reference, solid, strip,
    wait_until, whole,
};

const PLUGIN: &str = "OpenRGB Effects plugin";
const BLINK: Duration = Duration::from_secs(2);

fn gpu() -> DeviceRef {
    DeviceRef::new(GPU)
}

#[tokio::test(start_paused = true)]
async fn refuses_conflicting_output_owner() {
    let (handle, controls) = demo(config()).await;
    handle
        .declare_external(gpu(), PLUGIN.into())
        .await
        .expect("declare");

    let refused = handle.apply_effect(whole(GPU), rainbow()).await;
    handle.refresh().await.expect("barrier");

    let external = OutputOwner::External {
        label: PLUGIN.into(),
    };
    assert_eq!(
        refused,
        Err(OutputError::Session(SessionError::OutputConflict {
            device: gpu(),
            owner: external.clone()
        }))
    );
    assert!(controls.outputs().is_empty());
    assert_eq!(
        output_of(&handle.snapshot(), GPU)
            .expect("gpu output")
            .owner,
        external
    );

    handle.clear_external(gpu()).await.expect("hand over");
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply after hand-over");
    sleep(Duration::from_millis(100)).await;

    let outputs = controls.outputs();
    assert_eq!(outputs[0], DemoOutput::DirectMode { device: gpu() });
    assert!(!device_frames(&outputs, GPU).is_empty());
    assert_eq!(
        output_of(&handle.snapshot(), GPU)
            .expect("gpu output")
            .owner,
        OutputOwner::Effect
    );
}

#[tokio::test(start_paused = true)]
async fn declaring_external_control_stops_output_without_a_restore() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    handle
        .declare_external(gpu(), PLUGIN.into())
        .await
        .expect("declare");
    let after = controls.outputs();
    sleep(Duration::from_secs(10)).await;

    assert_eq!(controls.outputs(), after);
    assert!(
        !after
            .iter()
            .any(|output| matches!(output, DemoOutput::ModeReapplied { .. }))
    );
    assert!(handle.snapshot().output.scene.assignments.is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_device_without_direct_mode_is_refused_before_any_write() {
    let (handle, controls) = demo(config()).await;

    let refused = handle.apply_effect(whole(LIGHT_BAR), rainbow()).await;
    handle.refresh().await.expect("barrier");

    assert_eq!(
        refused,
        Err(OutputError::Session(SessionError::Backend(
            BackendError::DirectModeUnavailable {
                device: DeviceRef::new(LIGHT_BAR)
            }
        )))
    );
    assert!(controls.outputs().is_empty());
    assert!(handle.snapshot().output.devices.is_empty());
}

#[tokio::test(start_paused = true)]
async fn an_invalid_effect_is_refused_with_its_errors() {
    let (handle, controls) = demo(config()).await;

    let refused = handle
        .apply_effect(
            whole(GPU),
            effect("rainbow-wave", &[("speed", ControlValue::Number(999))]),
        )
        .await;
    handle.refresh().await.expect("barrier");

    assert_eq!(
        refused,
        Err(OutputError::InvalidEffect(InstanceError::InvalidValues {
            errors: vec![ValueError {
                control: "speed".into(),
                problem: ValueProblem::OutOfRange { min: 0, max: 240 }
            }]
        }))
    );
    assert!(controls.outputs().is_empty());
}

#[tokio::test(start_paused = true)]
async fn stopping_an_effect_restores_the_remembered_mode() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(200)).await;

    let restore = handle.stop_effect(whole(GPU)).await.expect("stop");
    let after = controls.outputs();
    sleep(Duration::from_secs(10)).await;

    assert_eq!(restore, Some(RestoreOutcome::Restored));
    assert_eq!(
        after.last(),
        Some(&DemoOutput::ModeReapplied {
            device: gpu(),
            mode: 2
        })
    );
    assert_eq!(controls.outputs(), after);
    let snapshot = handle.snapshot();
    assert!(output_of(&snapshot, GPU).is_none());
    assert!(snapshot.output.scene.assignments.is_empty());
    assert_eq!(
        handle.stop_effect(whole(GPU)).await,
        Err(OutputError::NoEffect { target: whole(GPU) })
    );
}

#[tokio::test(start_paused = true)]
async fn stopping_one_zone_keeps_the_device_until_its_last_zone_stops() {
    let (handle, controls) = demo(config()).await;
    let zone = |zone| OutputTarget {
        device: DeviceRef::new("demo-motherboard"),
        zone: Some(zone),
    };
    handle
        .apply_effect(zone(0), rainbow())
        .await
        .expect("zone 0");
    handle
        .apply_effect(zone(2), solid(Rgb::new(9, 9, 9)))
        .await
        .expect("zone 2");
    sleep(Duration::from_millis(100)).await;

    assert_eq!(handle.stop_effect(zone(0)).await, Ok(None));
    assert_eq!(handle.snapshot().output.scene.assignments.len(), 1);
    let last = handle.stop_effect(zone(2)).await.expect("stop last zone");

    assert_eq!(last, Some(RestoreOutcome::Restored));
    let outputs = controls.outputs();
    assert!(matches!(
        outputs.last(),
        Some(DemoOutput::DeviceFrame { colors, .. }) if colors.len() == 52
    ));
}

#[tokio::test(start_paused = true)]
async fn selecting_a_hardware_mode_stops_the_effect_and_records_the_owner() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    handle.select_mode(gpu(), 1).await.expect("select static");
    let after = controls.outputs();
    sleep(Duration::from_secs(10)).await;

    assert_eq!(
        after.last(),
        Some(&DemoOutput::ModeReapplied {
            device: gpu(),
            mode: 1
        })
    );
    assert_eq!(controls.outputs(), after);
    let snapshot = handle.snapshot();
    assert_eq!(
        output_of(&snapshot, GPU).expect("gpu output").owner,
        OutputOwner::HardwareMode { mode: 1 }
    );
    assert!(snapshot.output.scene.assignments.is_empty());
    let gpu_mode = snapshot
        .devices
        .iter()
        .find(|device| device.identity.reference() == Some(&gpu()))
        .and_then(|device| device.descriptor.capabilities.active_mode);
    assert_eq!(gpu_mode, Some(1));

    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply over hardware mode");
    sleep(Duration::from_millis(100)).await;
    let restore = handle.stop_effect(whole(GPU)).await.expect("stop");
    assert_eq!(restore, Some(RestoreOutcome::Restored));
    assert_eq!(
        controls.outputs().last(),
        Some(&DemoOutput::ModeReapplied {
            device: gpu(),
            mode: 1
        })
    );
}

#[tokio::test(start_paused = true)]
async fn identify_during_an_effect_pauses_and_resumes_it() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    let running = tokio::spawn(handle.identify(gpu(), None, BLINK));
    let lit = wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    let during = controls.outputs().len();
    sleep(BLINK / 2).await;
    let still = controls.outputs();
    let outcome = running.await.expect("task").expect("identify");
    sleep(Duration::from_millis(200)).await;
    let outputs = controls.outputs();

    assert_eq!(
        output_of(&lit, GPU).expect("gpu output").condition,
        OutputCondition::Paused {
            reason: PauseReason::Identify
        }
    );
    assert_eq!(still.len(), during);
    assert_eq!(
        still.last(),
        Some(&DemoOutput::DeviceFrame {
            device: gpu(),
            colors: vec![IDENTIFY_COLOR; GPU_LEDS]
        })
    );
    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    assert!(
        !outputs
            .iter()
            .any(|output| matches!(output, DemoOutput::ModeReapplied { .. }))
    );
    let resumed = device_frames(&outputs[during..], GPU);
    assert!(resumed.len() >= 5);
    assert!(
        resumed
            .iter()
            .all(|colors| colors != &vec![IDENTIFY_COLOR; GPU_LEDS])
    );
    assert_eq!(
        output_of(&handle.snapshot(), GPU)
            .expect("gpu output")
            .condition,
        OutputCondition::Running
    );

    let restore = handle.stop_effect(whole(GPU)).await.expect("stop");
    assert_eq!(restore, Some(RestoreOutcome::Restored));
    let second = handle.identify(gpu(), None, BLINK).await.expect("identify");
    assert_eq!(second.restore, RestoreOutcome::Restored);
}

#[tokio::test(start_paused = true)]
async fn shutdown_restores_every_device_before_the_session_ends() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let (handle, session) = Session::new(backend, config());
    let session = tokio::spawn(session.run());
    handle.connect().await.expect("connect");
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    drop(handle);
    session.await.expect("session ends");

    assert_eq!(
        controls.outputs().last(),
        Some(&DemoOutput::ModeReapplied {
            device: gpu(),
            mode: 2
        })
    );
}

#[tokio::test(start_paused = true)]
async fn shutdown_does_not_hang_on_a_backend_that_stops_answering() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let (handle, session) = Session::new(backend, config());
    let session = tokio::spawn(session.run());
    handle.connect().await.expect("connect");
    handle
        .apply_effect(whole(GPU), solid(Rgb::new(1, 2, 3)))
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;
    controls.set_call_delay(Duration::from_secs(3_600));
    sleep(Duration::from_millis(100)).await;

    let asked = Instant::now();
    drop(handle);
    session.await.expect("session ends");

    assert!(
        asked.elapsed() <= Duration::from_secs(5),
        "{:?}",
        asked.elapsed()
    );
}

#[tokio::test(start_paused = true)]
async fn an_ambiguous_device_gets_no_output() {
    let twin = |label: &str| FixtureDevice::new(label, "Twin Fan").zone("Ring", 4);
    let server = FakeServer::new();
    server.add_device(0, twin("left"));
    let handle = support::connected_openrgb(&server).await;
    let left = handle.snapshot().devices[0]
        .identity
        .reference()
        .cloned()
        .expect("single twin resolves");
    let target = OutputTarget {
        device: left.clone(),
        zone: None,
    };
    handle.apply_effect(target, rainbow()).await.expect("apply");
    server.wait_for_color_writes(1).await;

    server.restart(vec![(0, twin("left").into()), (1, twin("right").into())]);
    wait_until(&handle, is_retrying).await;
    let degraded = wait_until(&handle, support::is_live).await;
    let writes_after_restart = || {
        server
            .color_writes()
            .into_iter()
            .filter(|write| write.session == 1)
            .count()
    };
    sleep(Duration::from_secs(5)).await;

    assert_eq!(writes_after_restart(), 0);
    let condition = output_of(&handle.snapshot(), left.as_str())
        .expect("left output")
        .condition;
    assert_eq!(
        condition,
        OutputCondition::Paused {
            reason: PauseReason::Unavailable {
                error: BackendError::AmbiguousDevice { device: left }
            }
        }
    );
    assert!(!support::is_connected(&degraded));
}

#[tokio::test(start_paused = true)]
async fn effect_resumes_on_the_same_device_after_a_restart_with_reused_ids() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 2));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    handle
        .apply_effect(
            OutputTarget {
                device: alpha.clone(),
                zone: None,
            },
            rainbow(),
        )
        .await
        .expect("apply");
    sleep(Duration::from_millis(200)).await;

    server.restart(vec![
        (0, strip("beta", 2).into()),
        (1, strip("alpha", 3).into()),
        (2, strip("gamma", 4).into()),
    ]);
    wait_until(&handle, is_retrying).await;
    wait_until(&handle, is_connected).await;
    sleep(Duration::from_millis(500)).await;

    let packets = output_packets(&server);
    assert!(
        packets
            .iter()
            .all(|packet| packet.target.as_deref() == Some("alpha")),
        "{packets:?}"
    );
    let restarted: Vec<(u32, u32)> = packets
        .iter()
        .filter(|packet| packet.session == 1)
        .map(|packet| (packet.device_id, packet.packet_id))
        .collect();
    assert!(restarted.len() > 5);
    assert_eq!(restarted[0], (1, wire::SET_CUSTOM_MODE));
    assert!(
        restarted[1..]
            .iter()
            .all(|&packet| packet == (1, wire::UPDATE_LEDS))
    );
}

fn motherboard_zone(zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(MOTHERBOARD),
        zone: Some(zone),
    }
}

fn colours(snapshot: &prismforge_core::session::SessionSnapshot, device: &str) -> Vec<Rgb> {
    snapshot
        .devices
        .iter()
        .find(|entry| entry.identity.reference().map(DeviceRef::as_str) == Some(device))
        .map(|entry| entry.descriptor.colors.clone())
        .expect("device listed")
}

#[tokio::test(start_paused = true)]
async fn identify_of_a_whole_device_puts_back_the_leds_no_effect_covers() {
    let (handle, _controls) = demo(config()).await;
    let before = colours(&handle.snapshot(), MOTHERBOARD);
    handle
        .apply_effect(motherboard_zone(1), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    let outcome = handle
        .identify(DeviceRef::new(MOTHERBOARD), None, BLINK)
        .await
        .expect("identify");
    sleep(Duration::from_millis(100)).await;
    handle.refresh().await.expect("refresh");
    let after = colours(&handle.snapshot(), MOTHERBOARD);

    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    assert_eq!(after[..6], before[..6]);
    assert_eq!(after[36..], before[36..]);
    assert!(after[6..36].iter().any(|colour| *colour != IDENTIFY_COLOR));
}

#[tokio::test(start_paused = true)]
async fn identify_of_another_zone_puts_back_that_zone() {
    let (handle, _controls) = demo(config()).await;
    let before = colours(&handle.snapshot(), MOTHERBOARD);
    handle
        .apply_effect(motherboard_zone(1), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    let outcome = handle
        .identify(DeviceRef::new(MOTHERBOARD), Some(2), BLINK)
        .await
        .expect("identify");
    sleep(Duration::from_millis(100)).await;
    handle.refresh().await.expect("refresh");
    let after = colours(&handle.snapshot(), MOTHERBOARD);

    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    assert_eq!(after[36..], before[36..]);
}

#[tokio::test(start_paused = true)]
async fn the_original_mode_survives_an_external_declaration_and_hand_back() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;
    handle.refresh().await.expect("refresh");
    handle
        .declare_external(gpu(), PLUGIN.into())
        .await
        .expect("declare");
    handle.clear_external(gpu()).await.expect("hand back");
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply again");
    sleep(Duration::from_millis(100)).await;

    let restore = handle.stop_effect(whole(GPU)).await.expect("stop");

    assert_eq!(restore, Some(RestoreOutcome::Restored));
    assert_eq!(
        controls.outputs().last(),
        Some(&DemoOutput::ModeReapplied {
            device: gpu(),
            mode: 2
        })
    );
}

#[tokio::test(start_paused = true)]
async fn devices_the_shutdown_restore_did_not_reach_report_an_unknown_previous_state() {
    let (handle, controls) = demo(config()).await;
    for device in [GPU, MEMORY] {
        handle
            .apply_effect(whole(device), solid(Rgb::new(1, 2, 3)))
            .await
            .expect("apply");
    }
    sleep(Duration::from_millis(100)).await;
    controls.set_call_delay(Duration::from_secs(3_600));

    handle.disconnect().await.expect("disconnect");
    controls.set_call_delay(Duration::ZERO);
    handle.connect().await.expect("reconnect");
    let unknown = RestoreOutcome::PartlyRestored {
        failure: prismforge_core::session::RestoreFailure::PreviousStateUnknown,
    };

    for device in [GPU, MEMORY] {
        let outcome = handle
            .identify(DeviceRef::new(device), None, BLINK)
            .await
            .expect("identify");
        assert_eq!(outcome.restore, unknown, "{device}");
    }
}

#[tokio::test(start_paused = true)]
async fn no_command_writes_to_a_device_declared_external() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let (handle, session) = Session::new(backend, config());
    let session = tokio::spawn(session.run());
    handle.connect().await.expect("connect");
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;
    handle
        .declare_external(gpu(), PLUGIN.into())
        .await
        .expect("declare");
    let declared = controls.outputs().len();
    let conflict = SessionError::OutputConflict {
        device: gpu(),
        owner: OutputOwner::External {
            label: PLUGIN.into(),
        },
    };
    let gpu_zone = OutputTarget {
        device: gpu(),
        zone: Some(0),
    };

    assert_eq!(
        handle.identify(gpu(), None, BLINK).await,
        Err(conflict.clone())
    );
    assert_eq!(
        handle.identify(gpu(), Some(0), BLINK).await,
        Err(conflict.clone())
    );
    assert_eq!(
        handle.apply_effect(whole(GPU), rainbow()).await,
        Err(OutputError::Session(conflict.clone()))
    );
    assert_eq!(
        handle.apply_effect(gpu_zone.clone(), rainbow()).await,
        Err(OutputError::Session(conflict.clone()))
    );
    assert_eq!(
        handle.select_mode(gpu(), 1).await,
        Err(OutputError::Session(conflict))
    );
    assert_eq!(
        handle.stop_effect(whole(GPU)).await,
        Err(OutputError::NoEffect { target: whole(GPU) })
    );
    assert_eq!(
        handle.stop_effect(gpu_zone.clone()).await,
        Err(OutputError::NoEffect { target: gpu_zone })
    );
    controls.lose_connection(prismforge_core::LossCause::Timeout);
    wait_until(&handle, is_retrying).await;
    wait_until(&handle, is_connected).await;
    sleep(Duration::from_secs(5)).await;
    handle.refresh().await.expect("refresh");
    handle.disconnect().await.expect("disconnect");
    handle.connect().await.expect("connect again");
    sleep(Duration::from_secs(1)).await;
    drop(handle);
    session.await.expect("session ends");

    assert_eq!(controls.outputs()[declared..], []);
}
