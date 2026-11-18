use crate::builtin::controls::{self, DIRECTION, SPEED};
use crate::builtin::phase;
use crate::color::{Rgb, hue};
use crate::control::{Bounds, Direction, NumberUnit};
use crate::definition::EffectDefinition;
use crate::instance::Values;

const WAVES: &str = "waves";

#[derive(Debug, Clone)]
pub(crate) struct RainbowWave {
    speed: u32,
    direction: Direction,
    waves: u32,
}

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition {
        id: "rainbow-wave".into(),
        label: "Rainbow wave".into(),
        description: "The colour wheel travels along the LEDs.".into(),
        controls: vec![
            controls::speed(
                "How often each LED passes through every colour per minute. 0 stops the wave.",
                240,
                20,
            ),
            controls::direction("Forward moves from the first LED towards the last."),
            controls::number(
                WAVES,
                "Rainbows",
                "How many full rainbows fit along the LEDs.",
                Bounds { min: 1, max: 8 },
                1,
                1,
                NumberUnit::Count,
            ),
            controls::brightness(),
        ],
    }
}

impl RainbowWave {
    pub fn new(values: &Values) -> Self {
        Self {
            speed: values.number(SPEED),
            direction: values.direction(DIRECTION),
            waves: values.number(WAVES),
        }
    }

    pub fn is_static(&self) -> bool {
        self.speed == 0
    }

    pub fn color_at(&self, seconds: f64, position: f64) -> Rgb {
        let along = self.direction.orient(position) * f64::from(self.waves);
        hue(along - phase(seconds, self.speed))
    }
}
