use serde::{Deserialize, Serialize};

use crate::backend::BackendInfo;
use crate::connection::ConnectionState;
use crate::device::{DeviceDescriptor, DeviceRef};
use crate::error::BackendError;
use crate::inventory::{DeviceIdentity, DeviceIssue};
use crate::ownership::OutputOwner;
use crate::scene::Scene;
use crate::session::diagnostics::Diagnostics;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SessionSnapshot {
    pub connection: ConnectionState,
    pub backend: BackendInfo,
    pub devices: Vec<SessionDevice>,
    pub issues: Vec<DeviceIssue>,
    pub identify: Option<IdentifyTarget>,
    pub output: OutputStatus,
    pub diagnostics: Diagnostics,
}

/// A device from the last inventory; `available` is false while it is missing or the session is not connected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SessionDevice {
    pub identity: DeviceIdentity,
    pub descriptor: DeviceDescriptor,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct IdentifyTarget {
    pub device: DeviceRef,
    pub zone: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct OutputStatus {
    pub frame_rate: u32,
    pub scene: Scene,
    /// Every device with an owner; a device that is not listed has none.
    pub devices: Vec<DeviceOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceOutput {
    pub device: DeviceRef,
    pub owner: OutputOwner,
    pub condition: OutputCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum OutputCondition {
    /// PrismForge writes nothing to this device: its owner is not a PrismForge effect.
    Idle,
    Running,
    Paused {
        reason: PauseReason,
    },
    /// The last write was refused; the next try waits longer after each refusal.
    Refused {
        error: BackendError,
        attempts: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum PauseReason {
    NotConnected,
    InventoryStale,
    Identify,
    /// The device is ambiguous, missing or changed shape; output resumes after an inventory that resolves it.
    Unavailable {
        error: BackendError,
    },
}
