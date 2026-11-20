#![allow(dead_code)]

use prismforge_core::session::SessionDevice;
use prismforge_core::setup::{
    Component, ComponentId, FirstLed, Geometry, Location, OutputId, Placement, Ring, Setup,
    SetupOutput, Single, Strip, Turn, View, ViewId,
};
use prismforge_core::{
    BackendOrigin, DeviceCapabilities, DeviceDescriptor, DeviceIdentity, DeviceKind, DeviceRef,
    ModeDescriptor, ModePersistence, Rgb, ZoneDescriptor, ZoneKind,
};
use serde_json::Map;

pub const BOARD: &str = "fixture-board";
pub const HEADER_1: usize = 0;
pub const HEADER_2: usize = 1;
pub const LOGO: usize = 2;
pub const STRIP: usize = 3;
pub const HEADER_LEDS: usize = 36;

pub fn close(left: Location, right: Location) -> bool {
    (left.x - right.x).abs() < 1e-9 && (left.y - right.y).abs() < 1e-9
}

pub fn assert_locations(found: &[Location], wanted: &[(f64, f64)]) {
    assert_eq!(found.len(), wanted.len(), "{found:?}");
    for (index, (found, wanted)) in found.iter().zip(wanted).enumerate() {
        assert!(
            close(*found, Location::new(wanted.0, wanted.1)),
            "LED {index}: {found:?} is not {wanted:?}"
        );
    }
}

fn zone(name: &str, leds: usize, max: u32) -> ZoneDescriptor {
    ZoneDescriptor {
        name: name.into(),
        kind: ZoneKind::Linear,
        led_count: leds,
        leds_min: if max == leds as u32 { max } else { 0 },
        leds_max: max,
        matrix: None,
    }
}

/// A generic board: two resizable ARGB headers that report 36 LEDs (0 to 1024), a one-LED logo and a fixed 8-LED strip.
pub fn board() -> SessionDevice {
    let zones = vec![
        zone("ARGB Header 1", HEADER_LEDS, 1024),
        zone("ARGB Header 2", HEADER_LEDS, 1024),
        zone("Logo", 1, 1),
        zone("Strip", 8, 8),
    ];
    let led_count = zones.iter().map(|zone| zone.led_count).sum();
    SessionDevice {
        identity: DeviceIdentity::Matched(DeviceRef::new(BOARD)),
        descriptor: DeviceDescriptor {
            origin: BackendOrigin::Simulated,
            name: "Fixture Board".into(),
            vendor: "Fixture Vendor".into(),
            description: String::new(),
            version: String::new(),
            serial: String::new(),
            location: "fixture:board".into(),
            kind: DeviceKind::Motherboard,
            zones,
            led_count,
            colors: vec![Rgb::default(); led_count],
            capabilities: DeviceCapabilities {
                direct_mode: Some(0),
                active_mode: Some(0),
                modes: vec![ModeDescriptor {
                    name: "Direct".into(),
                    speed: None,
                    brightness: None,
                    directions: Vec::new(),
                    per_led_color: true,
                    mode_colors: None,
                    random_color: false,
                    applies_to_entire_device: true,
                    persistence: ModePersistence::None,
                }],
                resizable_zones: Vec::new(),
                saves_to_device_memory: false,
            },
        },
        available: true,
    }
}

pub fn component_id(value: &str) -> ComponentId {
    ComponentId::parse(value).expect("component id")
}

pub fn output_id(value: &str) -> OutputId {
    OutputId::parse(value).expect("output id")
}

pub fn view_id(value: &str) -> ViewId {
    ViewId::parse(value).expect("view id")
}

pub fn strip(leds: u32) -> Geometry {
    Geometry::Strip(Strip {
        leds,
        first: FirstLed::AtStart,
        extra: Map::new(),
    })
}

pub fn ring(leds: u32) -> Geometry {
    Geometry::Ring(Ring {
        leds,
        first_angle: 0,
        turn: Turn::Clockwise,
        extra: Map::new(),
    })
}

pub fn point() -> Geometry {
    Geometry::Point(Single::default())
}

pub fn component(id: &str, geometry: Geometry) -> Component {
    Component {
        id: component_id(id),
        name: id.trim_start_matches("c-").into(),
        geometry,
        frame: None,
        extra: Map::new(),
    }
}

pub fn output(id: &str, zone: usize, first_led: u32, branches: &[&[&str]]) -> SetupOutput {
    let device = board();
    SetupOutput {
        id: output_id(id),
        device: DeviceRef::new(BOARD),
        zone: Some(zone),
        label: device.descriptor.name.clone(),
        zone_name: Some(device.descriptor.zones[zone].name.clone()),
        first_led,
        branches: branches
            .iter()
            .map(|branch| branch.iter().map(|id| component_id(id)).collect())
            .collect(),
        extra: Map::new(),
    }
}

pub fn placed(id: &str, x: i32, y: i32, size: u32) -> Placement {
    Placement {
        component: component_id(id),
        x,
        y,
        size,
        rotation: 0,
        flipped: false,
        extra: Map::new(),
    }
}

pub fn view(id: &str, placements: Vec<Placement>) -> View {
    View {
        id: view_id(id),
        name: "PC".into(),
        placements,
        outlines: Vec::new(),
        extra: Map::new(),
    }
}

/// Three 12-LED fans on a hub of header 1: every fan gets the same data.
pub fn mirrored_fans() -> Setup {
    Setup {
        components: vec![
            component("c-fan-a", ring(12)),
            component("c-fan-b", ring(12)),
            component("c-fan-c", ring(12)),
        ],
        outputs: vec![output(
            "o-hub",
            HEADER_1,
            0,
            &[&["c-fan-a"], &["c-fan-b"], &["c-fan-c"]],
        )],
        views: Vec::new(),
        extra: Map::new(),
    }
}

/// Three 12-LED fans chained on header 2: one zone split into three parts in a row.
pub fn chained_fans() -> Setup {
    Setup {
        components: vec![
            component("c-fan-1", ring(12)),
            component("c-fan-2", ring(12)),
            component("c-fan-3", ring(12)),
        ],
        outputs: vec![output(
            "o-chain",
            HEADER_2,
            0,
            &[&["c-fan-1", "c-fan-2", "c-fan-3"]],
        )],
        views: Vec::new(),
        extra: Map::new(),
    }
}
