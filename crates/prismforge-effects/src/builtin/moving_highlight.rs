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
    width: f64,
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
                "Rounds along the LEDs per minute. 0 holds the band still on the first LED.",
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
            width: f64::from(values.number(WIDTH)) / 100.0,
        }
    }

    pub fn is_static(&self) -> bool {
        self.speed == 0
    }

    /// The band runs round the LEDs as a ring and is at least two LEDs wide, so one LED is always at least half lit.
    pub fn color_at(&self, seconds: f64, position: f64, leds: usize) -> Rgb {
        let ring = leds.max(1) as f64;
        let place = self.direction.orient(position) * (ring - 1.0);
        let centre = phase(seconds, self.speed) * ring;
        let gap = (place - centre).rem_euclid(ring);
        let distance = gap.min(ring - gap);
        let half_width = (self.width * ring / 2.0).max(1.0);
        mix(self.background, self.color, 1.0 - distance / half_width)
    }
}
