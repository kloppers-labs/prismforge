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

    /// Colours for `positions`, which together form one target; the seed is reserved for random effects.
    pub fn render(&self, time: Duration, positions: &[f64], _seed: u64) -> Vec<Rgb> {
        let seconds = time.as_secs_f64();
        let leds = positions.len();
        positions
            .iter()
            .map(|&position| {
                let position = if position.is_nan() {
                    0.0
                } else {
                    position.clamp(0.0, 1.0)
                };
                dimmed(
                    self.program.color_at(seconds, position, leds),
                    self.brightness,
                )
            })
            .collect()
    }
}
