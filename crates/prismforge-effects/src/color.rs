use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub(crate) const BLACK: Rgb = Rgb::new(0, 0, 0);

/// Moves from `from` towards `to` by `amount` (0.0 to 1.0), rounding each channel.
pub(crate) fn mix(from: Rgb, to: Rgb, amount: f64) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    let channel = |from: u8, to: u8| {
        let from = f64::from(from);
        (from + (f64::from(to) - from) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgb::new(
        channel(from.r, to.r),
        channel(from.g, to.g),
        channel(from.b, to.b),
    )
}

/// Fully saturated colour at `turns` around the colour wheel; 0.0 is red, 1/3 green, 2/3 blue.
pub(crate) fn hue(turns: f64) -> Rgb {
    let sector = turns.rem_euclid(1.0) * 6.0;
    let index = sector.floor();
    let rise = sector - index;
    let fall = 1.0 - rise;
    let (r, g, b) = match index as u8 % 6 {
        0 => (1.0, rise, 0.0),
        1 => (fall, 1.0, 0.0),
        2 => (0.0, 1.0, rise),
        3 => (0.0, fall, 1.0),
        4 => (rise, 0.0, 1.0),
        _ => (1.0, 0.0, fall),
    };
    let channel = |level: f64| (level * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgb::new(channel(r), channel(g), channel(b))
}

pub(crate) fn dimmed(color: Rgb, percent: u32) -> Rgb {
    let channel = |value: u8| ((u32::from(value) * percent + 50) / 100).min(255) as u8;
    Rgb::new(channel(color.r), channel(color.g), channel(color.b))
}
