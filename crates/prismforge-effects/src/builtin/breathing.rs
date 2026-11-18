use std::f64::consts::TAU;

use crate::builtin::controls::{self, COLOR, SPEED};
use crate::builtin::phase;
use crate::color::{BLACK, Rgb, mix};
use crate::definition::EffectDefinition;
use crate::instance::Values;

#[derive(Debug, Clone)]
pub(crate) struct Breathing {
    color: Rgb,
    speed: u32,
}

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition {
        id: "breathing".into(),
        label: "Breathing".into(),
        description: "One colour fades out and back in.".into(),
        controls: vec![
            controls::color(
                COLOR,
                "Colour",
                "The colour at full strength.",
                Some(Rgb::new(255, 96, 0)),
            ),
            controls::speed("Breaths per minute. 0 holds the full colour.", 120, 15),
            controls::brightness(),
        ],
    }
}

impl Breathing {
    pub fn new(values: &Values) -> Self {
        Self {
            color: values.color(COLOR),
            speed: values.number(SPEED),
        }
    }

    pub fn is_static(&self) -> bool {
        self.speed == 0
    }

    pub fn color_at(&self, seconds: f64) -> Rgb {
        let level = 0.5 + 0.5 * (TAU * phase(seconds, self.speed)).cos();
        mix(BLACK, self.color, level)
    }
}
