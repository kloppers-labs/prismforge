use std::time::Duration;

use prismforge_core::{
    BackendError, BackendEvent, BackendInfo, BackendOrigin, ConnectionState, DeviceKind, DeviceRef,
    LossCause, Rgb, RgbBackend, StateReason,
};
use prismforge_testkit::{DemoBackend, DemoOutput};
use tokio::time::Instant;

const GPU: &str = "demo-graphics-card";
const LIGHT_BAR: &str = "demo-light-bar";

async fn connected() -> DemoBackend {
    let mut backend = DemoBackend::new();
    backend.connect().await.expect("connect");
    backend.refresh_inventory().await.expect("inventory");
    backend
}

fn device(name: &str) -> DeviceRef {
    DeviceRef::new(name)
}

#[tokio::test]
async fn every_demo_device_is_marked_simulated() {
    let backend = connected().await;
    let inventory = backend.inventory().expect("inventory");

    assert!(
        inventory
            .devices
            .iter()
            .all(|device| device.descriptor.origin == BackendOrigin::Simulated)
    );
    assert_eq!(
        backend.info(),
        BackendInfo {
            origin: BackendOrigin::Simulated,
            address: None,
            protocol_version: None,
            server_name: None,
        }
    );
}

#[tokio::test]
async fn demo_inventory_is_a_plausible_generic_pc() {
    let backend = connected().await;
    let devices = &backend.inventory().expect("inventory").devices;
    let kinds: Vec<DeviceKind> = devices.iter().map(|d| d.descriptor.kind).collect();

    assert_eq!(
        kinds,
        [
            DeviceKind::Memory,
            DeviceKind::Memory,
            DeviceKind::Motherboard,
            DeviceKind::Gpu,
            DeviceKind::Light,
        ]
    );
    let motherboard = &devices[2].descriptor;
    assert_eq!(motherboard.capabilities.resizable_zones.len(), 1);
    assert!(motherboard.zones.len() >= 3);
    let without_direct: Vec<&str> = devices
        .iter()
        .filter(|d| !d.descriptor.capabilities.has_direct_mode())
        .map(|d| d.descriptor.name.as_str())
        .collect();
    assert_eq!(without_direct, ["Demo Light Bar"]);
    assert!(devices.iter().all(|d| d.identity.reference().is_some()));
}

#[tokio::test]
async fn demo_inventory_is_deterministic() {
    let first = connected().await;
    let second = connected().await;

    assert_eq!(first.inventory(), second.inventory());
}

#[tokio::test]
async fn records_outputs_and_reports_them_on_refresh() {
    let mut backend = connected().await;
    let controls = backend.controls();
    let gpu = device(GPU);
    let green = vec![Rgb::new(0, 255, 0); 12];

    backend.enable_direct_mode(&gpu).await.expect("direct");
    backend
        .write_device_frame(&gpu, &green)
        .await
        .expect("frame");
    backend
        .write_zone_frame(&gpu, 0, &green)
        .await
        .expect("zone frame");
    backend.reapply_mode(&gpu, 2).await.expect("re-apply");

    assert_eq!(
        controls.outputs(),
        [
            DemoOutput::DirectMode {
                device: gpu.clone()
            },
            DemoOutput::DeviceFrame {
                device: gpu.clone(),
                colors: green.clone()
            },
            DemoOutput::ZoneFrame {
                device: gpu.clone(),
                zone: 0,
                colors: green.clone()
            },
            DemoOutput::ModeReapplied {
                device: gpu.clone(),
                mode: 2
            },
        ]
    );
    let inventory = backend.refresh_inventory().await.expect("refresh");
    let descriptor = &inventory.device(&gpu).expect("gpu").descriptor;
    assert_eq!(descriptor.colors, green);
    assert_eq!(descriptor.capabilities.active_mode, Some(2));
}

#[tokio::test]
async fn refuses_invalid_output_like_a_real_backend() {
    let mut backend = connected().await;
    let controls = backend.controls();
    let gpu = device(GPU);
    let bar = device(LIGHT_BAR);

    assert_eq!(
        backend.enable_direct_mode(&bar).await,
        Err(BackendError::DirectModeUnavailable {
            device: bar.clone()
        })
    );
    assert_eq!(
        backend
            .write_zone_frame(&gpu, 1, &[Rgb::default(); 12])
            .await,
        Err(BackendError::ZoneOutOfRange {
            device: gpu.clone(),
            zone: 1,
            zone_count: 1
        })
    );
    assert_eq!(
        backend.write_device_frame(&gpu, &[Rgb::default(); 3]).await,
        Err(BackendError::ColorCountMismatch {
            device: gpu.clone(),
            expected: 12,
            actual: 3
        })
    );
    assert_eq!(
        backend.reapply_mode(&gpu, 3).await,
        Err(BackendError::ModeOutOfRange {
            device: gpu.clone(),
            mode: 3,
            mode_count: 3
        })
    );
    assert_eq!(
        backend.reapply_mode(&device("demo-unknown"), 0).await,
        Err(BackendError::UnknownDevice {
            device: device("demo-unknown")
        })
    );
    assert!(controls.outputs().is_empty());
}

#[tokio::test]
async fn failing_connect_reports_unreachable_without_devices() {
    let mut backend = DemoBackend::new();
    backend.controls().fail_connect(true);

    let error = backend.connect().await.unwrap_err();

    assert!(
        matches!(error, BackendError::Unreachable { .. }),
        "{error:?}"
    );
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Disconnected {
            reason: Some(StateReason::ServerUnreachable)
        }
    );
    assert_eq!(
        backend.refresh_inventory().await.map(|_| ()),
        Err(BackendError::NotConnected)
    );
    assert!(backend.inventory().is_none());
}

#[tokio::test]
async fn lost_connection_is_reported_with_its_cause() {
    let mut backend = connected().await;
    backend.controls().lose_connection(LossCause::Timeout);

    let event = backend.next_event().await;

    assert!(
        matches!(
            event,
            Some(BackendEvent::ConnectionLost {
                cause: LossCause::Timeout,
                ..
            })
        ),
        "{event:?}"
    );
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Disconnected {
            reason: Some(StateReason::ConnectionLost {
                cause: LossCause::Timeout
            })
        }
    );
    assert_eq!(backend.next_event().await, None);
    assert_eq!(
        backend
            .write_device_frame(&device(GPU), &[Rgb::default(); 12])
            .await,
        Err(BackendError::NotConnected)
    );
}

#[tokio::test]
async fn inventory_change_makes_output_wait_for_a_refresh() {
    let mut backend = connected().await;
    backend.controls().change_inventory();

    assert_eq!(
        backend.next_event().await,
        Some(BackendEvent::InventoryChanged)
    );
    assert_eq!(
        backend.enable_direct_mode(&device(GPU)).await,
        Err(BackendError::StaleInventory)
    );
    backend.refresh_inventory().await.expect("refresh");
    backend
        .enable_direct_mode(&device(GPU))
        .await
        .expect("after refresh");
}

#[tokio::test(start_paused = true)]
async fn slow_calls_take_the_configured_delay() {
    let mut backend = DemoBackend::new();
    backend.controls().set_call_delay(Duration::from_secs(3));
    let started = Instant::now();

    backend.connect().await.expect("connect");
    backend.refresh_inventory().await.expect("inventory");

    assert_eq!(started.elapsed(), Duration::from_secs(6));
}
