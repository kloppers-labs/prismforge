use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::profile::apply::ProfileItem;

/// One stored setting an update adds, replaces or removes; every other setting stays as it was.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SettingChange {
    pub device: DeviceRef,
    pub label: String,
    pub change: Change,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Change {
    Added { now: ProfileItem },
    Replaced { was: ProfileItem, now: ProfileItem },
    Removed { was: ProfileItem },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FrameRateChange {
    pub was: u32,
    pub now: u32,
}
