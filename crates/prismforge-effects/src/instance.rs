use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::color::Rgb;
use crate::control::{ControlValue, Direction, ValueProblem};
use crate::definition::EffectDefinition;

/// The stored form of an effect: its stable id and a value per control id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EffectInstance {
    pub effect: String,
    pub values: BTreeMap<String, ControlValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum InstanceError {
    #[error("there is no effect with id {effect}")]
    UnknownEffect { effect: String },
    #[error("{} effect values are not valid", errors.len())]
    InvalidValues { errors: Vec<ValueError> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ValueError {
    pub control: String,
    pub problem: ValueProblem,
}

/// Values checked against their definition, with every default filled in.
pub(crate) struct Values<'a>(&'a BTreeMap<String, ControlValue>);

impl EffectInstance {
    /// Checks every value and fills missing optional ones; errors come sorted by control id.
    pub(crate) fn complete(&self, definition: &EffectDefinition) -> Result<Self, Vec<ValueError>> {
        let mut errors = Vec::new();
        let mut values = BTreeMap::new();
        for control in &definition.controls {
            let value = match self.values.get(&control.id) {
                Some(value) => control.kind.check(value).map(|()| value.clone()),
                None => control.kind.default_value().ok_or(ValueProblem::Missing),
            };
            match value {
                Ok(value) => {
                    values.insert(control.id.clone(), value);
                }
                Err(problem) => errors.push(ValueError {
                    control: control.id.clone(),
                    problem,
                }),
            }
        }
        errors.extend(
            self.values
                .keys()
                .filter(|id| !definition.controls.iter().any(|control| &control.id == *id))
                .map(|id| ValueError {
                    control: id.clone(),
                    problem: ValueProblem::UnknownControl,
                }),
        );
        if errors.is_empty() {
            Ok(Self {
                effect: self.effect.clone(),
                values,
            })
        } else {
            errors.sort_by(|left, right| left.control.cmp(&right.control));
            Err(errors)
        }
    }

    pub(crate) fn checked(&self) -> Values<'_> {
        Values(&self.values)
    }
}

impl Values<'_> {
    pub fn color(&self, id: &str) -> Rgb {
        match self.0.get(id) {
            Some(ControlValue::Color(color)) => *color,
            _ => unreachable!("validated colour {id}"),
        }
    }

    pub fn palette(&self, id: &str) -> Vec<Rgb> {
        match self.0.get(id) {
            Some(ControlValue::Palette(colors)) => colors.clone(),
            _ => unreachable!("validated palette {id}"),
        }
    }

    pub fn number(&self, id: &str) -> u32 {
        match self.0.get(id) {
            Some(ControlValue::Number(number)) => *number,
            _ => unreachable!("validated number {id}"),
        }
    }

    pub fn direction(&self, id: &str) -> Direction {
        match self.0.get(id) {
            Some(ControlValue::Direction(direction)) => *direction,
            _ => unreachable!("validated direction {id}"),
        }
    }
}
