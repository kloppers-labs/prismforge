use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub direct_mode: bool,
    pub active_mode: Option<usize>,
    pub modes: Vec<ModeDescriptor>,
    pub resizable_zones: Vec<ZoneResizeBounds>,
    pub saves_to_device_memory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneResizeBounds {
    pub zone: usize,
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeDirection {
    LeftRight,
    UpDown,
    HorizontalVertical,
    Diagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModePersistence {
    None,
    ManualSave,
    AutomaticSave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
