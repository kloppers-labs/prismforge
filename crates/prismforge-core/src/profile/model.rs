use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::device::{DeviceKind, DeviceRef, ZoneKind};
use crate::identity::{DeviceFingerprint, ZoneShape};

/// Fields from a newer minor schema version; kept and written back unchanged.
pub type Extra = Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ProfileId(String);

impl ProfileId {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Lower-case letters, digits and hyphens only, so the id is always a safe file name.
    pub fn parse(value: &str) -> Option<Self> {
        let valid = (1..=64).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        valid.then(|| Self(value.to_owned()))
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A saved scene with its ownership statements; `devices` holds what re-links each referenced device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub frame_rate: u32,
    pub devices: Vec<ProfileDevice>,
    pub assignments: Vec<ProfileAssignment>,
    pub modes: Vec<ModeSelection>,
    pub external: Vec<ExternalControl>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `reference` is a key inside the profile; matching to hardware uses only `identity`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileDevice {
    pub reference: DeviceRef,
    pub label: String,
    #[serde(flatten)]
    pub identity: StoredIdentity,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredIdentity {
    pub kind: DeviceKind,
    pub vendor: String,
    pub model: String,
    pub serial: Option<String>,
    pub location: Option<String>,
    pub zones: Vec<StoredZone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredZone {
    pub name: String,
    pub kind: ZoneKind,
    pub leds_min: u32,
    pub leds_max: u32,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `zone` is an index into the device's zone list, which must match exactly for the device to resolve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileAssignment {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    pub effect: StoredEffect,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The stored form of an effect instance; values stay raw JSON so a value this build cannot read survives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredEffect {
    pub effect: String,
    pub values: BTreeMap<String, Value>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A hardware mode selected through PrismForge; the name detects a changed mode list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeSelection {
    pub device: DeviceRef,
    pub index: usize,
    pub name: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The user's statement that another program controls the device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalControl {
    pub device: DeviceRef,
    pub label: String,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Profile {
    pub fn device(&self, reference: &DeviceRef) -> Option<&ProfileDevice> {
        self.devices
            .iter()
            .find(|device| &device.reference == reference)
    }
}

impl StoredIdentity {
    pub fn from_fingerprint(fingerprint: &DeviceFingerprint) -> Self {
        Self {
            kind: fingerprint.kind,
            vendor: fingerprint.vendor.clone(),
            model: fingerprint.model.clone(),
            serial: fingerprint.serial.clone(),
            location: fingerprint.location.clone(),
            zones: fingerprint
                .zones
                .iter()
                .map(|zone| StoredZone {
                    name: zone.name.clone(),
                    kind: zone.kind,
                    leds_min: zone.leds_min,
                    leds_max: zone.leds_max,
                    extra: Extra::new(),
                })
                .collect(),
        }
    }

    pub fn fingerprint(&self) -> DeviceFingerprint {
        DeviceFingerprint {
            kind: self.kind,
            vendor: self.vendor.clone(),
            model: self.model.clone(),
            serial: self.serial.clone(),
            location: self.location.clone(),
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

    /// True when the identity carries neither a serial nor a location, as in an exported profile.
    pub fn is_portable(&self) -> bool {
        self.serial.is_none() && self.location.is_none()
    }
}
