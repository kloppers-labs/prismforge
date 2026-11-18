use std::time::Duration;

use prismforge_core::session::{
    DeviceOutput, ReconnectPolicy, Session, SessionConfig, SessionHandle, SessionSnapshot,
};
use prismforge_core::{ConnectionState, DeviceRef, OutputTarget, Rgb, RgbBackend};
use prismforge_effects::{ControlValue, EffectInstance};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{
    DemoBackend, DemoControls, DemoOutput, FakeServer, FixtureDevice, ReceivedPacket, wire,
};
use tokio::io::DuplexStream;

pub const REQUEST_TIMEOUT: Duration = Duration::from_millis(500);
pub const FIRST_RETRY: Duration = Duration::from_millis(100);
pub const MAX_RETRY: Duration = Duration::from_millis(800);
const PATIENCE: Duration = Duration::from_secs(3600);

pub fn config() -> SessionConfig {
    SessionConfig {
        command_capacity: 4,
        event_capacity: 64,
        reconnect: ReconnectPolicy {
            initial_delay: FIRST_RETRY,
            max_delay: MAX_RETRY,
        },
        max_identify_duration: Duration::from_secs(30),
        frame_rate: 30,
    }
}

pub fn client_config() -> ClientConfig {
    ClientConfig {
        request_timeout: REQUEST_TIMEOUT,
        ..ClientConfig::default()
    }
}

pub fn start<B: RgbBackend + Send + 'static>(backend: B, config: SessionConfig) -> SessionHandle {
    let (handle, session) = Session::new(backend, config);
    tokio::spawn(session.run());
    handle
}

pub fn openrgb(
    server: &FakeServer,
) -> OpenRgbBackend<impl Connector<Stream = DuplexStream> + use<>> {
    OpenRgbBackend::new(server.connector(), client_config())
}

pub async fn connected_openrgb(server: &FakeServer) -> SessionHandle {
    let handle = start(openrgb(server), config());
    handle.connect().await.expect("connect");
    handle
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

pub fn is_live(snapshot: &SessionSnapshot) -> bool {
    matches!(
        snapshot.connection,
        ConnectionState::Connected | ConnectionState::Degraded { .. }
    )
}

pub fn is_retrying(snapshot: &SessionSnapshot) -> bool {
    matches!(snapshot.connection, ConnectionState::Retrying { .. })
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

pub const GPU: &str = "demo-graphics-card";
pub const GPU_LEDS: usize = 12;
pub const MEMORY: &str = "demo-memory-1";
pub const MOTHERBOARD: &str = "demo-motherboard";
pub const LIGHT_BAR: &str = "demo-light-bar";

pub fn effect(id: &str, values: &[(&str, ControlValue)]) -> EffectInstance {
    EffectInstance {
        effect: id.into(),
        values: values
            .iter()
            .map(|(control, value)| ((*control).to_owned(), value.clone()))
            .collect(),
    }
}

pub fn rainbow() -> EffectInstance {
    effect("rainbow-wave", &[])
}

pub fn solid(color: Rgb) -> EffectInstance {
    effect("solid", &[("color", ControlValue::Color(color))])
}

pub fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

pub async fn demo(config: SessionConfig) -> (SessionHandle, DemoControls) {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config);
    handle.connect().await.expect("demo connects");
    (handle, controls)
}

pub fn device_frames(outputs: &[DemoOutput], device: &str) -> Vec<Vec<Rgb>> {
    outputs
        .iter()
        .filter_map(|output| match output {
            DemoOutput::DeviceFrame {
                device: target,
                colors,
            } if target.as_str() == device => Some(colors.clone()),
            _ => None,
        })
        .collect()
}

pub fn direct_mode_calls(outputs: &[DemoOutput], device: &str) -> usize {
    outputs
        .iter()
        .filter(|output| {
            matches!(output, DemoOutput::DirectMode { device: target } if target.as_str() == device)
        })
        .count()
}

pub fn output_of(snapshot: &SessionSnapshot, device: &str) -> Option<DeviceOutput> {
    snapshot
        .output
        .devices
        .iter()
        .find(|output| output.device.as_str() == device)
        .cloned()
}
