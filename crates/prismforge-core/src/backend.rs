use std::future::Future;

use crate::capabilities::DeviceCapabilities;
use crate::color::Rgb;
use crate::connection::ConnectionState;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::inventory::Inventory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEvent {
    InventoryChanged,
    ConnectionLost { detail: String },
}

pub trait RgbBackend {
    fn connection_state(&self) -> &ConnectionState;

    fn connect(&mut self) -> impl Future<Output = Result<(), BackendError>> + Send;

    fn disconnect(&mut self);

    /// Resolves when the server reports a change; `None` once no connection is open.
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
