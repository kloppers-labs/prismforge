use serde::{Deserialize, Serialize};

/// Who controls a device's lighting, as far as PrismForge knows; a device without an owner is not listed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum OutputOwner {
    Effect,
    HardwareMode {
        mode: usize,
    },
    /// Only the user's statement; PrismForge cannot see other writers.
    External {
        label: String,
    },
}
