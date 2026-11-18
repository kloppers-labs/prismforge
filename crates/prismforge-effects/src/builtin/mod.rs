mod breathing;
mod color_cycle;
pub(crate) mod controls;
mod gradient;
mod moving_highlight;
mod rainbow_wave;
mod solid;

use crate::color::{Rgb, mix};
use crate::definition::EffectDefinition;
use crate::instance::Values;

use breathing::Breathing;
use color_cycle::ColorCycle;
use gradient::Gradient;
use moving_highlight::MovingHighlight;
use rainbow_wave::RainbowWave;
use solid::Solid;

#[derive(Debug, Clone)]
pub(crate) enum Program {
    Solid(Solid),
    Breathing(Breathing),
    Gradient(Gradient),
    RainbowWave(RainbowWave),
    ColorCycle(ColorCycle),
    MovingHighlight(MovingHighlight),
}

pub(crate) struct Builtin {
    pub definition: fn() -> EffectDefinition,
    pub build: fn(&Values) -> Program,
}

pub(crate) const BUILTINS: [Builtin; 6] = [
    Builtin {
        definition: solid::definition,
        build: |values| Program::Solid(Solid::new(values)),
    },
    Builtin {
        definition: breathing::definition,
        build: |values| Program::Breathing(Breathing::new(values)),
    },
    Builtin {
        definition: gradient::definition,
        build: |values| Program::Gradient(Gradient::new(values)),
    },
    Builtin {
        definition: rainbow_wave::definition,
        build: |values| Program::RainbowWave(RainbowWave::new(values)),
    },
    Builtin {
        definition: color_cycle::definition,
        build: |values| Program::ColorCycle(ColorCycle::new(values)),
    },
    Builtin {
        definition: moving_highlight::definition,
        build: |values| Program::MovingHighlight(MovingHighlight::new(values)),
    },
];

impl Program {
    pub fn is_static(&self) -> bool {
        match self {
            Self::Solid(_) | Self::Gradient(_) => true,
            Self::Breathing(effect) => effect.is_static(),
            Self::RainbowWave(effect) => effect.is_static(),
            Self::ColorCycle(effect) => effect.is_static(),
            Self::MovingHighlight(effect) => effect.is_static(),
        }
    }

    pub fn color_at(&self, seconds: f64, position: f64) -> Rgb {
        match self {
            Self::Solid(effect) => effect.color_at(),
            Self::Breathing(effect) => effect.color_at(seconds),
            Self::Gradient(effect) => effect.color_at(position),
            Self::RainbowWave(effect) => effect.color_at(seconds, position),
            Self::ColorCycle(effect) => effect.color_at(seconds),
            Self::MovingHighlight(effect) => effect.color_at(seconds, position),
        }
    }
}

/// Fraction of the current cycle at `per_minute` cycles per minute.
fn phase(seconds: f64, per_minute: u32) -> f64 {
    (seconds * f64::from(per_minute) / 60.0).rem_euclid(1.0)
}

/// Colour at `stop` along `palette` (at least two colours); with `wrap` the last colour blends into the first.
fn blend(palette: &[Rgb], stop: f64, wrap: bool) -> Rgb {
    let count = palette.len();
    let whole = stop.floor();
    if wrap {
        let index = whole as usize % count;
        mix(palette[index], palette[(index + 1) % count], stop - whole)
    } else {
        let index = (whole as usize).min(count - 2);
        mix(palette[index], palette[index + 1], stop - index as f64)
    }
}
