use std::future::Future;

use prismforge_effects::Rgb;
use serde::{Deserialize, Serialize};

use crate::capabilities::DeviceCapabilities;
use crate::connection::{ConnectionState, LossCause};
use crate::device::{BackendOrigin, DeviceRef};
use crate::error::BackendError;
use crate::inventory::Inventory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEvent {
    InventoryChanged,
    ConnectionLost { cause: LossCause, detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct BackendInfo {
    pub origin: BackendOrigin,
    pub address: Option<String>,
    pub protocol_version: Option<u32>,
    pub server_name: Option<String>,
}

/// Futures other than `next_event` are not cancel-safe: dropping one mid-call closes the connection.
pub trait RgbBackend {
    fn connection_state(&self) -> &ConnectionState;

    fn info(&self) -> BackendInfo;

    fn connect(&mut self) -> impl Future<Output = Result<(), BackendError>> + Send;

    fn disconnect(&mut self);

    /// Resolves when the server reports a change; `None` once no connection is open. Cancel-safe.
    fn next_event(&mut self) -> impl Future<Output = Option<BackendEvent>> + Send;

    fn refresh_inventory(
        &mut self,
    ) -> impl Future<Output = Result<&Inventory, BackendError>> + Send;

    fn inventory(&self) -> Option<&Inventory>;

    fn capabilities(&self, device: &DeviceRef) -> Result<&DeviceCapabilities, BackendError>;

    fn enable_direct_mode(
        &mut self,
        device: &DeviceRef,
    ) -> impl Future<Output = Result<(), BackendError>> + Send;

    /// Re-applies an existing mode with the settings last reported for it; never saves to device memory.
    fn reapply_mode(
        &mut self,
        device: &DeviceRef,
        mode: usize,
    ) -> impl Future<Output = Result<(), BackendError>> + Send;

    fn write_device_frame(
        &mut self,
        device: &DeviceRef,
        colors: &[Rgb],
    ) -> impl Future<Output = Result<(), BackendError>> + Send;

    fn write_zone_frame(
        &mut self,
        device: &DeviceRef,
        zone: usize,
        colors: &[Rgb],
    ) -> impl Future<Output = Result<(), BackendError>> + Send;
}
