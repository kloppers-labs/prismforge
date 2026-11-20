use serde::{Deserialize, Serialize};

use crate::capabilities::ModePersistence;
use crate::device::DeviceRef;
use crate::session::{OutputError, SessionDevice};

const OFF_MODE: &str = "off";

/// What switching the lights off will write, worked out before the event so little is left to do then.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LightsOffPlan {
    pub steps: Vec<LightsOffStep>,
    pub skipped: Vec<DeviceLightsOff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightsOffStep {
    pub device: DeviceRef,
    pub name: String,
    pub write: LightsOffWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum LightsOffWrite {
    /// Black on every LED through Direct Mode, which never saves to device memory.
    Black { leds: usize },
    /// The device's own mode named "Off"; chosen only when that mode does not save to device memory.
    OffMode { mode: usize, name: String },
}

/// What switching the lights off did, one entry per device.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct LightsOffReport {
    pub devices: Vec<DeviceLightsOff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceLightsOff {
    /// `None` for a device PrismForge cannot tell apart from an identical one.
    pub device: Option<DeviceRef>,
    pub name: String,
    pub outcome: LightsOffOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum LightsOffOutcome {
    Black,
    OffMode { mode: usize, name: String },
    Skipped { reason: LightsOffSkip },
    Failed { error: OutputError },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum LightsOffSkip {
    NotResolved,
    NotAvailable,
    /// No Direct Mode and no mode named "Off".
    NoDirectOrOffMode,
    /// The "Off" mode saves to device memory, and frequent writes there wear it.
    OffModeSavesToDevice,
    DeclaredExternal {
        label: String,
    },
    Identifying,
    /// The event's time had passed when the session came to it, so nothing was written.
    TooLate,
}

pub fn plan_lights_off(devices: &[SessionDevice]) -> LightsOffPlan {
    let mut plan = LightsOffPlan::default();
    for device in devices {
        let name = device.descriptor.name.clone();
        let Some(reference) = device.identity.reference() else {
            plan.skipped
                .push(skip(None, name, LightsOffSkip::NotResolved));
            continue;
        };
        let reference = reference.clone();
        let written = if device.available {
            write_for(device)
        } else {
            Err(LightsOffSkip::NotAvailable)
        };
        match written {
            Ok(write) => plan.steps.push(LightsOffStep {
                device: reference,
                name,
                write,
            }),
            Err(reason) => plan.skipped.push(skip(Some(reference), name, reason)),
        }
    }
    plan
}

fn write_for(device: &SessionDevice) -> Result<LightsOffWrite, LightsOffSkip> {
    let capabilities = &device.descriptor.capabilities;
    if capabilities.has_direct_mode() {
        return Ok(LightsOffWrite::Black {
            leds: device.descriptor.led_count,
        });
    }
    let (mode, off) = capabilities
        .modes
        .iter()
        .enumerate()
        .find(|(_, mode)| mode.name.trim().eq_ignore_ascii_case(OFF_MODE))
        .ok_or(LightsOffSkip::NoDirectOrOffMode)?;
    if off.persistence == ModePersistence::AutomaticSave {
        return Err(LightsOffSkip::OffModeSavesToDevice);
    }
    Ok(LightsOffWrite::OffMode {
        mode,
        name: off.name.clone(),
    })
}

fn skip(device: Option<DeviceRef>, name: String, reason: LightsOffSkip) -> DeviceLightsOff {
    DeviceLightsOff {
        device,
        name,
        outcome: LightsOffOutcome::Skipped { reason },
    }
}
