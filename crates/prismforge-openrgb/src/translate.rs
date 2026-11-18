use prismforge_core::{
    DeviceCapabilities, DeviceDescriptor, DeviceKind, MatrixSize, ModeDescriptor, ModeDirection,
    ModePersistence, ValueRange, ZoneDescriptor, ZoneKind, ZoneResizeBounds,
};

use crate::protocol::{ControllerData, ModeData, ZoneData};

const MODE_HAS_SPEED: u32 = 1 << 0;
const MODE_HAS_DIRECTION_LR: u32 = 1 << 1;
const MODE_HAS_DIRECTION_UD: u32 = 1 << 2;
const MODE_HAS_DIRECTION_HV: u32 = 1 << 3;
const MODE_HAS_BRIGHTNESS: u32 = 1 << 4;
const MODE_HAS_PER_LED_COLOR: u32 = 1 << 5;
const MODE_HAS_MODE_SPECIFIC_COLOR: u32 = 1 << 6;
const MODE_HAS_RANDOM_COLOR: u32 = 1 << 7;
const MODE_MANUAL_SAVE: u32 = 1 << 8;
const MODE_AUTOMATIC_SAVE: u32 = 1 << 9;
const MODE_REQUIRES_ENTIRE_DEVICE: u32 = 1 << 10;
const MODE_HAS_DIRECTION_DIAGONAL: u32 = 1 << 11;
const COLOR_MODE_PER_LED: u32 = 1;
/// The server's SetCustomMode prefers a per-LED mode with exactly this name.
const DIRECT_MODE_NAME: &str = "Direct";

pub fn describe(data: &ControllerData) -> DeviceDescriptor {
    let zones: Vec<ZoneDescriptor> = data.zones.iter().map(describe_zone).collect();
    DeviceDescriptor {
        name: data.name.clone(),
        vendor: data.vendor.clone(),
        description: data.description.clone(),
        version: data.version.clone(),
        serial: data.serial.clone(),
        location: data.location.clone(),
        kind: device_kind(data.device_type),
        led_count: data.led_names.len(),
        capabilities: capabilities(data, &zones),
        zones,
    }
}

fn capabilities(data: &ControllerData, zones: &[ZoneDescriptor]) -> DeviceCapabilities {
    DeviceCapabilities {
        direct_mode: data.modes.iter().any(is_direct_mode),
        active_mode: (!data.modes.is_empty()).then_some(data.active_mode as usize),
        modes: data.modes.iter().map(describe_mode).collect(),
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
        saves_to_device_memory: data
            .modes
            .iter()
            .any(|mode| mode.flags & MODE_MANUAL_SAVE != 0),
    }
}

fn is_direct_mode(mode: &ModeData) -> bool {
    mode.name == DIRECT_MODE_NAME
        && mode.flags & MODE_HAS_PER_LED_COLOR != 0
        && mode.color_mode == COLOR_MODE_PER_LED
}

fn describe_mode(mode: &ModeData) -> ModeDescriptor {
    let has = |flag: u32| mode.flags & flag != 0;
    let range = |flag, min, max| has(flag).then_some(ValueRange { min, max });
    let directions = [
        (MODE_HAS_DIRECTION_LR, ModeDirection::LeftRight),
        (MODE_HAS_DIRECTION_UD, ModeDirection::UpDown),
        (MODE_HAS_DIRECTION_HV, ModeDirection::HorizontalVertical),
        (MODE_HAS_DIRECTION_DIAGONAL, ModeDirection::Diagonal),
    ];
    ModeDescriptor {
        name: mode.name.clone(),
        speed: range(MODE_HAS_SPEED, mode.speed_min, mode.speed_max),
        brightness: range(
            MODE_HAS_BRIGHTNESS,
            mode.brightness_min,
            mode.brightness_max,
        ),
        directions: directions
            .into_iter()
            .filter(|(flag, _)| has(*flag))
            .map(|(_, direction)| direction)
            .collect(),
        per_led_color: has(MODE_HAS_PER_LED_COLOR),
        mode_colors: range(
            MODE_HAS_MODE_SPECIFIC_COLOR,
            mode.colors_min,
            mode.colors_max,
        ),
        random_color: has(MODE_HAS_RANDOM_COLOR),
        applies_to_entire_device: has(MODE_REQUIRES_ENTIRE_DEVICE),
        persistence: if has(MODE_AUTOMATIC_SAVE) {
            ModePersistence::AutomaticSave
        } else if has(MODE_MANUAL_SAVE) {
            ModePersistence::ManualSave
        } else {
            ModePersistence::None
        },
    }
}

fn describe_zone(zone: &ZoneData) -> ZoneDescriptor {
    ZoneDescriptor {
        name: zone.name.clone(),
        kind: zone_kind(zone.zone_type),
        led_count: zone.leds_count as usize,
        leds_min: zone.leds_min,
        leds_max: zone.leds_max,
        matrix: zone.matrix.as_ref().map(|matrix| MatrixSize {
            width: matrix.width,
            height: matrix.height,
        }),
    }
}

fn zone_kind(code: i32) -> ZoneKind {
    match code {
        0 => ZoneKind::Single,
        1 => ZoneKind::Linear,
        2 => ZoneKind::Matrix,
        3 => ZoneKind::LinearLoop,
        4 => ZoneKind::MatrixLoopX,
        5 => ZoneKind::MatrixLoopY,
        6 => ZoneKind::Segmented,
        other => ZoneKind::Other(other),
    }
}

fn device_kind(code: i32) -> DeviceKind {
    const KINDS: [DeviceKind; 21] = [
        DeviceKind::Motherboard,
        DeviceKind::Memory,
        DeviceKind::Gpu,
        DeviceKind::Cooler,
        DeviceKind::LedStrip,
        DeviceKind::Keyboard,
        DeviceKind::Mouse,
        DeviceKind::Mousemat,
        DeviceKind::Headset,
        DeviceKind::HeadsetStand,
        DeviceKind::Gamepad,
        DeviceKind::Light,
        DeviceKind::Speaker,
        DeviceKind::Virtual,
        DeviceKind::Storage,
        DeviceKind::Case,
        DeviceKind::Microphone,
        DeviceKind::Accessory,
        DeviceKind::Keypad,
        DeviceKind::Laptop,
        DeviceKind::Monitor,
    ];
    usize::try_from(code)
        .ok()
        .and_then(|index| KINDS.get(index).copied())
        .unwrap_or(DeviceKind::Unknown)
}
