use std::time::Duration;

use prismforge_core::session::{
    ReconnectPolicy, Session, SessionConfig, SessionHandle, SessionSnapshot,
};
use prismforge_core::{ConnectionState, DeviceRef, KnownDevice, OutputTarget, RgbBackend};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{
    DemoBackend, DemoControls, FakeServer, FixtureDevice, ReceivedPacket, wire,
};
use tokio::io::DuplexStream;

const PATIENCE: Duration = Duration::from_secs(3600);

pub fn config() -> SessionConfig {
    SessionConfig {
        command_capacity: 4,
        event_capacity: 64,
        reconnect: ReconnectPolicy {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(800),
        },
        max_identify_duration: Duration::from_secs(30),
        frame_rate: 30,
    }
}

pub fn start<B: RgbBackend + Send + 'static>(backend: B) -> SessionHandle {
    let (handle, session) = Session::new(backend, config());
    tokio::spawn(session.run());
    handle
}

pub fn openrgb(
    server: &FakeServer,
    known: Vec<KnownDevice>,
) -> OpenRgbBackend<impl Connector<Stream = DuplexStream> + use<>> {
    let mut backend = OpenRgbBackend::new(
        server.connector(),
        ClientConfig {
            request_timeout: Duration::from_millis(500),
            ..ClientConfig::default()
        },
    );
    backend.seed_known_devices(known);
    backend
}

pub async fn connected(server: &FakeServer, known: Vec<KnownDevice>) -> SessionHandle {
    let handle = start(openrgb(server, known));
    handle.connect().await.expect("connect");
    handle
}

pub async fn demo() -> (SessionHandle, DemoControls) {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend);
    handle.connect().await.expect("demo connects");
    (handle, controls)
}

pub async fn wait_until(
    handle: &SessionHandle,
    condition: impl FnMut(&SessionSnapshot) -> bool,
) -> SessionSnapshot {
    let mut updates = handle.subscribe();
    tokio::time::timeout(PATIENCE, updates.wait_for(condition))
        .await
        .expect("condition never reached")
        .expect("session stopped")
        .clone()
}

pub fn is_connected(snapshot: &SessionSnapshot) -> bool {
    snapshot.connection == ConnectionState::Connected
}

pub fn strip(label: &str, leds: u32) -> FixtureDevice {
    FixtureDevice::new(label, &format!("Strip {label}"))
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .zone("Strip", leds)
}

pub fn reference(snapshot: &SessionSnapshot, serial: &str) -> DeviceRef {
    snapshot
        .devices
        .iter()
        .find(|device| device.descriptor.serial == serial)
        .and_then(|device| device.identity.reference())
        .cloned()
        .unwrap_or_else(|| panic!("no resolved device with serial {serial}"))
}

pub fn whole(device: &DeviceRef) -> OutputTarget {
    OutputTarget {
        device: device.clone(),
        zone: None,
    }
}

pub fn output_packets(server: &FakeServer) -> Vec<ReceivedPacket> {
    server
        .received()
        .into_iter()
        .filter(|packet| {
            [
                wire::SET_CUSTOM_MODE,
                wire::UPDATE_LEDS,
                wire::UPDATE_ZONE_LEDS,
                wire::UPDATE_MODE,
                wire::SAVE_MODE,
            ]
            .contains(&packet.packet_id)
        })
        .collect()
}
