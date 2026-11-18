use crate::builtin::controls::{self, COLOR, DIRECTION, SPEED};
use crate::builtin::phase;
use crate::color::{Rgb, mix};
use crate::control::{Bounds, Direction, NumberUnit};
use crate::definition::EffectDefinition;
use crate::instance::Values;

const BACKGROUND: &str = "background";
const WIDTH: &str = "width";

#[derive(Debug, Clone)]
pub(crate) struct MovingHighlight {
    color: Rgb,
    background: Rgb,
    speed: u32,
    direction: Direction,
    half_width: f64,
}

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition {
        id: "moving-highlight".into(),
        label: "Moving highlight".into(),
        description: "A bright band runs along the LEDs over a background colour.".into(),
        controls: vec![
            controls::color(
                COLOR,
                "Highlight",
                "The colour at the centre of the band.",
                Some(Rgb::new(255, 255, 255)),
            ),
            controls::color(
                BACKGROUND,
                "Background",
                "The colour outside the band.",
                Some(Rgb::new(0, 0, 0)),
            ),
            controls::speed(
                "Runs from one end to the other per minute. 0 keeps the band out of sight.",
                240,
                30,
            ),
            controls::direction("Forward runs from the first LED towards the last."),
            controls::number(
                WIDTH,
                "Width",
                "Width of the band as a percent of the LEDs.",
                Bounds { min: 5, max: 100 },
                5,
                20,
                NumberUnit::Percent,
            ),
            controls::brightness(),
        ],
    }
}

impl MovingHighlight {
    pub fn new(values: &Values) -> Self {
        Self {
            color: values.color(COLOR),
            background: values.color(BACKGROUND),
            speed: values.number(SPEED),
            direction: values.direction(DIRECTION),
            half_width: f64::from(values.number(WIDTH)) / 200.0,
        }
    }

    pub fn is_static(&self) -> bool {
        self.speed == 0
    }

    /// The band enters before the first LED and leaves after the last, so each run starts dark.
    pub fn color_at(&self, seconds: f64, position: f64) -> Rgb {
        let travel = 1.0 + 2.0 * self.half_width;
        let centre = phase(seconds, self.speed) * travel - self.half_width;
        let distance = (self.direction.orient(position) - centre).abs();
        mix(
            self.background,
            self.color,
            1.0 - distance / self.half_width,
        )
    }
}
