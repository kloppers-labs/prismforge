pub mod automation;
mod backend;
mod capabilities;
mod connection;
mod device;
mod error;
mod identity;
mod inventory;
mod ownership;
pub mod profile;
mod resolution;
mod scene;
pub mod session;
pub mod setup;
pub mod store;

pub use backend::{BackendEvent, BackendInfo, RgbBackend};
pub use capabilities::{
    DeviceCapabilities, ModeDescriptor, ModeDirection, ModePersistence, ValueRange,
    ZoneResizeBounds,
};
pub use connection::{ConnectionState, LossCause, StateReason};
pub use device::{
    BackendOrigin, DeviceDescriptor, DeviceKind, DeviceRef, MatrixSize, ZoneDescriptor, ZoneKind,
};
pub use error::BackendError;
pub use identity::{
    AmbiguousGroup, DeviceFingerprint, IdentityMatch, KnownDevice, Reconciliation, ZoneShape,
    reconcile,
};
pub use inventory::{DeviceIdentity, DeviceIssue, Inventory, InventoryDevice, IssueReason};
pub use ownership::OutputOwner;
pub use prismforge_effects::Rgb;
pub use scene::{ApplyMode, Assignment, OutputTarget, Scene};
