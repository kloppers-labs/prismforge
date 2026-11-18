use std::time::Duration;

use prismforge_core::{DeviceRef, Inventory, Rgb};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{FakeServer, FixtureDevice};
use tokio::io::DuplexStream;

pub const TIMEOUT: Duration = Duration::from_millis(500);

pub fn config() -> ClientConfig {
    ClientConfig {
        request_timeout: TIMEOUT,
        ..ClientConfig::default()
    }
}

pub fn backend(server: &FakeServer) -> OpenRgbBackend<impl Connector<Stream = DuplexStream>> {
    OpenRgbBackend::new(server.connector(), config())
}

pub fn strip(label: &str, leds: u32) -> FixtureDevice {
    FixtureDevice::new(label, &format!("Strip {label}"))
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .zone("Strip", leds)
}

pub fn reference_by_serial(inventory: &Inventory, serial: &str) -> DeviceRef {
    inventory
        .devices
        .iter()
        .find(|device| device.descriptor.serial == serial)
        .and_then(|device| device.identity.reference())
        .cloned()
        .unwrap_or_else(|| panic!("no resolved device with serial {serial}"))
}

pub fn solid(color: Rgb, count: usize) -> Vec<Rgb> {
    vec![color; count]
}

pub fn wire(colors: &[Rgb]) -> Vec<[u8; 3]> {
    colors.iter().map(|c| [c.r, c.g, c.b]).collect()
}

pub const RED: Rgb = Rgb::new(255, 0, 0);
pub const GREEN: Rgb = Rgb::new(0, 255, 0);
pub const BLUE: Rgb = Rgb::new(0, 0, 255);
