use prismforge_effects::{EffectInstance, Flow};
use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::setup::ViewId;

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
    pub mode: ApplyMode,
}

/// How an effect uses the LEDs of its targets: each target on its own, or one picture across a setup view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ApplyMode {
    EachDevice,
    AcrossSetup { view: ViewId, flow: Flow },
}

/// The effects PrismForge runs; no two assignments overlap.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Scene {
    pub assignments: Vec<Assignment>,
}
