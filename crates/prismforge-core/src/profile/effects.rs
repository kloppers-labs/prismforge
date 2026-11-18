use std::collections::BTreeMap;

use prismforge_effects::{
    ControlValue, EffectInstance, InstanceError, ValueError, ValueProblem, prepare,
};
use serde::{Deserialize, Serialize};

use crate::profile::model::{Extra, StoredEffect};

/// Why this build cannot run a stored effect; the stored effect itself is kept unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum EffectProblem {
    /// This version of PrismForge has no effect with this id.
    EffectNotAvailable {
        effect: String,
    },
    /// These controls, or their kind of value, do not exist in this version.
    ControlsNotAvailable {
        controls: Vec<String>,
    },
    InvalidValues {
        errors: Vec<ValueError>,
    },
}

impl StoredEffect {
    pub fn from_instance(instance: &EffectInstance) -> Self {
        Self {
            effect: instance.effect.clone(),
            values: instance
                .values
                .iter()
                .map(|(control, value)| {
                    let value = serde_json::to_value(value).expect("control values are plain data");
                    (control.clone(), value)
                })
                .collect(),
            extra: Extra::new(),
        }
    }

    /// The instance this build can run, checked against its effect definition.
    pub fn instance(&self) -> Result<EffectInstance, EffectProblem> {
        let known = prismforge_effects::definitions()
            .iter()
            .any(|definition| definition.id == self.effect);
        if !known {
            return Err(self.not_available());
        }
        let mut values = BTreeMap::new();
        let mut unreadable = Vec::new();
        for (control, value) in &self.values {
            match serde_json::from_value::<ControlValue>(value.clone()) {
                Ok(value) => {
                    values.insert(control.clone(), value);
                }
                Err(_) => unreadable.push(control.clone()),
            }
        }
        if !unreadable.is_empty() {
            return Err(EffectProblem::ControlsNotAvailable {
                controls: unreadable,
            });
        }
        let instance = EffectInstance {
            effect: self.effect.clone(),
            values,
        };
        match prepare(&instance) {
            Ok(_) => Ok(instance),
            Err(InstanceError::UnknownEffect { .. }) => Err(self.not_available()),
            Err(InstanceError::InvalidValues { errors }) => Err(problem_of(errors)),
        }
    }

    fn not_available(&self) -> EffectProblem {
        EffectProblem::EffectNotAvailable {
            effect: self.effect.clone(),
        }
    }
}

fn problem_of(errors: Vec<ValueError>) -> EffectProblem {
    if errors
        .iter()
        .all(|error| error.problem == ValueProblem::UnknownControl)
    {
        EffectProblem::ControlsNotAvailable {
            controls: errors.into_iter().map(|error| error.control).collect(),
        }
    } else {
        EffectProblem::InvalidValues { errors }
    }
}
