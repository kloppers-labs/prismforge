use prismforge_effects::EffectInstance;
use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;

/// A whole device, or one zone of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct OutputTarget {
    pub device: DeviceRef,
    pub zone: Option<usize>,
}

impl OutputTarget {
    /// True when both targets would write to at least one shared LED.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.device == other.device
            && (self.zone.is_none() || other.zone.is_none() || self.zone == other.zone)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Assignment {
    pub target: OutputTarget,
    pub effect: EffectInstance,
}

/// The effects PrismForge runs; no two assignments overlap.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Scene {
    pub assignments: Vec<Assignment>,
}
