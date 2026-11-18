use std::time::Duration;

use prismforge_core::session::{
    IDENTIFY_COLOR, IdentifyTarget, RestoreFailure, RestoreOutcome, SessionError,
};
use prismforge_core::{BackendError, DeviceRef, LossCause, StateReason};
use prismforge_testkit::{DemoBackend, DemoOutput, FakeServer, FixtureDevice, FixtureMode, wire};

use crate::support::{
    self, config, is_connected, is_live, output_packets, reference, start, strip, wait_until,
};

const BLINK: Duration = Duration::from_secs(2);

fn white(count: usize) -> Vec<[u8; 3]> {
    vec![[IDENTIFY_COLOR.r, IDENTIFY_COLOR.g, IDENTIFY_COLOR.b]; count]
}

#[tokio::test(start_paused = true)]
async fn identify_validates_before_any_write() {
    let twin = |label: &str| FixtureDevice::new(label, "Twin Stick").zone("Stick", 2);
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("plain", 3).without_direct_mode());
    server.add_device(2, twin("left"));
    let handle = support::connected_openrgb(&server).await;
    let snapshot = handle.snapshot();
    let alpha = reference(&snapshot, "SN-alpha");
    let plain = reference(&snapshot, "SN-plain");
    let left = snapshot.devices[2]
        .identity
        .reference()
        .cloned()
        .expect("single twin resolves");
    server.restart(vec![
        (0, strip("alpha", 3).into()),
        (1, strip("plain", 3).without_direct_mode().into()),
        (2, twin("left").into()),
        (3, twin("right").into()),
    ]);
    wait_until(&handle, support::is_retrying).await;
    wait_until(&handle, is_live).await;

    let results = [
        handle.identify(plain.clone(), None, BLINK).await,
        handle.identify(left.clone(), None, BLINK).await,
        handle.identify(alpha.clone(), Some(1), BLINK).await,
        handle.identify(alpha.clone(), None, Duration::ZERO).await,
    ];

    assert_eq!(
        results,
        [
            Err(SessionError::Backend(BackendError::DirectModeUnavailable {
                device: plain
            })),
            Err(SessionError::Backend(BackendError::AmbiguousDevice {
                device: left
            })),
            Err(SessionError::Backend(BackendError::ZoneOutOfRange {
                device: alpha,
                zone: 1,
                zone_count: 1
            })),
            Err(SessionError::InvalidDuration { max_ms: 30_000 }),
        ]
    );
    handle.refresh().await.expect("barrier");
    assert!(output_packets(&server).is_empty());
}

#[tokio::test(start_paused = true)]
async fn identify_lights_only_the_selected_zone() {
    let server = FakeServer::new();
    server.add_device(4, strip("alpha", 3).zone("Fan", 2));
    server.add_device(5, strip("beta", 2));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    let running = tokio::spawn(handle.identify(alpha.clone(), Some(1), BLINK));
    let lit = wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    let writes = server.wait_for_color_writes(1).await;

    assert_eq!(
        lit.identify,
        Some(IdentifyTarget {
            device: alpha.clone(),
            zone: Some(1)
        })
    );
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].target.as_deref(), Some("alpha"));
    assert_eq!(writes[0].zone, Some(1));
    assert_eq!(writes[0].colors, white(2));
    let outcome = running.await.expect("task").expect("identify");
    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    assert_eq!(outcome.light_failure, None);
    assert_eq!(handle.snapshot().identify, None);
    server.wait_for_color_writes(2).await;
    assert!(
        output_packets(&server)
            .iter()
            .all(|packet| packet.target.as_deref() == Some("alpha"))
    );
}

#[tokio::test(start_paused = true)]
async fn identify_restores_the_previous_hardware_mode() {
    let server = FakeServer::new();
    let alpha_fixture = strip("alpha", 3)
        .modes(vec![
            FixtureMode::direct(),
            FixtureMode::static_color(),
            FixtureMode::breathing(),
        ])
        .active_mode(2);
    server.add_device(0, alpha_fixture);
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    let outcome = handle
        .identify(alpha.clone(), None, BLINK)
        .await
        .expect("identify");

    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    let updates = server.mode_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].target.as_deref(), Some("alpha"));
    assert_eq!(updates[0].mode_index, 2);
    assert_eq!(updates[0].mode, FixtureMode::breathing());
    assert!(server.received_with_id(wire::SAVE_MODE).is_empty());
    let writes = server.color_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].colors, white(3));
}

#[tokio::test(start_paused = true)]
async fn identify_restores_the_previous_colours_in_direct_mode() {
    let previous = vec![[10, 20, 30], [40, 50, 60], [70, 80, 90]];
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3).led_colors(previous.clone()));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    let outcome = handle
        .identify(alpha.clone(), None, BLINK)
        .await
        .expect("identify");

    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    let writes = server.wait_for_color_writes(2).await;
    assert_eq!(writes[0].colors, white(3));
    assert_eq!(writes[1].colors, previous);
    assert!(server.mode_updates().is_empty());
}

