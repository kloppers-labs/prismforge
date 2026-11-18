mod backend;
mod capabilities;
mod color;
mod connection;
mod device;
mod error;
mod identity;
mod inventory;

pub use backend::{BackendEvent, RgbBackend};
pub use capabilities::{
    DeviceCapabilities, ModeDescriptor, ModeDirection, ModePersistence, ValueRange,
    ZoneResizeBounds,
};
pub use color::Rgb;
pub use connection::{ConnectionState, StateReason};
pub use device::{DeviceDescriptor, DeviceKind, DeviceRef, MatrixSize, ZoneDescriptor, ZoneKind};
pub use error::BackendError;
pub use identity::{
    AmbiguousGroup, DeviceFingerprint, IdentityMatch, KnownDevice, Reconciliation, ZoneShape,
    reconcile,
};
pub use inventory::{DeviceIdentity, DeviceIssue, Inventory, InventoryDevice, IssueReason};
