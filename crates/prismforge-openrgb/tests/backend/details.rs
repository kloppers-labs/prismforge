use prismforge_core::{BackendInfo, BackendOrigin, Rgb, RgbBackend};
use prismforge_openrgb::{OpenRgbBackend, TcpConnector};
use prismforge_testkit::{FakeServer, FixtureDevice, FixtureMode};

use crate::support::{self, strip};

#[tokio::test]
async fn real_backend_devices_are_marked_as_openrgb() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 2).without_direct_mode());
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    let inventory = backend.refresh_inventory().await.expect("inventory");

    assert_eq!(inventory.devices.len(), 2);
    assert!(
        inventory
            .devices
            .iter()
            .all(|device| device.descriptor.origin == BackendOrigin::OpenRgb)
    );
    assert_eq!(backend.info().origin, BackendOrigin::OpenRgb);
}

#[tokio::test]
async fn reports_protocol_server_name_and_address() {
    let server = FakeServer::new();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("bind");
    let port = listener.local_addr().expect("address").port();
    let serving = server.clone();
    tokio::spawn(async move { serving.serve_tcp(listener).await });
    let mut backend = OpenRgbBackend::new(TcpConnector::new("127.0.0.1", port), support::config());
    let address = format!("127.0.0.1:{port}");

    assert_eq!(
        backend.info(),
        BackendInfo {
            origin: BackendOrigin::OpenRgb,
            address: Some(address.clone()),
            protocol_version: None,
            server_name: None,
        }
    );
    backend.connect().await.expect("connect");
    backend.refresh_inventory().await.expect("inventory");

    assert_eq!(
        backend.info(),
        BackendInfo {
            origin: BackendOrigin::OpenRgb,
            address: Some(address),
            protocol_version: Some(6),
            server_name: Some("PrismForge fake OpenRGB".into()),
        }
    );
}

#[tokio::test]
async fn reports_current_led_colours_and_direct_mode_index() {
    let server = FakeServer::new();
    let device = strip("alpha", 3)
        .modes(vec![FixtureMode::static_color(), FixtureMode::direct()])
        .active_mode(1)
        .led_colors(vec![[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
    server.add_device(0, device);
    server.add_device(
        1,
        FixtureDevice::new("plain", "Plain").without_direct_mode(),
    );
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    let inventory = backend.refresh_inventory().await.expect("inventory");

    let alpha = &inventory.devices[0].descriptor;
    assert_eq!(
        alpha.colors,
        [Rgb::new(1, 2, 3), Rgb::new(4, 5, 6), Rgb::new(7, 8, 9)]
    );
    assert_eq!(alpha.capabilities.direct_mode, Some(1));
    assert!(alpha.capabilities.in_direct_mode());
    let plain = &inventory.devices[1].descriptor.capabilities;
    assert_eq!(plain.direct_mode, None);
    assert!(!plain.in_direct_mode());
}