#[tokio::test(start_paused = true)]
async fn identify_reports_not_restored_when_the_connection_drops() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    let running = tokio::spawn(handle.identify(alpha.clone(), None, BLINK));
    wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    server.wait_for_color_writes(1).await;
    server.disconnect_all();
    let outcome = running.await.expect("task").expect("identify");
    wait_until(&handle, is_connected).await;
    tokio::time::sleep(BLINK * 10).await;

    assert_eq!(
        outcome.restore,
        RestoreOutcome::NotRestored {
            failure: RestoreFailure::ConnectionLost {
                reason: StateReason::ConnectionLost {
                    cause: LossCause::ServerClosed
                }
            }
        }
    );
    let packets = output_packets(&server);
    assert_eq!(packets.len(), 2, "{packets:?}");
    assert_eq!(packets[0].packet_id, wire::SET_CUSTOM_MODE);
    assert_eq!(packets[1].packet_id, wire::UPDATE_LEDS);
}

#[tokio::test(start_paused = true)]
async fn identify_never_fires_without_the_explicit_command() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");

    controls.change_inventory();
    tokio::time::sleep(Duration::from_secs(1)).await;
    controls.lose_connection(LossCause::Timeout);
    wait_until(&handle, support::is_retrying).await;
    wait_until(&handle, is_connected).await;
    handle.refresh().await.expect("refresh");
    tokio::time::sleep(Duration::from_secs(60)).await;

    assert!(controls.outputs().is_empty(), "{:?}", controls.outputs());
    assert!(handle.snapshot().diagnostics.counters.inventory_refreshes >= 3);
}

#[tokio::test(start_paused = true)]
async fn second_identify_is_refused_while_one_runs() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");
    let gpu = DeviceRef::new("demo-graphics-card");
    let memory = DeviceRef::new("demo-memory-1");

    let running = tokio::spawn(handle.identify(gpu.clone(), None, BLINK));
    wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    let same = handle.identify(gpu.clone(), None, BLINK).await;
    let other = handle.identify(memory.clone(), None, BLINK).await;
    let outputs_while_running = controls.outputs().len();
    let first = running.await.expect("task").expect("first identify");
    let later = handle.identify(memory.clone(), None, BLINK).await;

    let refused = Err(SessionError::IdentifyInProgress {
        device: gpu.clone(),
    });
    assert_eq!(same, refused);
    assert_eq!(other, refused);
    assert_eq!(outputs_while_running, 2);
    assert_eq!(first.restore, RestoreOutcome::Restored);
    assert_eq!(
        later.expect("later identify").restore,
        RestoreOutcome::Restored
    );
}

#[tokio::test(start_paused = true)]
async fn identify_finishes_even_when_the_caller_stops_waiting() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");
    let gpu = DeviceRef::new("demo-graphics-card");

    drop(handle.identify(gpu.clone(), None, BLINK));
    wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    wait_until(&handle, |snapshot| snapshot.identify.is_none()).await;

    assert_eq!(
        controls.outputs(),
        [
            DemoOutput::DirectMode {
                device: gpu.clone()
            },
            DemoOutput::DeviceFrame {
                device: gpu.clone(),
                colors: vec![IDENTIFY_COLOR; 12]
            },
            DemoOutput::ModeReapplied {
                device: gpu.clone(),
                mode: 2
            },
        ]
    );
}

fn breathing_strip(label: &str) -> FixtureDevice {
    strip(label, 3)
        .modes(vec![
            FixtureMode::direct(),
            FixtureMode::static_color(),
            FixtureMode::breathing(),
        ])
        .active_mode(2)
}

#[tokio::test(start_paused = true)]
async fn inventory_change_while_lighting_is_refreshed_then_restored() {
    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha"));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    server.delay_replies(wire::SET_CUSTOM_MODE, Duration::from_millis(100));

    let running = tokio::spawn(handle.identify(alpha.clone(), None, BLINK));
    tokio::time::sleep(Duration::from_millis(10)).await;
    server.notify_device_list_updated();
    let outcome = running.await.expect("task").expect("identify");
    tokio::time::sleep(Duration::from_secs(60)).await;
    let again = handle
        .identify(alpha.clone(), None, BLINK)
        .await
        .expect("second identify");

    assert_eq!(
        outcome.light_failure,
        Some(BackendError::StaleInventory),
        "{outcome:?}"
    );
    assert_eq!(outcome.restore, RestoreOutcome::Restored);
    let updates = server.mode_updates();
    assert_eq!(updates[0].target.as_deref(), Some("alpha"));
    assert_eq!(updates[0].mode_index, 2);
    assert_eq!(updates[0].mode, FixtureMode::breathing());
    assert_eq!(again.light_failure, None);
    assert_eq!(again.restore, RestoreOutcome::Restored);
    assert_eq!(
        handle.snapshot().connection,
        prismforge_core::ConnectionState::Connected
    );
}

