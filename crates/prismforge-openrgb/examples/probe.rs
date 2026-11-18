//! Read-only OpenRGB inventory probe; writes a colour only when `--device` and `--color` are both given.

use std::process::ExitCode;

use prismforge_core::{DeviceIdentity, DeviceRef, InventoryDevice, Rgb, RgbBackend};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend, TcpConnector};

const USAGE: &str =
    "usage: probe [--host HOST] [--port PORT] [--device REF --color RRGGBB [--zone INDEX]]";

struct Args {
    host: String,
    port: u16,
    write: Option<Write>,
}

struct Write {
    device: DeviceRef,
    color: Rgb,
    zone: Option<usize>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let connector = TcpConnector::new(args.host.clone(), args.port);
    let mut backend = OpenRgbBackend::new(connector, ClientConfig::default());

    if let Err(error) = backend.connect().await {
        eprintln!("connect to {}:{} failed: {error}", args.host, args.port);
        return ExitCode::FAILURE;
    }
    println!(
        "negotiated protocol {}",
        backend.protocol_version().unwrap_or_default()
    );
    let inventory = match backend.refresh_inventory().await {
        Ok(inventory) => inventory.clone(),
        Err(error) => {
            eprintln!("inventory failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    for device in &inventory.devices {
        print_device(device);
    }
    for issue in &inventory.issues {
        println!(
            "rejected controller {}: {:?} ({})",
            issue.runtime_id, issue.reason, issue.detail
        );
    }
    println!("connection state: {:?}", backend.connection_state());

    let Some(write) = args.write else {
        return ExitCode::SUCCESS;
    };
    match apply(&mut backend, &write).await {
        Ok(()) => {
            println!("wrote {:?} to {}", write.color, write.device);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("colour write refused: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn apply<C: Connector>(
    backend: &mut OpenRgbBackend<C>,
    write: &Write,
) -> Result<(), prismforge_core::BackendError> {
    let descriptor = &backend
        .inventory()
        .ok_or(prismforge_core::BackendError::StaleInventory)?
        .output_target(&write.device, write.zone)?
        .descriptor;
    let led_count = match write.zone {
        Some(zone) => descriptor.zones[zone].led_count,
        None => descriptor.led_count,
    };
    let colors = vec![write.color; led_count];
    backend.enable_direct_mode(&write.device).await?;
    match write.zone {
        Some(zone) => backend.write_zone_frame(&write.device, zone, &colors).await,
        None => backend.write_device_frame(&write.device, &colors).await,
    }
}

fn print_device(device: &InventoryDevice) {
    let descriptor = &device.descriptor;
    let identity = match &device.identity {
        DeviceIdentity::Matched(reference) | DeviceIdentity::New(reference) => {
            reference.to_string()
        }
        DeviceIdentity::Ambiguous { .. } => "ambiguous (output disabled)".into(),
    };
    println!(
        "\n{identity}: {} / {} [{:?}] serial={:?} location={:?} leds={}",
        descriptor.vendor,
        descriptor.name,
        descriptor.kind,
        descriptor.serial,
        descriptor.location,
        descriptor.led_count
    );
    for (index, zone) in descriptor.zones.iter().enumerate() {
        println!(
            "  zone {index} {:?} [{:?}] leds={} bounds={}..={} matrix={:?}",
            zone.name, zone.kind, zone.led_count, zone.leds_min, zone.leds_max, zone.matrix
        );
    }
    let capabilities = &descriptor.capabilities;
    println!(
        "  direct mode: {:?}, saves to device memory: {}, active mode: {:?}",
        capabilities.direct_mode, capabilities.saves_to_device_memory, capabilities.active_mode
    );
    for mode in &capabilities.modes {
        println!(
            "  mode {:?}: speed={:?} brightness={:?} directions={:?} per-led={} colours={:?} random={} persistence={:?}",
            mode.name,
            mode.speed,
            mode.brightness,
            mode.directions,
            mode.per_led_color,
            mode.mode_colors,
            mode.random_color,
            mode.persistence
        );
    }
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut host = "127.0.0.1".to_owned();
    let mut port = TcpConnector::DEFAULT_PORT;
    let (mut device, mut color, mut zone) = (None, None, None);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--host" => host = value()?,
            "--port" => port = value()?.parse().map_err(|_| "invalid --port".to_owned())?,
            "--device" => device = Some(DeviceRef::new(value()?)),
            "--color" => color = Some(parse_color(&value()?)?),
            "--zone" => zone = Some(value()?.parse().map_err(|_| "invalid --zone".to_owned())?),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let write = match (device, color) {
        (Some(device), Some(color)) => Some(Write {
            device,
            color,
            zone,
        }),
        (None, None) if zone.is_none() => None,
        _ => return Err("a colour write needs both --device and --color".into()),
    };
    Ok(Args { host, port, write })
}

fn parse_color(text: &str) -> Result<Rgb, String> {
    let value = u32::from_str_radix(text.trim_start_matches('#'), 16)
        .ok()
        .filter(|_| text.trim_start_matches('#').len() == 6)
        .ok_or(format!("invalid colour {text}; expected RRGGBB"))?;
    let [_, r, g, b] = value.to_be_bytes();
    Ok(Rgb::new(r, g, b))
}

#[cfg(test)]
mod tests {
    use prismforge_core::{BackendError, Rgb, RgbBackend};
    use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
    use prismforge_testkit::{FakeServer, FixtureDevice, wire};

    use super::{Write, apply};

    fn hardware_writes(server: &FakeServer) -> usize {
        server
            .received()
            .iter()
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
            .count()
    }

    async fn refused_write(device: FixtureDevice, zone: Option<usize>) -> (BackendError, usize) {
        let server = FakeServer::new();
        server.add_device(0, device);
        let mut backend = OpenRgbBackend::new(server.connector(), ClientConfig::default());
        backend.connect().await.expect("connect");
        let inventory = backend.refresh_inventory().await.expect("inventory");
        let write = Write {
            device: inventory.devices[0]
                .identity
                .reference()
                .cloned()
                .expect("resolved"),
            color: Rgb::new(255, 0, 0),
            zone,
        };

        let error = apply(&mut backend, &write).await.unwrap_err();
        backend.refresh_inventory().await.expect("barrier");
        (error, hardware_writes(&server))
    }

    #[tokio::test]
    async fn out_of_range_zone_sends_no_hardware_write() {
        let device = FixtureDevice::new("alpha", "Strip").zone("Strip", 3);

        let (error, writes) = refused_write(device, Some(1)).await;

        assert!(
            matches!(error, BackendError::ZoneOutOfRange { zone: 1, .. }),
            "{error:?}"
        );
        assert_eq!(writes, 0);
    }

    #[tokio::test]
    async fn device_without_direct_mode_sends_no_hardware_write() {
        let device = FixtureDevice::new("alpha", "Strip")
            .zone("Strip", 3)
            .without_direct_mode();

        let (error, writes) = refused_write(device, None).await;

        assert!(
            matches!(error, BackendError::DirectModeUnavailable { .. }),
            "{error:?}"
        );
        assert_eq!(writes, 0);
    }
}
