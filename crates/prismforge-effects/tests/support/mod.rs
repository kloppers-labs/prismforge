#![allow(dead_code)]

use std::time::Duration;

use prismforge_effects::{ControlValue, Effect, EffectInstance, Rgb, linear_positions, prepare};

pub const RED: Rgb = Rgb::new(255, 0, 0);
pub const GREEN: Rgb = Rgb::new(0, 255, 0);
pub const BLUE: Rgb = Rgb::new(0, 0, 255);
pub const WHITE: Rgb = Rgb::new(255, 255, 255);
pub const BLACK: Rgb = Rgb::new(0, 0, 0);
pub const SEED: u64 = 7;

pub fn instance(effect: &str, values: &[(&str, ControlValue)]) -> EffectInstance {
    EffectInstance {
        effect: effect.into(),
        values: values
            .iter()
            .map(|(id, value)| ((*id).to_owned(), value.clone()))
            .collect(),
    }
}

pub fn effect(effect: &str, values: &[(&str, ControlValue)]) -> Effect {
    prepare(&instance(effect, values)).expect("valid instance")
}

pub fn frame(effect: &Effect, millis: u64, leds: usize) -> Vec<Rgb> {
    effect.render(Duration::from_millis(millis), &linear_positions(leds), SEED)
}

pub fn number(value: u32) -> ControlValue {
    ControlValue::Number(value)
}

pub fn color(value: Rgb) -> ControlValue {
    ControlValue::Color(value)
}

pub fn palette(colors: &[Rgb]) -> ControlValue {
    ControlValue::Palette(colors.to_vec())
}

/// A valid instance of every built-in effect, animated where the effect can animate.
pub fn every_effect(extra: &[(&str, ControlValue)]) -> Vec<Effect> {
    prismforge_effects::definitions()
        .iter()
        .map(|definition| {
            let mut values = extra.to_vec();
            if definition.id == "solid" {
                values.push(("color", color(Rgb::new(200, 100, 50))));
            }
            effect(&definition.id, &values)
        })
        .collect()
}
