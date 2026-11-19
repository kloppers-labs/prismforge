use std::time::Duration;

use crate::builtin::Program;
use crate::builtin::controls::BRIGHTNESS;
use crate::color::{BLACK, Rgb, dimmed};
use crate::instance::EffectInstance;
use crate::plane::{Flow, Plane, Projection, ScenePoint};
use crate::position::linear_positions;

const STRIP_SEED: u64 = 0;

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

    /// One device or zone with `led_count` LEDs in a line, as the session writes it and the preview shows it.
    pub fn render_strip(&self, time: Duration, led_count: usize) -> Vec<Rgb> {
        self.render(time, &linear_positions(led_count), STRIP_SEED)
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

    /// Colours for LEDs placed across a setup, one picture for every target; an LED without a place stays black.
    pub fn render_across(
        &self,
        time: Duration,
        flow: Flow,
        plane: Plane,
        points: &[Option<ScenePoint>],
        _seed: u64,
    ) -> Vec<Rgb> {
        let seconds = time.as_secs_f64();
        let projection = Projection::new(flow, plane);
        let pitch = projection.share(plane.pitch);
        points
            .iter()
            .map(|point| match point {
                Some(point) => dimmed(
                    self.program
                        .color_along(seconds, projection.position(*point), pitch),
                    self.brightness,
                ),
                None => BLACK,
            })
            .collect()
    }

    /// True when the colour depends on where the LED is, so a flow across a setup changes the picture.
    pub fn follows_position(&self) -> bool {
        self.program.follows_position()
    }
}
