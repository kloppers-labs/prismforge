use std::future::Future;
use std::sync::{Arc, Mutex};

use prismforge_core::{
    BackendError, BackendEvent, BackendInfo, ConnectionState, DeviceCapabilities, DeviceRef,
    Inventory, KnownDevice, Rgb, RgbBackend,
};

#[derive(Clone, Default)]
pub struct Log(Arc<Mutex<Vec<String>>>);

impl Log {
    pub fn entries(&self) -> Vec<String> {
        self.0.lock().expect("log").clone()
    }

    fn push(&self, entry: String) {
        self.0.lock().expect("log").push(entry);
    }
}

pub struct LoggedBackend<B> {
    pub name: &'static str,
    pub inner: B,
    pub log: Log,
}

impl<B: RgbBackend + Send> RgbBackend for LoggedBackend<B> {
    fn connection_state(&self) -> &ConnectionState {
        self.inner.connection_state()
    }

    fn info(&self) -> BackendInfo {
        self.inner.info()
    }

    async fn connect(&mut self) -> Result<(), BackendError> {
        self.log.push(format!("{} connect", self.name));
        self.inner.connect().await
    }

    fn disconnect(&mut self) {
        self.log.push(format!("{} disconnect", self.name));
        self.inner.disconnect();
    }

    fn next_event(&mut self) -> impl Future<Output = Option<BackendEvent>> + Send {
        self.inner.next_event()
    }

    async fn refresh_inventory(&mut self) -> Result<&Inventory, BackendError> {
        self.inner.refresh_inventory().await
    }

    fn inventory(&self) -> Option<&Inventory> {
        self.inner.inventory()
    }

    fn known_devices(&self) -> &[KnownDevice] {
        self.inner.known_devices()
    }

    fn seed_known_devices(&mut self, known: Vec<KnownDevice>) {
        self.inner.seed_known_devices(known);
    }

    fn capabilities(&self, device: &DeviceRef) -> Result<&DeviceCapabilities, BackendError> {
        self.inner.capabilities(device)
    }

    async fn enable_direct_mode(&mut self, device: &DeviceRef) -> Result<(), BackendError> {
        self.inner.enable_direct_mode(device).await
    }

    async fn reapply_mode(&mut self, device: &DeviceRef, mode: usize) -> Result<(), BackendError> {
        self.inner.reapply_mode(device, mode).await
    }

    async fn write_device_frame(
        &mut self,
        device: &DeviceRef,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        self.inner.write_device_frame(device, colors).await
    }

    async fn write_zone_frame(
        &mut self,
        device: &DeviceRef,
        zone: usize,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        self.inner.write_zone_frame(device, zone, colors).await
    }
}
