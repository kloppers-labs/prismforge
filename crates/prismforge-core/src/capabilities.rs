use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceCapabilities {
    /// Index of the mode PrismForge treats as Direct Mode, if the device has one.
    pub direct_mode: Option<usize>,
    pub active_mode: Option<usize>,
    pub modes: Vec<ModeDescriptor>,
    pub resizable_zones: Vec<ZoneResizeBounds>,
    pub saves_to_device_memory: bool,
}

impl DeviceCapabilities {
    pub fn has_direct_mode(&self) -> bool {
        self.direct_mode.is_some()
    }

    pub fn in_direct_mode(&self) -> bool {
        self.direct_mode.is_some() && self.direct_mode == self.active_mode
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ZoneResizeBounds {
    pub zone: usize,
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ValueRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ModeDirection {
    LeftRight,
    UpDown,
    HorizontalVertical,
    Diagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ModePersistence {
    None,
    ManualSave,
    AutomaticSave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModeDescriptor {
    pub name: String,
    pub speed: Option<ValueRange>,
    pub brightness: Option<ValueRange>,
    pub directions: Vec<ModeDirection>,
    pub per_led_color: bool,
    pub mode_colors: Option<ValueRange>,
    pub random_color: bool,
    pub applies_to_entire_device: bool,
    pub persistence: ModePersistence,
}
