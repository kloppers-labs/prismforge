use serde::{Deserialize, Serialize};

use crate::device::{DeviceDescriptor, DeviceRef};
use crate::error::BackendError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub generation: u64,
    pub devices: Vec<InventoryDevice>,
    pub missing: Vec<DeviceRef>,
    pub issues: Vec<DeviceIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryDevice {
    pub identity: DeviceIdentity,
    pub descriptor: DeviceDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum DeviceIdentity {
    Matched(DeviceRef),
    New(DeviceRef),
    Ambiguous { candidates: Vec<DeviceRef> },
}

impl DeviceIdentity {
    pub fn reference(&self) -> Option<&DeviceRef> {
        match self {
            Self::Matched(reference) | Self::New(reference) => Some(reference),
            Self::Ambiguous { .. } => None,
        }
    }
}

/// A controller the backend reported but could not accept; other devices stay usable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceIssue {
    pub runtime_id: u32,
    pub reason: IssueReason,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum IssueReason {
    MalformedData,
    InconsistentData,
}

impl Inventory {
    pub fn device(&self, reference: &DeviceRef) -> Option<&InventoryDevice> {
        self.devices
            .iter()
            .find(|device| device.identity.reference() == Some(reference))
    }

    /// Position of `reference`; an ambiguous, missing or unknown reference is refused.
    pub fn locate(&self, reference: &DeviceRef) -> Result<usize, BackendError> {
        if let Some(index) = self
            .devices
            .iter()
            .position(|device| device.identity.reference() == Some(reference))
        {
            return Ok(index);
        }
        let device = reference.clone();
        Err(if self.is_ambiguous(&device) {
            BackendError::AmbiguousDevice { device }
        } else if self.missing.contains(&device) {
            BackendError::DeviceMissing { device }
        } else {
            BackendError::UnknownDevice { device }
        })
    }

    pub fn output_target(
        &self,
        reference: &DeviceRef,
        zone: Option<usize>,
    ) -> Result<&InventoryDevice, BackendError> {
        let device = &self.devices[self.locate(reference)?];
        let descriptor = &device.descriptor;
        if !descriptor.capabilities.has_direct_mode() {
            return Err(BackendError::DirectModeUnavailable {
                device: reference.clone(),
            });
        }
        if let Some(zone) = zone.filter(|&zone| zone >= descriptor.zones.len()) {
            return Err(BackendError::ZoneOutOfRange {
                device: reference.clone(),
                zone,
                zone_count: descriptor.zones.len(),
            });
        }
        Ok(device)
    }

    pub fn is_ambiguous(&self, reference: &DeviceRef) -> bool {
        self.devices.iter().any(|device| {
            matches!(&device.identity, DeviceIdentity::Ambiguous { candidates } if candidates.contains(reference))
        })
    }
}
