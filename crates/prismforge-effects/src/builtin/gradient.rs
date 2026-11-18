use crate::builtin::blend;
use crate::builtin::controls::{self, DIRECTION, PALETTE};
use crate::color::Rgb;
use crate::control::Direction;
use crate::definition::EffectDefinition;
use crate::instance::Values;

#[derive(Debug, Clone)]
pub(crate) struct Gradient {
    palette: Vec<Rgb>,
    direction: Direction,
}

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition {
        id: "gradient".into(),
        label: "Gradient".into(),
        description: "Colours blend from the first LED to the last.".into(),
        controls: vec![
            controls::palette(
                "The first colour is at the start, the last at the end.",
                vec![Rgb::new(255, 0, 64), Rgb::new(0, 96, 255)],
            ),
            controls::direction("Forward starts at the first LED."),
            controls::brightness(),
        ],
    }
}

impl Gradient {
    pub fn new(values: &Values) -> Self {
        Self {
            palette: values.palette(PALETTE),
            direction: values.direction(DIRECTION),
        }
    }

    pub fn color_at(&self, position: f64) -> Rgb {
        let stops = (self.palette.len() - 1) as f64;
        blend(
            &self.palette,
            self.direction.orient(position) * stops,
            false,
        )
    }
}
