use serde::{Deserialize, Serialize};

use crate::backend::RgbBackend;
use crate::color::Rgb;
use crate::connection::StateReason;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::inventory::Inventory;

pub const IDENTIFY_COLOR: Rgb = Rgb::new(255, 255, 255);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifyOutcome {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    /// Set when lighting the target failed; restore then ran at once.
    pub light_failure: Option<BackendError>,
    pub restore: RestoreOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreOutcome {
    Restored,
    PartlyRestored { failure: RestoreFailure },
    NotRestored { failure: RestoreFailure },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreFailure {
    /// Nothing is retried after a reconnect, because the device behind the reference may differ.
    ConnectionLost {
        reason: StateReason,
    },
    Refused {
        error: BackendError,
    },
}

pub(crate) struct IdentifyPlan {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    lit_leds: usize,
    restore_mode: Option<usize>,
    restore_colors: bool,
    previous_colors: Vec<Rgb>,
}

pub(crate) struct RestoreError {
    pub steps_done: usize,
    pub error: BackendError,
}

impl IdentifyPlan {
    /// Validates the target and remembers the state the inventory reports, before anything is sent.
    pub fn new(
        inventory: &Inventory,
        device: &DeviceRef,
        zone: Option<usize>,
    ) -> Result<Self, BackendError> {
        let descriptor = &inventory.output_target(device, zone)?.descriptor;
        let capabilities = &descriptor.capabilities;
        let restore_mode = capabilities
            .active_mode
            .filter(|_| !capabilities.in_direct_mode());
        let restore_colors = restore_mode.is_none_or(|mode| {
            capabilities
                .modes
                .get(mode)
                .is_some_and(|mode| mode.per_led_color)
        });
        Ok(Self {
            device: device.clone(),
            zone,
            lit_leds: zone.map_or(descriptor.led_count, |zone| {
                descriptor.zones[zone].led_count
            }),
            restore_mode,
            restore_colors,
            previous_colors: descriptor.colors.clone(),
        })
    }

    pub async fn light<B: RgbBackend>(&self, backend: &mut B) -> Result<(), BackendError> {
        backend.enable_direct_mode(&self.device).await?;
        let colors = vec![IDENTIFY_COLOR; self.lit_leds];
        match self.zone {
            Some(zone) => backend.write_zone_frame(&self.device, zone, &colors).await,
            None => backend.write_device_frame(&self.device, &colors).await,
        }
    }

    pub async fn restore<B: RgbBackend>(&self, backend: &mut B) -> Result<(), RestoreError> {
        let mut steps_done = 0;
        let fail = |steps_done, error| RestoreError { steps_done, error };
        if let Some(mode) = self.restore_mode {
            backend
                .reapply_mode(&self.device, mode)
                .await
                .map_err(|error| fail(steps_done, error))?;
            steps_done += 1;
        }
        if self.restore_colors {
            backend
                .write_device_frame(&self.device, &self.previous_colors)
                .await
                .map_err(|error| fail(steps_done, error))?;
        }
        Ok(())
    }
}
