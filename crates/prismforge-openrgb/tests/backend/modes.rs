use prismforge_core::{BackendError, BackendEvent, RgbBackend};
use prismforge_testkit::{FakeServer, FixtureDevice, FixtureMode, wire};

use crate::support::{self, strip};

fn mode_packets(server: &FakeServer) -> usize {
    server
        .received()
        .iter()
        .filter(|packet| [wire::UPDATE_MODE, wire::SAVE_MODE].contains(&packet.packet_id))
        .count()
}

#[tokio::test]
async fn reapplies_reported_mode_to_its_device_without_saving() {
    let server = FakeServer::new();
    server.add_device(3, strip("alpha", 3));
    server.add_device(4, strip("beta", 2).without_direct_mode());
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let beta = support::reference_by_serial(inventory, "SN-beta");

    backend.reapply_mode(&beta, 1).await.expect("re-apply");

    let updates = server.mode_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].device_id, 4);
    assert_eq!(updates[0].target.as_deref(), Some("beta"));
    assert_eq!(updates[0].mode_index, 1);
    assert_eq!(updates[0].mode, FixtureMode::breathing());
    assert!(server.received_with_id(wire::SAVE_MODE).is_empty());
}

#[tokio::test]
async fn refuses_out_of_range_mode_before_sending() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");

    assert_eq!(
        backend.reapply_mode(&alpha, 2).await,
        Err(BackendError::ModeOutOfRange {
            device: alpha.clone(),
            mode: 2,
            mode_count: 2
        })
    );

    backend.refresh_inventory().await.expect("barrier");
    assert_eq!(mode_packets(&server), 0);
}

#[tokio::test]
async fn refuses_mode_reapply_on_stale_inventory() {
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
        backend.reapply_mode(&alpha, 1).await,
        Err(BackendError::StaleInventory)
    );
    backend.refresh_inventory().await.expect("barrier");
    assert_eq!(mode_packets(&server), 0);
}

#[tokio::test]
async fn refuses_mode_reapply_on_ambiguous_device() {
    let twin = |label: &str| FixtureDevice::new(label, "Twin Stick").zone("Stick", 2);
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
    backend.refresh_inventory().await.expect("inventory");

    assert_eq!(
        backend.reapply_mode(&left, 1).await,
        Err(BackendError::AmbiguousDevice {
            device: left.clone()
        })
    );
    backend.refresh_inventory().await.expect("barrier");
    assert_eq!(mode_packets(&server), 0);
}
