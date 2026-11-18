mod inventory;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use prismforge_core::{
    BackendError, BackendEvent, BackendInfo, BackendOrigin, ConnectionState, DeviceCapabilities,
    DeviceIdentity, DeviceRef, Inventory, InventoryDevice, LossCause, Rgb, RgbBackend, StateReason,
};
use tokio::sync::Notify;

use inventory::{DemoDevice, demo_devices};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoOutput {
    DirectMode {
        device: DeviceRef,
    },
    ModeReapplied {
        device: DeviceRef,
        mode: usize,
    },
    DeviceFrame {
        device: DeviceRef,
        colors: Vec<Rgb>,
    },
    ZoneFrame {
        device: DeviceRef,
        zone: usize,
        colors: Vec<Rgb>,
    },
}

#[derive(Default)]
struct Shared {
    outputs: Vec<DemoOutput>,
    fail_connect: bool,
    call_delay: Duration,
    pending_loss: Option<LossCause>,
    inventory_changed: bool,
}

/// Test and preview handle for a [`DemoBackend`]; clones share the same simulation.
#[derive(Clone, Default)]
pub struct DemoControls {
    shared: Arc<Mutex<Shared>>,
    wake: Arc<Notify>,
}

impl DemoControls {
    pub fn fail_connect(&self, fail: bool) {
        self.lock().fail_connect = fail;
    }

    pub fn lose_connection(&self, cause: LossCause) {
        self.lock().pending_loss = Some(cause);
        self.wake.notify_one();
    }

    pub fn change_inventory(&self) {
        self.lock().inventory_changed = true;
        self.wake.notify_one();
    }

    /// Every later backend call waits this long before it does anything.
    pub fn set_call_delay(&self, delay: Duration) {
        self.lock().call_delay = delay;
    }

    pub fn outputs(&self) -> Vec<DemoOutput> {
        self.lock().outputs.clone()
    }

    fn record(&self, output: DemoOutput) {
        self.lock().outputs.push(output);
    }

    fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().expect("demo state poisoned")
    }
}

/// Simulated backend with a fixed, made-up PC inventory; every device reports origin `Simulated`.
pub struct DemoBackend {
    controls: DemoControls,
    devices: Vec<DemoDevice>,
    state: ConnectionState,
    connected: bool,
    stale: bool,
    inventory: Option<Inventory>,
}

impl Default for DemoBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoBackend {
    pub fn new() -> Self {
        Self {
            controls: DemoControls::default(),
            devices: demo_devices(),
            state: ConnectionState::Disconnected { reason: None },
            connected: false,
            stale: true,
            inventory: None,
        }
    }

    pub fn controls(&self) -> DemoControls {
        self.controls.clone()
    }

    fn absorb(&mut self) -> Option<BackendEvent> {
        if !self.connected {
            return None;
        }
        let mut shared = self.controls.lock();
        if let Some(cause) = shared.pending_loss.take() {
            drop(shared);
            self.connected = false;
            self.stale = true;
            self.state = ConnectionState::Disconnected {
                reason: Some(StateReason::ConnectionLost { cause }),
            };
            return Some(BackendEvent::ConnectionLost {
                cause,
                detail: "simulated connection loss".into(),
            });
        }
        if std::mem::take(&mut shared.inventory_changed) {
            drop(shared);
            self.stale = true;
            if self.inventory.is_some() {
                self.state = ConnectionState::Degraded {
                    reasons: vec![StateReason::InventoryStale],
                };
            }
            return Some(BackendEvent::InventoryChanged);
        }
        None
    }

    async fn begin(&mut self) -> Result<(), BackendError> {
        let delay = self.controls.lock().call_delay;
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        self.absorb();
        if self.connected {
            Ok(())
        } else {
            Err(BackendError::NotConnected)
        }
    }

    async fn begin_output(&mut self, device: &DeviceRef) -> Result<usize, BackendError> {
        self.begin().await?;
        if self.stale {
            return Err(BackendError::StaleInventory);
        }
        self.current().locate(device)
    }

    fn check_frame(
        &self,
        device: &DeviceRef,
        zone: Option<usize>,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        let descriptor = &self.current().output_target(device, zone)?.descriptor;
        let expected = zone.map_or(descriptor.led_count, |zone| {
            descriptor.zones[zone].led_count
        });
        if colors.len() != expected {
            return Err(BackendError::ColorCountMismatch {
                device: device.clone(),
                expected,
                actual: colors.len(),
            });
        }
        Ok(())
    }

    fn current(&self) -> &Inventory {
        self.inventory
            .as_ref()
            .expect("a fresh inventory exists while not stale")
    }
}

