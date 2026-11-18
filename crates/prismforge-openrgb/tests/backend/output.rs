use prismforge_core::{
    BackendError, MatrixSize, ModeDirection, ModePersistence, RgbBackend, ValueRange,
    ZoneResizeBounds,
};
use prismforge_testkit::{FakeServer, FixtureDevice, FixtureMode, wire};

use crate::support::{self, GREEN, RED, solid, strip};

#[tokio::test]
async fn refuses_colour_count_mismatch() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 4));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");

    assert_eq!(
        backend.write_device_frame(&alpha, &solid(RED, 3)).await,
        Err(BackendError::ColorCountMismatch {
            device: alpha.clone(),
            expected: 4,
            actual: 3
        })
    );
    assert_eq!(
        backend.write_zone_frame(&alpha, 0, &solid(RED, 5)).await,
        Err(BackendError::ColorCountMismatch {
            device: alpha.clone(),
            expected: 4,
            actual: 5
        })
    );
    assert_eq!(
        backend.write_zone_frame(&alpha, 1, &solid(RED, 4)).await,
        Err(BackendError::ZoneOutOfRange {
            device: alpha.clone(),
            zone: 1,
            zone_count: 1
        })
    );

    backend.refresh_inventory().await.expect("barrier");
    assert!(server.color_writes().is_empty());
}

#[tokio::test]
async fn refuses_device_without_direct_mode() {
    let server = FakeServer::new();
    server.add_device(0, strip("plain", 3).without_direct_mode());
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let plain = support::reference_by_serial(inventory, "SN-plain");

    assert_eq!(
        backend
            .capabilities(&plain)
            .expect("capabilities")
            .direct_mode,
        None
    );
    let refused = BackendError::DirectModeUnavailable {
        device: plain.clone(),
    };
    assert_eq!(
        backend.write_device_frame(&plain, &solid(RED, 3)).await,
        Err(refused.clone())
    );
    assert_eq!(
        backend.write_zone_frame(&plain, 0, &solid(RED, 3)).await,
        Err(refused.clone())
    );
    assert_eq!(backend.enable_direct_mode(&plain).await, Err(refused));

    backend.refresh_inventory().await.expect("barrier");
    assert!(server.color_writes().is_empty());
    assert!(server.received_with_id(wire::SET_CUSTOM_MODE).is_empty());
}

#[tokio::test]
async fn writes_zone_frame_to_its_zone() {
    let server = FakeServer::new();
    server.add_device(7, strip("alpha", 3).zone("Fan", 2));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");

    backend
        .write_zone_frame(&alpha, 1, &solid(GREEN, 2))
        .await
        .expect("zone frame");

    let writes = server.wait_for_color_writes(1).await;
    assert_eq!(writes[0].device_id, 7);
    assert_eq!(writes[0].zone, Some(1));
    assert_eq!(writes[0].colors, support::wire(&solid(GREEN, 2)));
}

#[tokio::test]
async fn enables_direct_mode_explicitly() {
    let server = FakeServer::new();
    server.add_device(4, strip("alpha", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");
    assert!(server.received_with_id(wire::SET_CUSTOM_MODE).is_empty());

    backend
        .enable_direct_mode(&alpha)
        .await
        .expect("direct mode");

    let requests = server.received_with_id(wire::SET_CUSTOM_MODE);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].device_id, 4);
    assert!(requests[0].payload.is_empty());
}

#[tokio::test]
async fn reports_modes_and_capabilities() {
    let server = FakeServer::new();
    let device = FixtureDevice::new("kbd", "Keyboard K1")
        .serial("SN-K1")
        .matrix_zone("Keys", 2, 4)
        .resizable_zone("Header", 0, 60, 4)
        .modes(vec![
            FixtureMode::direct(),
            FixtureMode::static_color(),
            FixtureMode::breathing(),
        ]);
    server.add_device(0, device);
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let keyboard = support::reference_by_serial(inventory, "SN-K1");
    let descriptor = &inventory.device(&keyboard).expect("device").descriptor;
    assert_eq!(descriptor.led_count, 12);
    assert_eq!(
        descriptor.zones[0].matrix,
        Some(MatrixSize {
            width: 4,
            height: 2
        })
    );

    let capabilities = backend.capabilities(&keyboard).expect("capabilities");

    assert_eq!(capabilities.direct_mode, Some(0));
    assert!(capabilities.saves_to_device_memory);
    assert_eq!(capabilities.active_mode, Some(0));
    assert_eq!(
        capabilities.resizable_zones,
        [ZoneResizeBounds {
            zone: 1,
            min: 0,
            max: 60
        }]
    );
    let [direct, fixed, breathing] = capabilities.modes.as_slice() else {
        panic!("expected three modes");
    };
    assert_eq!(direct.name, "Direct");
    assert!(direct.per_led_color);
    assert_eq!(direct.persistence, ModePersistence::None);
    assert_eq!(fixed.mode_colors, Some(ValueRange { min: 1, max: 1 }));
    assert_eq!(fixed.persistence, ModePersistence::ManualSave);
    assert_eq!(breathing.speed, Some(ValueRange { min: 1, max: 5 }));
    assert_eq!(breathing.brightness, Some(ValueRange { min: 0, max: 100 }));
    assert_eq!(breathing.directions, [ModeDirection::LeftRight]);
    assert!(!breathing.per_led_color);
}
