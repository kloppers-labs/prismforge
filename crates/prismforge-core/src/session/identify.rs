use serde::{Deserialize, Serialize};

use crate::backend::RgbBackend;
use crate::color::Rgb;
use crate::connection::StateReason;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::inventory::Inventory;
use crate::session::previous::PreviousState;

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
    pub previous: PreviousState,
    lit_leds: usize,
}

pub(crate) struct LightError {
    pub device_changed: bool,
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
        Ok(Self {
            device: device.clone(),
            zone,
            previous: PreviousState::capture(device, descriptor, state_suspect),
            lit_leds: zone.map_or(descriptor.led_count, |zone| {
                descriptor.zones[zone].led_count
            }),
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
}
