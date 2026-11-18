use prismforge_core::{
    BackendOrigin, DeviceCapabilities, DeviceDescriptor, DeviceKind, DeviceRef, ModeDescriptor,
    ModeDirection, ModePersistence, Rgb, ValueRange, ZoneDescriptor, ZoneKind, ZoneResizeBounds,
};

const VENDOR: &str = "PrismForge Demo";
const DESCRIPTION: &str = "Simulated device, no hardware";

pub(crate) struct DemoDevice {
    pub reference: DeviceRef,
    pub descriptor: DeviceDescriptor,
}

pub(crate) fn demo_devices() -> Vec<DemoDevice> {
    vec![
        memory_module(1),
        memory_module(2),
        motherboard(),
        graphics_card(),
        light_bar(),
    ]
}

fn memory_module(slot: u32) -> DemoDevice {
    let zones = vec![fixed_zone("Module", 8)];
    device(
        &format!("demo-memory-{slot}"),
        Spec {
            name: "Demo Memory Module",
            kind: DeviceKind::Memory,
            serial: "",
            location: &format!("Simulated memory slot {slot}"),
            modes: vec![direct(), static_color(), breathing()],
            active_mode: 1,
            colors: solid(Rgb::new(255, 96, 32), 8),
        },
        zones,
    )
}

fn motherboard() -> DemoDevice {
    let zones = vec![
        fixed_zone("Onboard", 6),
        ZoneDescriptor {
            leds_min: 0,
            leds_max: 120,
            ..fixed_zone("Addressable Header 1", 30)
        },
        fixed_zone("Addressable Header 2", 16),
    ];
    device(
        "demo-motherboard",
        Spec {
            name: "Demo Motherboard",
            kind: DeviceKind::Motherboard,
            serial: "SIMULATED-MB-01",
            location: "Simulated system board",
            modes: vec![direct(), static_color(), rainbow_wave()],
            active_mode: 0,
            colors: gradient(52),
        },
        zones,
    )
}

fn graphics_card() -> DemoDevice {
    device(
        "demo-graphics-card",
        Spec {
            name: "Demo Graphics Card",
            kind: DeviceKind::Gpu,
            serial: "SIMULATED-GPU-01",
            location: "Simulated expansion slot 1",
            modes: vec![direct(), static_color(), breathing()],
            active_mode: 2,
            colors: solid(Rgb::new(0, 160, 255), 12),
        },
        vec![fixed_zone("Logo", 12)],
    )
}

fn light_bar() -> DemoDevice {
    device(
        "demo-light-bar",
        Spec {
            name: "Demo Light Bar",
            kind: DeviceKind::Light,
            serial: "SIMULATED-BAR-01",
            location: "Simulated USB port 1",
            modes: vec![static_color(), breathing()],
            active_mode: 0,
            colors: solid(Rgb::new(255, 255, 255), 10),
        },
        vec![fixed_zone("Bar", 10)],
    )
}

struct Spec<'a> {
    name: &'a str,
    kind: DeviceKind,
    serial: &'a str,
    location: &'a str,
    modes: Vec<ModeDescriptor>,
    active_mode: usize,
    colors: Vec<Rgb>,
}

fn device(reference: &str, spec: Spec, zones: Vec<ZoneDescriptor>) -> DemoDevice {
    let led_count = zones.iter().map(|zone| zone.led_count).sum();
    assert_eq!(spec.colors.len(), led_count, "demo colours for {reference}");
    let capabilities = DeviceCapabilities {
        direct_mode: spec.modes.iter().position(|mode| mode.name == "Direct"),
        active_mode: Some(spec.active_mode),
        resizable_zones: zones
            .iter()
            .enumerate()
            .filter(|(_, zone)| zone.leds_min < zone.leds_max)
            .map(|(index, zone)| ZoneResizeBounds {
                zone: index,
                min: zone.leds_min,
                max: zone.leds_max,
            })
            .collect(),
        saves_to_device_memory: spec
            .modes
            .iter()
            .any(|mode| mode.persistence == ModePersistence::ManualSave),
        modes: spec.modes,
    };
    DemoDevice {
        reference: DeviceRef::new(reference),
        descriptor: DeviceDescriptor {
            origin: BackendOrigin::Simulated,
            name: spec.name.into(),
            vendor: VENDOR.into(),
            description: DESCRIPTION.into(),
            version: "1.0".into(),
            serial: spec.serial.into(),
            location: spec.location.into(),
            kind: spec.kind,
            zones,
            led_count,
            colors: spec.colors,
            capabilities,
        },
    }
}

fn fixed_zone(name: &str, leds: u32) -> ZoneDescriptor {
    ZoneDescriptor {
        name: name.into(),
        kind: ZoneKind::Linear,
        led_count: leds as usize,
        leds_min: leds,
        leds_max: leds,
        matrix: None,
    }
}

fn mode(name: &str) -> ModeDescriptor {
    ModeDescriptor {
        name: name.into(),
        speed: None,
        brightness: None,
        directions: Vec::new(),
        per_led_color: false,
        mode_colors: None,
        random_color: false,
        applies_to_entire_device: false,
        persistence: ModePersistence::None,
    }
}

fn direct() -> ModeDescriptor {
    ModeDescriptor {
        per_led_color: true,
        ..mode("Direct")
    }
}

fn static_color() -> ModeDescriptor {
    ModeDescriptor {
        mode_colors: Some(ValueRange { min: 1, max: 1 }),
        persistence: ModePersistence::ManualSave,
        ..mode("Static")
    }
}

fn breathing() -> ModeDescriptor {
    ModeDescriptor {
        speed: Some(ValueRange { min: 1, max: 5 }),
        brightness: Some(ValueRange { min: 0, max: 100 }),
        mode_colors: Some(ValueRange { min: 1, max: 2 }),
        persistence: ModePersistence::ManualSave,
        ..mode("Breathing")
    }
}

fn rainbow_wave() -> ModeDescriptor {
    ModeDescriptor {
        speed: Some(ValueRange { min: 1, max: 5 }),
        directions: vec![ModeDirection::LeftRight],
        applies_to_entire_device: true,
        ..mode("Rainbow Wave")
    }
}

fn solid(color: Rgb, count: usize) -> Vec<Rgb> {
    vec![color; count]
}

fn gradient(count: usize) -> Vec<Rgb> {
    (0..count)
        .map(|index| {
            let step = (index * 255 / (count - 1)) as u8;
            Rgb::new(step, 64, 255 - step)
        })
        .collect()
}
