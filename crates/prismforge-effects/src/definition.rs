use serde::{Deserialize, Serialize};

use crate::control::ControlDefinition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EffectDefinition {
    pub id: String,
    pub label: String,
    pub description: String,
    pub controls: Vec<ControlDefinition>,
}
