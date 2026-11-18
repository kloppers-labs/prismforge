use crate::builtin::controls::{self, PALETTE, SPEED};
use crate::builtin::{blend, phase};
use crate::color::Rgb;
use crate::definition::EffectDefinition;
use crate::instance::Values;

#[derive(Debug, Clone)]
pub(crate) struct ColorCycle {
    palette: Vec<Rgb>,
    speed: u32,
}

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition {
        id: "color-cycle".into(),
        label: "Colour cycle".into(),
        description: "All LEDs blend through the colours in turn, then start again.".into(),
        controls: vec![
            controls::palette(
                "The colours in the order they appear.",
                vec![
                    Rgb::new(255, 0, 0),
                    Rgb::new(0, 255, 0),
                    Rgb::new(0, 0, 255),
                ],
            ),
            controls::speed(
                "Full rounds through all colours per minute. 0 holds the first colour.",
                120,
                6,
            ),
            controls::brightness(),
        ],
    }
}

impl ColorCycle {
    pub fn new(values: &Values) -> Self {
        Self {
            palette: values.palette(PALETTE),
            speed: values.number(SPEED),
        }
    }

    pub fn is_static(&self) -> bool {
        self.speed == 0
    }

    pub fn color_at(&self, seconds: f64) -> Rgb {
        let stops = self.palette.len() as f64;
        blend(&self.palette, phase(seconds, self.speed) * stops, true)
    }
}
