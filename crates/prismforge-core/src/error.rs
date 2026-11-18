use serde::{Deserialize, Serialize};

use crate::connection::LossCause;
use crate::device::DeviceRef;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum BackendError {
    #[error("backend is not connected")]
    NotConnected,
    #[error("server unreachable: {detail}")]
    Unreachable { detail: String },
    #[error("server protocol {server_version:?} is below required protocol {required}")]
    UnsupportedProtocol {
        server_version: Option<u32>,
        required: u32,
    },
    #[error("{operation} timed out after {timeout_ms} ms")]
    Timeout { operation: String, timeout_ms: u64 },
    #[error("connection lost ({cause:?}): {detail}")]
    ConnectionLost { cause: LossCause, detail: String },
    #[error("protocol violation: {detail}")]
    ProtocolViolation { detail: String },
    #[error("server rejected {operation} with status {status}")]
    Rejected { operation: String, status: u32 },
    #[error("inventory is stale; refresh required")]
    StaleInventory,
    #[error("device {device} is not known")]
    UnknownDevice { device: DeviceRef },
    #[error("device {device} is missing from the current inventory")]
    DeviceMissing { device: DeviceRef },
    #[error("device {device} cannot be told apart from another device")]
    AmbiguousDevice { device: DeviceRef },
    #[error("device {device} has no direct mode")]
    DirectModeUnavailable { device: DeviceRef },
    #[error("zone {zone} is out of range for device {device} with {zone_count} zones")]
    ZoneOutOfRange {
        device: DeviceRef,
        zone: usize,
        zone_count: usize,
    },
    #[error("mode {mode} is out of range for device {device} with {mode_count} modes")]
    ModeOutOfRange {
        device: DeviceRef,
        mode: usize,
        mode_count: usize,
    },
    #[error("{actual} colours do not match {expected} LEDs on device {device}")]
    ColorCountMismatch {
        device: DeviceRef,
        expected: usize,
        actual: usize,
    },
}
