use prismforge_core::session::{SessionConfig, SessionEventKind};
use prismforge_core::{BackendInfo, BackendOrigin};
use prismforge_openrgb::{OpenRgbBackend, TcpConnector};
use prismforge_testkit::{DemoBackend, FakeServer};

use crate::support::{client_config, config, start, strip};

#[tokio::test(start_paused = true)]
async fn event_ring_stays_at_its_fixed_capacity() {
    let handle = start(
        DemoBackend::new(),
        SessionConfig {
            event_capacity: 4,
            ..config()
        },
    );
    handle.connect().await.expect("connect");
    for _ in 0..6 {
        handle.refresh().await.expect("refresh");
    }

    let diagnostics = handle.snapshot().diagnostics;

    assert_eq!(diagnostics.events.len(), 4);
    let sequences: Vec<u64> = diagnostics.events.iter().map(|e| e.sequence).collect();
    let newest = sequences[3];
    assert_eq!(sequences, [newest - 3, newest - 2, newest - 1, newest]);
    assert!(newest >= 8, "{sequences:?}");
    assert!(
        diagnostics
            .events
            .iter()
            .all(|event| matches!(event.kind, SessionEventKind::InventoryRefreshed { .. }))
    );
    assert_eq!(diagnostics.counters.connect_attempts, 1);
    assert_eq!(diagnostics.counters.inventory_refreshes, 7);
}

#[tokio::test(start_paused = true)]
async fn demo_session_reports_simulated_origin() {
    let handle = start(DemoBackend::new(), config());
    assert_eq!(handle.snapshot().backend.origin, BackendOrigin::Simulated);

    handle.connect().await.expect("connect");

    let snapshot = handle.snapshot();
    assert_eq!(snapshot.backend.origin, BackendOrigin::Simulated);
    assert_eq!(snapshot.devices.len(), 5);
    assert!(
        snapshot
            .devices
            .iter()
            .all(|device| device.descriptor.origin == BackendOrigin::Simulated)
    );
}

#[tokio::test]
async fn reports_protocol_server_name_and_address() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("bind");
    let port = listener.local_addr().expect("address").port();
    let serving = server.clone();
    tokio::spawn(async move { serving.serve_tcp(listener).await });
    let backend = OpenRgbBackend::new(TcpConnector::new("127.0.0.1", port), client_config());
    let handle = start(backend, config());

    handle.connect().await.expect("connect");

    assert_eq!(
        handle.snapshot().backend,
        BackendInfo {
            origin: BackendOrigin::OpenRgb,
            address: Some(format!("127.0.0.1:{port}")),
            protocol_version: Some(6),
            server_name: Some("PrismForge fake OpenRGB".into()),
        }
    );
}
