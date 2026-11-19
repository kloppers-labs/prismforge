use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::identity::{DeviceFingerprint, KnownDevice, reconcile};
use crate::profile::model::{Profile, ProfileDevice};
use crate::session::SessionDevice;

/// How a stored device relates to the devices the session knows now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum DeviceStatus {
    /// Exactly one available device matches; `device` is its reference in the session.
    Resolved { device: DeviceRef },
    /// The matching device is known to the session but not available now.
    Unavailable { device: DeviceRef },
    /// No device matches.
    Missing,
    /// More than one device matches, so none is used.
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceResolution {
    pub reference: DeviceRef,
    pub label: String,
    pub status: DeviceStatus,
}

impl DeviceStatus {
    pub fn device(&self) -> Option<&DeviceRef> {
        match self {
            Self::Resolved { device } | Self::Unavailable { device } => Some(device),
            Self::Missing | Self::Ambiguous => None,
        }
    }
}

/// Matches each stored device on its identifying fields with the registry rules, never by runtime ID or position.
pub fn resolve(profile: &Profile, devices: &[SessionDevice]) -> Vec<DeviceResolution> {
    let (portable, local): (Vec<&ProfileDevice>, Vec<&ProfileDevice>) = profile
        .devices
        .iter()
        .partition(|device| device.identity.is_portable());
    let current: Vec<DeviceFingerprint> = devices
        .iter()
        .map(|device| device.descriptor.fingerprint())
        .collect();
    let stripped: Vec<DeviceFingerprint> = current
        .iter()
        .map(|fingerprint| DeviceFingerprint {
            serial: None,
            location: None,
            ..fingerprint.clone()
        })
        .collect();
    let mut found: Vec<(DeviceRef, Found)> = Vec::new();
    found.extend(matches(&local, &current));
    found.extend(matches(&portable, &stripped));
    let statuses: Vec<(DeviceRef, DeviceStatus)> = found
        .iter()
        .map(|(reference, outcome)| {
            let status = match outcome {
                Found::Current(index) if claimed_once(&found, *index) => {
                    status_of(&devices[*index])
                }
                Found::Current(_) | Found::Ambiguous => DeviceStatus::Ambiguous,
                Found::Missing => DeviceStatus::Missing,
            };
            (reference.clone(), status)
        })
        .collect();
    profile
        .devices
        .iter()
        .map(|device| DeviceResolution {
            reference: device.reference.clone(),
            label: device.label.clone(),
            status: statuses
                .iter()
                .find(|(reference, _)| reference == &device.reference)
                .map_or(DeviceStatus::Missing, |(_, status)| status.clone()),
        })
        .collect()
}

enum Found {
    Current(usize),
    Ambiguous,
    Missing,
}

fn matches(stored: &[&ProfileDevice], current: &[DeviceFingerprint]) -> Vec<(DeviceRef, Found)> {
    let known: Vec<KnownDevice> = stored
        .iter()
        .map(|device| KnownDevice {
            reference: device.reference.clone(),
            fingerprint: device.identity.fingerprint(),
        })
        .collect();
    let reconciliation = reconcile(&known, current);
    let matched = reconciliation
        .matched
        .into_iter()
        .map(|matched| (matched.reference, Found::Current(matched.current)));
    let ambiguous = reconciliation
        .ambiguous
        .into_iter()
        .flat_map(|group| group.candidates)
        .map(|reference| (reference, Found::Ambiguous));
    let missing = reconciliation
        .missing
        .into_iter()
        .map(|reference| (reference, Found::Missing));
    matched.chain(ambiguous).chain(missing).collect()
}

fn claimed_once(found: &[(DeviceRef, Found)], index: usize) -> bool {
    found
        .iter()
        .filter(|(_, outcome)| matches!(outcome, Found::Current(other) if *other == index))
        .count()
        == 1
}

fn status_of(device: &SessionDevice) -> DeviceStatus {
    match (device.identity.reference(), device.available) {
        (Some(reference), true) => DeviceStatus::Resolved {
            device: reference.clone(),
        },
        (Some(reference), false) => DeviceStatus::Unavailable {
            device: reference.clone(),
        },
        (None, _) => DeviceStatus::Ambiguous,
    }
}
