use crate::color::Rgb;
use crate::control::{Bounds, ControlDefinition, ControlKind, Direction, NumberUnit, SpeedUnit};

pub const COLOR: &str = "color";
pub const PALETTE: &str = "palette";
pub const SPEED: &str = "speed";
pub const DIRECTION: &str = "direction";
pub const BRIGHTNESS: &str = "brightness";

const PALETTE_COLORS: (u32, u32) = (2, 8);

fn control(id: &str, label: &str, description: &str, kind: ControlKind) -> ControlDefinition {
    ControlDefinition {
        id: id.into(),
        label: label.into(),
        description: description.into(),
        kind,
    }
}

pub fn color(id: &str, label: &str, description: &str, default: Option<Rgb>) -> ControlDefinition {
    control(id, label, description, ControlKind::Color { default })
}

pub fn palette(description: &str, default: Vec<Rgb>) -> ControlDefinition {
    control(
        PALETTE,
        "Colours",
        description,
        ControlKind::Palette {
            min_colors: PALETTE_COLORS.0,
            max_colors: PALETTE_COLORS.1,
            default,
        },
    )
}

pub fn speed(description: &str, max: u32, default: u32) -> ControlDefinition {
    control(
        SPEED,
        "Speed",
        description,
        ControlKind::Speed {
            range: Bounds { min: 0, max },
            default,
            unit: SpeedUnit::CyclesPerMinute,
        },
    )
}

pub fn direction(description: &str) -> ControlDefinition {
    control(
        DIRECTION,
        "Direction",
        description,
        ControlKind::Direction {
            default: Direction::Forward,
        },
    )
}

pub fn number(
    id: &str,
    label: &str,
    description: &str,
    range: Bounds,
    step: u32,
    default: u32,
    unit: NumberUnit,
) -> ControlDefinition {
    control(
        id,
        label,
        description,
        ControlKind::Number {
            range,
            step,
            default,
            unit,
        },
    )
}

pub fn brightness() -> ControlDefinition {
    control(
        BRIGHTNESS,
        "Brightness",
        "Percent of the effect's colours. 0 is off.",
        ControlKind::Brightness {
            range: Bounds { min: 0, max: 100 },
            default: 100,
        },
    )
}
