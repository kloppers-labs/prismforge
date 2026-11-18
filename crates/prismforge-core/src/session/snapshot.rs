use serde::{Deserialize, Serialize};

use crate::backend::BackendInfo;
use crate::connection::ConnectionState;
use crate::device::{DeviceDescriptor, DeviceRef};
use crate::inventory::{DeviceIdentity, DeviceIssue};
use crate::session::diagnostics::Diagnostics;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub connection: ConnectionState,
    pub backend: BackendInfo,
    pub devices: Vec<SessionDevice>,
    pub issues: Vec<DeviceIssue>,
    pub identify: Option<IdentifyTarget>,
    pub diagnostics: Diagnostics,
}

/// A device from the last inventory; `available` is false while it is missing or the session is not connected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDevice {
    pub identity: DeviceIdentity,
    pub descriptor: DeviceDescriptor,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifyTarget {
    pub device: DeviceRef,
    pub zone: Option<usize>,
}
