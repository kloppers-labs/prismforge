use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::profile::apply::{ProfileItem, SkipReason};

/// What an update did to one stored or captured setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SettingChange {
    pub device: DeviceRef,
    pub label: String,
    pub item: ProfileItem,
    pub change: Change,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Change {
    /// The session runs exactly this setting; the stored one is kept as it was.
    Unchanged,
    /// The session runs something new that the profile did not have.
    Added,
    /// The user put another state on this target in this session.
    Replaced,
    /// It ran from this profile in this session and the user ended it.
    Removed,
    Kept {
        reason: KeepReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum KeepReason {
    Missing,
    Ambiguous,
    Unavailable,
    /// It is not running and the user did nothing to its target; `skipped` is why apply skipped it, if it did.
    NotRunning {
        skipped: Option<SkipReason>,
    },
}