impl RgbBackend for DemoBackend {
    fn connection_state(&self) -> &ConnectionState {
        &self.state
    }

    fn info(&self) -> BackendInfo {
        BackendInfo {
            origin: BackendOrigin::Simulated,
            address: None,
            protocol_version: None,
            server_name: None,
        }
    }

    async fn connect(&mut self) -> Result<(), BackendError> {
        self.disconnect();
        self.state = ConnectionState::Connecting;
        let delay = self.controls.lock().call_delay;
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        let mut shared = self.controls.lock();
        shared.pending_loss = None;
        shared.inventory_changed = false;
        if shared.fail_connect {
            drop(shared);
            self.state = ConnectionState::Disconnected {
                reason: Some(StateReason::ServerUnreachable),
            };
            return Err(BackendError::Unreachable {
                detail: "simulated connection failure".into(),
            });
        }
        drop(shared);
        self.connected = true;
        self.state = ConnectionState::Connected;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
        self.stale = true;
        self.state = ConnectionState::Disconnected { reason: None };
    }

    async fn next_event(&mut self) -> Option<BackendEvent> {
        loop {
            if !self.connected {
                return None;
            }
            if let Some(event) = self.absorb() {
                return Some(event);
            }
            self.controls.wake.clone().notified().await;
        }
    }

    async fn refresh_inventory(&mut self) -> Result<&Inventory, BackendError> {
        self.begin().await?;
        let generation = self
            .inventory
            .as_ref()
            .map_or(1, |previous| previous.generation + 1);
        let devices = self
            .devices
            .iter()
            .map(|device| InventoryDevice {
                identity: if generation == 1 {
                    DeviceIdentity::New(device.reference.clone())
                } else {
                    DeviceIdentity::Matched(device.reference.clone())
                },
                descriptor: device.descriptor.clone(),
            })
            .collect();
        self.stale = false;
        self.state = ConnectionState::Connected;
        Ok(self.inventory.insert(Inventory {
            generation,
            devices,
            missing: Vec::new(),
            issues: Vec::new(),
        }))
    }

    fn inventory(&self) -> Option<&Inventory> {
        self.inventory.as_ref()
    }

    fn capabilities(&self, device: &DeviceRef) -> Result<&DeviceCapabilities, BackendError> {
        let inventory = self
            .inventory
            .as_ref()
            .ok_or(BackendError::StaleInventory)?;
        Ok(&inventory.devices[inventory.locate(device)?]
            .descriptor
            .capabilities)
    }

    async fn enable_direct_mode(&mut self, device: &DeviceRef) -> Result<(), BackendError> {
        let index = self.begin_output(device).await?;
        self.current().output_target(device, None)?;
        let capabilities = &mut self.devices[index].descriptor.capabilities;
        capabilities.active_mode = capabilities.direct_mode;
        self.controls.record(DemoOutput::DirectMode {
            device: device.clone(),
        });
        Ok(())
    }

    async fn reapply_mode(&mut self, device: &DeviceRef, mode: usize) -> Result<(), BackendError> {
        let index = self.begin_output(device).await?;
        let capabilities = &mut self.devices[index].descriptor.capabilities;
        if mode >= capabilities.modes.len() {
            return Err(BackendError::ModeOutOfRange {
                device: device.clone(),
                mode,
                mode_count: capabilities.modes.len(),
            });
        }
        capabilities.active_mode = Some(mode);
        self.controls.record(DemoOutput::ModeReapplied {
            device: device.clone(),
            mode,
        });
        Ok(())
    }

    async fn write_device_frame(
        &mut self,
        device: &DeviceRef,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        let index = self.begin_output(device).await?;
        self.check_frame(device, None, colors)?;
        self.devices[index].descriptor.colors = colors.to_vec();
        self.controls.record(DemoOutput::DeviceFrame {
            device: device.clone(),
            colors: colors.to_vec(),
        });
        Ok(())
    }

    async fn write_zone_frame(
        &mut self,
        device: &DeviceRef,
        zone: usize,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        let index = self.begin_output(device).await?;
        self.check_frame(device, Some(zone), colors)?;
        let descriptor = &mut self.devices[index].descriptor;
        let start: usize = descriptor.zones[..zone]
            .iter()
            .map(|zone| zone.led_count)
            .sum();
        descriptor.colors[start..start + colors.len()].copy_from_slice(colors);
        self.controls.record(DemoOutput::ZoneFrame {
            device: device.clone(),
            zone,
            colors: colors.to_vec(),
        });
        Ok(())
    }
}
