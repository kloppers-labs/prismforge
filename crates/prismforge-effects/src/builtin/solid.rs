use crate::builtin::controls::{self, COLOR};
use crate::color::Rgb;
use crate::definition::EffectDefinition;
use crate::instance::Values;

#[derive(Debug, Clone)]
pub(crate) struct Solid {
    color: Rgb,
}

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition {
        id: "solid".into(),
        label: "Solid colour".into(),
        description: "Every LED shows one colour.".into(),
        controls: vec![
            controls::color(COLOR, "Colour", "The colour of every LED.", None),
            controls::brightness(),
        ],
    }
}

impl Solid {
    pub fn new(values: &Values) -> Self {
        Self {
            color: values.color(COLOR),
        }
    }

    pub fn color_at(&self) -> Rgb {
        self.color
    }
}
