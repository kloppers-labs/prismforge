use std::time::Duration;

use crate::builtin::Program;
use crate::builtin::controls::BRIGHTNESS;
use crate::color::{Rgb, dimmed};
use crate::instance::EffectInstance;

/// A validated effect ready to render; every frame depends only on its inputs.
#[derive(Debug, Clone)]
pub struct Effect {
    instance: EffectInstance,
    program: Program,
    brightness: u32,
}

impl Effect {
    pub(crate) fn new(instance: EffectInstance, program: Program) -> Self {
        let brightness = instance.checked().number(BRIGHTNESS);
        Self {
            instance,
            program,
            brightness,
        }
    }

    /// The stored form, with every default filled in.
    pub fn instance(&self) -> &EffectInstance {
        &self.instance
    }

    /// True when the output never changes with time, so one write is enough.
    pub fn is_static(&self) -> bool {
        self.brightness == 0 || self.program.is_static()
    }

    /// No built-in effect varies with `seed`; it is part of the contract so random effects stay reproducible.
    pub fn color_at(&self, time: Duration, position: f64, _seed: u64) -> Rgb {
        let position = if position.is_nan() {
            0.0
        } else {
            position.clamp(0.0, 1.0)
        };
        let color = self.program.color_at(time.as_secs_f64(), position);
        dimmed(color, self.brightness)
    }

    pub fn render(&self, time: Duration, positions: &[f64], seed: u64) -> Vec<Rgb> {
        positions
            .iter()
            .map(|&position| self.color_at(time, position, seed))
            .collect()
    }
}
