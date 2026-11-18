use serde::{Deserialize, Serialize};

use crate::device::{DeviceDescriptor, DeviceRef};

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
pub struct DeviceIssue {
    pub runtime_id: u32,
    pub reason: IssueReason,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn is_ambiguous(&self, reference: &DeviceRef) -> bool {
        self.devices.iter().any(|device| {
            matches!(&device.identity, DeviceIdentity::Ambiguous { candidates } if candidates.contains(reference))
        })
    }
}
