use prismforge_core::{
    BackendError, BackendEvent, ConnectionState, DeviceIdentity, RgbBackend, StateReason,
};
use prismforge_testkit::{FakeServer, FixtureDevice, wire};

use crate::support::{self, BLUE, GREEN, RED, solid, strip};

fn twin(label: &str) -> FixtureDevice {
    FixtureDevice::new(label, "Twin Stick").zone("Stick", 2)
}

fn output_packets_in_session(server: &FakeServer, session: u32) -> usize {
    server
        .received()
        .iter()
        .filter(|packet| packet.session == session)
        .filter(|packet| {
            [
                wire::UPDATE_LEDS,
                wire::UPDATE_ZONE_LEDS,
                wire::SET_CUSTOM_MODE,
            ]
            .contains(&packet.packet_id)
        })
        .count()
}

#[tokio::test]
async fn reconciles_changed_runtime_ids() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");
    backend
        .write_device_frame(&alpha, &solid(RED, 3))
        .await
        .expect("first frame");
    server.wait_for_color_writes(1).await;

    server.restart(vec![
        (0, strip("beta", 3).into()),
        (1, strip("alpha", 3).into()),
    ]);
    assert!(matches!(
        backend.next_event().await,
        Some(BackendEvent::ConnectionLost { .. })
    ));
    assert_eq!(
        backend.write_device_frame(&alpha, &solid(GREEN, 3)).await,
        Err(BackendError::NotConnected)
    );

    backend.connect().await.expect("reconnect");
    assert_eq!(
        backend.write_device_frame(&alpha, &solid(GREEN, 3)).await,
        Err(BackendError::StaleInventory)
    );

    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha_entry = inventory.device(&alpha).expect("alpha keeps its reference");
    assert_eq!(alpha_entry.identity, DeviceIdentity::Matched(alpha.clone()));
    assert_eq!(alpha_entry.descriptor.serial, "SN-alpha");
    backend
        .write_device_frame(&alpha, &solid(BLUE, 3))
        .await
        .expect("frame after refresh");

    let writes = server.wait_for_color_writes(2).await;
    assert_eq!(writes.len(), 2);
    assert!(
        writes
            .iter()
            .all(|write| write.target.as_deref() == Some("alpha"))
    );
    assert_eq!((writes[0].session, writes[0].device_id), (0, 0));
    assert_eq!((writes[1].session, writes[1].device_id), (1, 1));
    assert_eq!(writes[0].colors, support::wire(&solid(RED, 3)));
    assert_eq!(writes[1].colors, support::wire(&solid(BLUE, 3)));
}

#[tokio::test]
async fn reports_ambiguous_identity() {
    let server = FakeServer::new();
    server.add_device(0, twin("left"));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let left = inventory.devices[0]
        .identity
        .reference()
        .cloned()
        .expect("a single device resolves");

    server.restart(vec![(0, twin("left").into()), (1, twin("right").into())]);
    backend.next_event().await;
    backend.connect().await.expect("reconnect");
    let inventory = backend.refresh_inventory().await.expect("inventory");

    assert_eq!(inventory.devices.len(), 2);
    for device in &inventory.devices {
        assert_eq!(
            device.identity,
            DeviceIdentity::Ambiguous {
                candidates: vec![left.clone()]
            }
        );
    }
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Degraded {
            reasons: vec![StateReason::AmbiguousDevices { devices: 2 }]
        }
    );
    let refused = BackendError::AmbiguousDevice {
        device: left.clone(),
    };
    assert_eq!(
        backend.write_device_frame(&left, &solid(RED, 2)).await,
        Err(refused.clone())
    );
    assert_eq!(
        backend.write_zone_frame(&left, 0, &solid(RED, 2)).await,
        Err(refused.clone())
    );
    assert_eq!(
        backend.enable_direct_mode(&left).await,
        Err(refused.clone())
    );
    assert_eq!(backend.capabilities(&left), Err(refused));

    backend.refresh_inventory().await.expect("barrier");
    assert_eq!(output_packets_in_session(&server, 1), 0);
}

#[tokio::test]
async fn identical_devices_seen_first_time_get_no_reference() {
    let server = FakeServer::new();
    server.add_device(0, twin("left"));
    server.add_device(1, twin("right"));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    let inventory = backend.refresh_inventory().await.expect("inventory");

    assert!(inventory.devices.iter().all(|device| device.identity
        == DeviceIdentity::Ambiguous {
            candidates: Vec::new()
        }));
}

#[tokio::test]
async fn refuses_output_after_device_list_updated() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");

    server.notify_device_list_updated();
    assert_eq!(
        backend.next_event().await,
        Some(BackendEvent::InventoryChanged)
    );

    assert_eq!(
        backend.write_device_frame(&alpha, &solid(RED, 3)).await,
        Err(BackendError::StaleInventory)
    );
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Degraded {
            reasons: vec![StateReason::InventoryStale]
        }
    );
    backend.refresh_inventory().await.expect("refresh");
    assert_eq!(output_packets_in_session(&server, 0), 0);

    backend
        .write_device_frame(&alpha, &solid(RED, 3))
        .await
        .expect("frame after refresh");
    assert_eq!(backend.connection_state(), &ConnectionState::Connected);
    assert_eq!(server.wait_for_color_writes(1).await.len(), 1);
}
