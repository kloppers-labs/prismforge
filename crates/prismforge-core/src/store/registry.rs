use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::identity::KnownDevice;
use crate::profile::{Extra, StoredIdentity, clean_name};
use crate::store::envelope::FileProblem;

/// Every device PrismForge gave a reference, with the user's names for it and its zones.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Registry {
    pub devices: Vec<RegistryDevice>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryDevice {
    pub reference: DeviceRef,
    #[serde(flatten)]
    pub identity: StoredIdentity,
    pub name: Option<String>,
    pub zone_names: Vec<ZoneName>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneName {
    pub zone: usize,
    pub name: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The user's names for one device and its zones, for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceNames {
    pub device: DeviceRef,
    pub name: Option<String>,
    pub zones: Vec<ZoneLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ZoneLabel {
    pub zone: usize,
    pub name: String,
}

impl Registry {
    pub fn names(&self) -> Vec<DeviceNames> {
        self.devices
            .iter()
            .filter(|device| device.name.is_some() || !device.zone_names.is_empty())
            .map(|device| DeviceNames {
                device: device.reference.clone(),
                name: device.name.clone(),
                zones: device
                    .zone_names
                    .iter()
                    .map(|zone| ZoneLabel {
                        zone: zone.zone,
                        name: zone.name.clone(),
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn known_devices(&self) -> Vec<KnownDevice> {
        self.devices
            .iter()
            .map(|device| KnownDevice {
                reference: device.reference.clone(),
                fingerprint: device.identity.fingerprint(),
            })
            .collect()
    }

    pub fn device(&self, reference: &DeviceRef) -> Option<&RegistryDevice> {
        self.devices
            .iter()
            .find(|device| &device.reference == reference)
    }

    pub(crate) fn device_mut(&mut self, reference: &DeviceRef) -> Option<&mut RegistryDevice> {
        self.devices
            .iter_mut()
            .find(|device| &device.reference == reference)
    }

    /// Adds or updates the identifying fields; names and devices the backend no longer lists stay.
    pub(crate) fn record(&mut self, known: &[KnownDevice]) -> bool {
        let mut changed = false;
        for device in known {
            let identity = StoredIdentity::from_fingerprint(&device.fingerprint);
            match self.device_mut(&device.reference) {
                Some(entry) if entry.identity.fingerprint() == device.fingerprint => {}
                Some(entry) => {
                    entry.identity = entry.identity.refreshed(&device.fingerprint);
                    changed = true;
                }
                None => {
                    self.devices.push(RegistryDevice {
                        reference: device.reference.clone(),
                        identity,
                        name: None,
                        zone_names: Vec::new(),
                        extra: Extra::new(),
                    });
                    changed = true;
                }
            }
        }
        changed
    }

    pub(crate) fn validate(&self) -> Result<(), FileProblem> {
        let mut references = BTreeSet::new();
        for device in &self.devices {
            if !references.insert(&device.reference) {
                return Err(FileProblem::invalid(format!(
                    "device {} is listed twice",
                    device.reference
                )));
            }
            let names = device
                .name
                .iter()
                .chain(device.zone_names.iter().map(|zone| &zone.name));
            for name in names {
                if clean_name(name).as_ref() != Ok(name) {
                    return Err(FileProblem::invalid(format!(
                        "device {} has a name that is not valid",
                        device.reference
                    )));
                }
            }
            let mut zones = BTreeSet::new();
            for zone in &device.zone_names {
                if zone.zone >= device.identity.zones.len() || !zones.insert(zone.zone) {
                    return Err(FileProblem::invalid(format!(
                        "device {} names zone {} that does not exist or twice",
                        device.reference, zone.zone
                    )));
                }
            }
        }
        Ok(())
    }
}

impl RegistryDevice {
    pub fn zone_name(&self, zone: usize) -> Option<&str> {
        self.zone_names
            .iter()
            .find(|entry| entry.zone == zone)
            .map(|entry| entry.name.as_str())
    }
}
