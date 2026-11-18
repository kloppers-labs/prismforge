use std::time::Duration;

use prismforge_core::{BackendError, ConnectionState, RgbBackend, StateReason};
use prismforge_openrgb::{OpenRgbBackend, TcpConnector};
use prismforge_testkit::{FakeServer, wire};
use tokio::time::Instant;

use crate::support::{self, RED, TIMEOUT, solid, strip};

fn payloads_of(server: &FakeServer, packet_id: u32) -> Vec<Vec<u8>> {
    server
        .received_with_id(packet_id)
        .into_iter()
        .map(|packet| packet.payload)
        .collect()
}

#[tokio::test]
async fn negotiates_supported_protocol() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);

    backend
        .connect()
        .await
        .expect("protocol 6 server must be accepted");

    assert_eq!(backend.protocol_version(), Some(6));
    assert_eq!(backend.connection_state(), &ConnectionState::Connected);
    assert_eq!(
        payloads_of(&server, wire::REQUEST_PROTOCOL_VERSION),
        vec![6u32.to_le_bytes().to_vec()]
    );

    let inventory = backend.refresh_inventory().await.expect("inventory");
    assert_eq!(
        payloads_of(&server, wire::SET_CLIENT_NAME),
        vec![b"PrismForge\0".to_vec()]
    );
    assert_eq!(inventory.devices.len(), 1);
    assert_eq!(inventory.devices[0].descriptor.name, "Strip alpha");
    let data_requests = server.received_with_id(wire::REQUEST_CONTROLLER_DATA);
    assert_eq!(data_requests.len(), 1);
    assert_eq!(data_requests[0].device_id, 0);
    assert_eq!(data_requests[0].payload, 6u32.to_le_bytes());
}

#[tokio::test]
async fn rejects_server_below_protocol_6() {
    let server = FakeServer::new();
    server.set_protocol_version(Some(5));
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);

    let error = backend.connect().await.unwrap_err();

    assert_eq!(
        error,
        BackendError::UnsupportedProtocol {
            server_version: Some(5),
            required: 6
        }
    );
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Disconnected {
            reason: Some(StateReason::UnsupportedProtocol {
                server_version: Some(5)
            })
        }
    );
    assert!(
        server
            .received_with_id(wire::REQUEST_CONTROLLER_COUNT)
            .is_empty()
    );
}

#[tokio::test]
async fn runs_newer_server_at_protocol_6() {
    let server = FakeServer::new();
    server.set_protocol_version(Some(7));
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);

    backend
        .connect()
        .await
        .expect("newer server must be accepted");
    assert_eq!(backend.protocol_version(), Some(6));
    let inventory = backend.refresh_inventory().await.expect("inventory");

    assert_eq!(inventory.devices.len(), 1);
    assert_eq!(
        payloads_of(&server, wire::REQUEST_CONTROLLER_DATA),
        vec![6u32.to_le_bytes().to_vec()]
    );
}

#[tokio::test(start_paused = true)]
async fn treats_unanswered_version_request_as_unsupported() {
    let server = FakeServer::new();
    server.set_protocol_version(None);
    let mut backend = support::backend(&server);

    let error = backend.connect().await.unwrap_err();

    assert_eq!(
        error,
        BackendError::UnsupportedProtocol {
            server_version: None,
            required: 6
        }
    );
}

#[tokio::test(start_paused = true)]
async fn times_out_request_without_reply() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.silence(wire::REQUEST_CONTROLLER_COUNT);
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    let started = Instant::now();
    let error = backend.refresh_inventory().await.unwrap_err();

    assert!(matches!(error, BackendError::Timeout { .. }), "{error:?}");
    assert!(started.elapsed() >= TIMEOUT && started.elapsed() < TIMEOUT * 2);
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Disconnected {
            reason: Some(StateReason::ConnectionLost)
        }
    );
}

#[tokio::test(start_paused = true)]
async fn accepts_slow_reply_within_timeout() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.delay_replies(wire::REQUEST_CONTROLLER_COUNT, Duration::from_millis(400));
    let mut backend = support::backend(&server);

    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");

    assert_eq!(inventory.devices.len(), 1);
}

#[tokio::test]
async fn skips_unsolicited_and_unknown_packets() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    server.broadcast(wire::frame(0, 9_999, &[1, 2, 3, 4, 5]));
    server.broadcast(wire::frame(0, wire::DETECTION_PROGRESS_CHANGED, &[0; 12]));
    server.broadcast(wire::frame(0, wire::SIGNAL_UPDATE, &vec![7; 4_096]));
    server.broadcast(wire::frame(0, wire::ACK, &[0; 3]));
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");

    backend
        .write_device_frame(&alpha, &solid(RED, 3))
        .await
        .expect("frame");
    assert_eq!(server.wait_for_color_writes(1).await.len(), 1);
}

#[tokio::test]
async fn connects_over_loopback_tcp() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("bind");
    let port = listener.local_addr().expect("address").port();
    let serving = server.clone();
    tokio::spawn(async move { serving.serve_tcp(listener).await });
    let mut backend = OpenRgbBackend::new(TcpConnector::new("127.0.0.1", port), support::config());

    backend.connect().await.expect("connect over tcp");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");
    backend
        .write_device_frame(&alpha, &solid(RED, 3))
        .await
        .expect("frame");

    let writes = server.wait_for_color_writes(1).await;
    assert_eq!(writes[0].target.as_deref(), Some("alpha"));
}
