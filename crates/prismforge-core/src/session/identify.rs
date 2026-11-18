use serde::{Deserialize, Serialize};

use crate::backend::RgbBackend;
use crate::color::Rgb;
use crate::connection::StateReason;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::inventory::Inventory;

pub const IDENTIFY_COLOR: Rgb = Rgb::new(255, 255, 255);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct IdentifyOutcome {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    /// Set when lighting the target failed; restore then ran at once.
    pub light_failure: Option<BackendError>,
    pub restore: RestoreOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RestoreOutcome {
    /// Lighting was refused before the device changed, so there was nothing to restore.
    Unchanged,
    Restored,
    PartlyRestored {
        failure: RestoreFailure,
    },
    NotRestored {
        failure: RestoreFailure,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RestoreFailure {
    /// Nothing is retried after a reconnect, because the device behind the reference may differ.
    ConnectionLost {
        reason: StateReason,
    },
    Refused {
        error: BackendError,
    },
    /// The remembered state may be an earlier identify's own output, or no active mode was reported.
    PreviousStateUnknown,
    /// After a rescan the device no longer has the remembered modes or LED count; nothing was written.
    TargetChanged,
}

pub(crate) struct IdentifyPlan {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    pub previous_state_known: bool,
    lit_leds: usize,
    steps: Vec<RestoreStep>,
    previous_colors: Vec<Rgb>,
}

enum RestoreStep {
    Mode { index: usize, name: String },
    Colors,
}

pub(crate) struct LightError {
    pub device_changed: bool,
    pub error: BackendError,
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
        state_suspect: bool,
    ) -> Result<Self, BackendError> {
        let descriptor = &inventory.output_target(device, zone)?.descriptor;
        let capabilities = &descriptor.capabilities;
        let restore_mode = capabilities
            .active_mode
            .filter(|_| !capabilities.in_direct_mode())
            .and_then(|index| Some((index, capabilities.modes.get(index)?)));
        let mut steps = Vec::new();
        if let Some((index, mode)) = restore_mode {
            steps.push(RestoreStep::Mode {
                index,
                name: mode.name.clone(),
            });
        }
        if restore_mode.is_none_or(|(_, mode)| mode.per_led_color) {
            steps.push(RestoreStep::Colors);
        }
        Ok(Self {
            device: device.clone(),
            zone,
            previous_state_known: capabilities.active_mode.is_some() && !state_suspect,
            lit_leds: zone.map_or(descriptor.led_count, |zone| {
                descriptor.zones[zone].led_count
            }),
            steps,
            previous_colors: descriptor.colors.clone(),
        })
    }

    pub async fn light<B: RgbBackend>(&self, backend: &mut B) -> Result<(), LightError> {
        backend
            .enable_direct_mode(&self.device)
            .await
            .map_err(|error| LightError {
                device_changed: false,
                error,
            })?;
        let colors = vec![IDENTIFY_COLOR; self.lit_leds];
        let written = match self.zone {
            Some(zone) => backend.write_zone_frame(&self.device, zone, &colors).await,
            None => backend.write_device_frame(&self.device, &colors).await,
        };
        written.map_err(|error| LightError {
            device_changed: true,
            error,
        })
    }

    /// Runs the restore steps after the first `skip`; the count in an error includes the skipped ones.
    pub async fn restore<B: RgbBackend>(
        &self,
        backend: &mut B,
        skip: usize,
    ) -> Result<(), RestoreError> {
        for (position, step) in self.steps.iter().enumerate().skip(skip) {
            let result = match step {
                RestoreStep::Mode { index, .. } => backend.reapply_mode(&self.device, *index).await,
                RestoreStep::Colors => {
                    backend
                        .write_device_frame(&self.device, &self.previous_colors)
                        .await
                }
            };
            result.map_err(|error| RestoreError {
                steps_done: position,
                error,
            })?;
        }
        Ok(())
    }

    /// Checks that a refreshed inventory still resolves this reference to a device with the remembered shape.
    pub fn still_matches(&self, inventory: &Inventory) -> Result<(), RestoreFailure> {
        let index = inventory
            .locate(&self.device)
            .map_err(|error| RestoreFailure::Refused { error })?;
        let descriptor = &inventory.devices[index].descriptor;
        let modes_match = self.steps.iter().all(|step| match step {
            RestoreStep::Mode { index, name } => descriptor
                .capabilities
                .modes
                .get(*index)
                .is_some_and(|mode| &mode.name == name),
            RestoreStep::Colors => descriptor.led_count == self.previous_colors.len(),
        });
        if modes_match {
            Ok(())
        } else {
            Err(RestoreFailure::TargetChanged)
        }
    }
}
