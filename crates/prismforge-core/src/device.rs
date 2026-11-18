use std::fmt;

use serde::{Deserialize, Serialize};

use crate::capabilities::DeviceCapabilities;
use crate::color::Rgb;
use crate::identity::{DeviceFingerprint, ZoneShape};

/// Persistent PrismForge reference to a physical device; never a backend runtime ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceRef(String);

impl DeviceRef {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum BackendOrigin {
    OpenRgb,
    Simulated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum DeviceKind {
    Motherboard,
    Memory,
    Gpu,
    Cooler,
    LedStrip,
    Keyboard,
    Mouse,
    Mousemat,
    Headset,
    HeadsetStand,
    Gamepad,
    Light,
    Speaker,
    Virtual,
    Storage,
    Case,
    Microphone,
    Accessory,
    Keypad,
    Laptop,
    Monitor,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ZoneKind {
    Single,
    Linear,
    Matrix,
    LinearLoop,
    MatrixLoopX,
    MatrixLoopY,
    Segmented,
    Other(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MatrixSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ZoneDescriptor {
    pub name: String,
    pub kind: ZoneKind,
    pub led_count: usize,
    pub leds_min: u32,
    pub leds_max: u32,
    pub matrix: Option<MatrixSize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceDescriptor {
    pub origin: BackendOrigin,
    pub name: String,
    pub vendor: String,
    pub description: String,
    pub version: String,
    pub serial: String,
    pub location: String,
    pub kind: DeviceKind,
    pub zones: Vec<ZoneDescriptor>,
    pub led_count: usize,
    /// LED colours as the backend last reported them, one per LED.
    pub colors: Vec<Rgb>,
    pub capabilities: DeviceCapabilities,
}

impl DeviceDescriptor {
    pub fn fingerprint(&self) -> DeviceFingerprint {
        DeviceFingerprint {
            kind: self.kind,
            vendor: self.vendor.trim().to_owned(),
            model: self.name.trim().to_owned(),
            serial: non_empty(&self.serial),
            location: non_empty(&self.location),
            zones: self
                .zones
                .iter()
                .map(|zone| ZoneShape {
                    name: zone.name.clone(),
                    kind: zone.kind,
                    leds_min: zone.leds_min,
                    leds_max: zone.leds_max,
                })
                .collect(),
        }
    }
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