#[tokio::test(start_paused = true)]
async fn restore_after_a_rescan_never_writes_to_an_ambiguous_device() {
    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha"));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    server.delay_replies(wire::SET_CUSTOM_MODE, Duration::from_millis(100));

    let running = tokio::spawn(handle.identify(alpha.clone(), None, BLINK));
    tokio::time::sleep(Duration::from_millis(10)).await;
    server.add_device(1, breathing_strip("alpha"));
    server.notify_device_list_updated();
    let outcome = running.await.expect("task").expect("identify");

    assert_eq!(
        outcome.restore,
        RestoreOutcome::NotRestored {
            failure: RestoreFailure::Refused {
                error: BackendError::AmbiguousDevice {
                    device: alpha.clone()
                }
            }
        }
    );
    let packets: Vec<u32> = output_packets(&server)
        .iter()
        .map(|packet| packet.packet_id)
        .collect();
    assert_eq!(packets, [wire::SET_CUSTOM_MODE]);
}

#[tokio::test(start_paused = true)]
async fn refused_lighting_changes_nothing_and_says_so() {
    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha"));
    server.reject(wire::SET_CUSTOM_MODE);
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    let outcome = handle
        .identify(alpha.clone(), None, BLINK)
        .await
        .expect("identify");

    assert!(
        matches!(outcome.light_failure, Some(BackendError::Rejected { .. })),
        "{outcome:?}"
    );
    assert_eq!(outcome.restore, RestoreOutcome::Unchanged);
    handle.refresh().await.expect("barrier");
    let packets: Vec<u32> = output_packets(&server)
        .iter()
        .map(|packet| packet.packet_id)
        .collect();
    assert_eq!(packets, [wire::SET_CUSTOM_MODE]);
}

#[tokio::test(start_paused = true)]
async fn refresh_during_identify_does_not_poison_the_next_identify() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");
    let gpu = DeviceRef::new("demo-graphics-card");
    let gpu_mode = |handle: &prismforge_core::session::SessionHandle| {
        handle
            .snapshot()
            .devices
            .into_iter()
            .find(|device| device.identity.reference() == Some(&gpu))
            .and_then(|device| device.descriptor.capabilities.active_mode)
    };

    let running = tokio::spawn(handle.identify(gpu.clone(), None, BLINK));
    wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    handle.refresh().await.expect("refresh while lit");
    let first = running.await.expect("task").expect("first identify");
    let after_first = gpu_mode(&handle);
    let outputs_before_second = controls.outputs().len();
    let second = handle
        .identify(gpu.clone(), None, BLINK)
        .await
        .expect("second identify");

    assert_eq!(first.restore, RestoreOutcome::Restored);
    assert_eq!(after_first, Some(2));
    assert_eq!(second.restore, RestoreOutcome::Restored);
    assert_eq!(
        controls.outputs()[outputs_before_second..],
        [
            DemoOutput::DirectMode {
                device: gpu.clone()
            },
            DemoOutput::DeviceFrame {
                device: gpu.clone(),
                colors: vec![IDENTIFY_COLOR; 12]
            },
            DemoOutput::ModeReapplied {
                device: gpu.clone(),
                mode: 2
            },
        ]
    );
    assert_eq!(gpu_mode(&handle), Some(2));
}

#[tokio::test(start_paused = true)]
async fn identify_after_an_unrestored_identify_says_the_previous_state_is_unknown() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");
    let gpu = DeviceRef::new("demo-graphics-card");

    let running = tokio::spawn(handle.identify(gpu.clone(), None, BLINK));
    wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    controls.lose_connection(LossCause::Timeout);
    let interrupted = running.await.expect("task").expect("first identify");
    wait_until(&handle, is_connected).await;
    let second = handle
        .identify(gpu.clone(), None, BLINK)
        .await
        .expect("second identify");

    assert!(
        matches!(interrupted.restore, RestoreOutcome::NotRestored { .. }),
        "{interrupted:?}"
    );
    assert_eq!(
        second.restore,
        RestoreOutcome::PartlyRestored {
            failure: RestoreFailure::PreviousStateUnknown
        }
    );
}

#[tokio::test(start_paused = true)]
async fn unknown_previous_mode_is_not_reported_as_restored() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let gpu = DeviceRef::new("demo-graphics-card");
    controls.hide_active_mode(&gpu);
    let handle = start(backend, config());
    handle.connect().await.expect("connect");

    let outcome = handle
        .identify(gpu.clone(), None, BLINK)
        .await
        .expect("identify");

    assert_eq!(
        outcome.restore,
        RestoreOutcome::PartlyRestored {
            failure: RestoreFailure::PreviousStateUnknown
        }
    );
}
