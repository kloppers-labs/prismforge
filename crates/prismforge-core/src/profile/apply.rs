use serde::{Deserialize, Serialize};

use crate::connection::ConnectionState;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::ownership::OutputOwner;
use crate::profile::effects::EffectProblem;
use crate::profile::model::{Profile, ProfileDevice};
use crate::profile::resolve::{DeviceResolution, DeviceStatus, resolve};
use crate::scene::OutputTarget;
use crate::session::{Origin, OutputError, SessionDevice, SessionError, SessionHandle};

/// What happened to each stored setting; partial application is normal and nothing is rolled back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ApplyReport {
    pub frame_rate: FrameRateOutcome,
    pub items: Vec<ItemReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum FrameRateOutcome {
    Applied { in_use: u32 },
    Refused { error: OutputError },
}

/// `device` is the key in the profile; `target` is the session device it resolved to, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ItemReport {
    pub device: DeviceRef,
    pub label: String,
    pub target: Option<DeviceRef>,
    pub item: ProfileItem,
    pub outcome: ItemOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ProfileItem {
    Effect { zone: Option<usize>, effect: String },
    HardwareMode { index: usize, name: String },
    External { label: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ItemOutcome {
    /// The session accepted the setting.
    Applied,
    Skipped {
        reason: SkipReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SkipReason {
    Missing,
    Ambiguous,
    Unavailable,
    /// The user said another program controls the device; PrismForge sent it nothing.
    DeclaredExternal {
        label: String,
    },
    NoDirectMode,
    /// The stored mode name is not at the stored index any more; `found` is the name there now.
    ModeListChanged {
        found: Option<String>,
    },
    NotAvailable {
        problem: EffectProblem,
    },
    Refused {
        error: OutputError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ApplyError {
    #[error("PrismForge is not connected, so the profile was not applied")]
    NotConnected,
    #[error(transparent)]
    Store(#[from] crate::store::StoreError),
}

/// Applies `profile` through the session commands as a profile, so it creates no user intent records.
pub async fn apply(handle: &SessionHandle, profile: &Profile) -> Result<ApplyReport, ApplyError> {
    let snapshot = handle.snapshot();
    if !is_live(&snapshot.connection) {
        return Err(ApplyError::NotConnected);
    }
    let resolutions = resolve(profile, &snapshot.devices);
    let frame_rate = match handle
        .set_frame_rate_as(Origin::Profile, profile.frame_rate)
        .await
    {
        Ok(in_use) => FrameRateOutcome::Applied { in_use },
        Err(error) => FrameRateOutcome::Refused { error },
    };
    let mut items = Vec::new();
    for external in &profile.external {
        let item = ProfileItem::External {
            label: external.label.clone(),
        };
        let outcome = match target_of(&resolutions, &external.device) {
            Ok(device) => done(
                handle
                    .declare_external_as(Origin::Profile, device, external.label.clone())
                    .await,
            ),
            Err(reason) => skip(reason),
        };
        items.push(report(
            profile,
            &resolutions,
            &external.device,
            item,
            outcome,
        ));
    }
    for mode in &profile.modes {
        let item = ProfileItem::HardwareMode {
            index: mode.index,
            name: mode.name.clone(),
        };
        let outcome = match target_of(&resolutions, &mode.device) {
            Ok(device) => match mode_now(&snapshot.devices, &device, mode.index) {
                Some(found) if found == mode.name => done(
                    handle
                        .select_mode_as(Origin::Profile, device, mode.index)
                        .await,
                ),
                found => skip(SkipReason::ModeListChanged { found }),
            },
            Err(reason) => skip(reason),
        };
        items.push(report(profile, &resolutions, &mode.device, item, outcome));
    }
    for assignment in &profile.assignments {
        let item = ProfileItem::Effect {
            zone: assignment.zone,
            effect: assignment.effect.effect.clone(),
        };
        let outcome = match (
            target_of(&resolutions, &assignment.device),
            assignment.effect.instance(),
        ) {
            (Err(reason), _) => skip(reason),
            (Ok(_), Err(problem)) => skip(SkipReason::NotAvailable { problem }),
            (Ok(device), Ok(instance)) => {
                let target = OutputTarget {
                    device,
                    zone: assignment.zone,
                };
                done(
                    handle
                        .apply_effect_as(Origin::Profile, target, instance)
                        .await,
                )
            }
        };
        items.push(report(
            profile,
            &resolutions,
            &assignment.device,
            item,
            outcome,
        ));
    }
    Ok(ApplyReport { frame_rate, items })
}

pub(crate) fn is_live(connection: &ConnectionState) -> bool {
    matches!(
        connection,
        ConnectionState::Connected | ConnectionState::Degraded { .. }
    )
}

pub(crate) fn target_of(
    resolutions: &[DeviceResolution],
    device: &DeviceRef,
) -> Result<DeviceRef, SkipReason> {
    let status = resolutions
        .iter()
        .find(|resolution| &resolution.reference == device)
        .map(|resolution| &resolution.status);
    match status {
        Some(DeviceStatus::Resolved { device }) => Ok(device.clone()),
        Some(DeviceStatus::Unavailable { .. }) => Err(SkipReason::Unavailable),
        Some(DeviceStatus::Ambiguous) => Err(SkipReason::Ambiguous),
        Some(DeviceStatus::Missing) | None => Err(SkipReason::Missing),
    }
}

pub(crate) fn mode_now(
    devices: &[SessionDevice],
    device: &DeviceRef,
    index: usize,
) -> Option<String> {
    devices
        .iter()
        .find(|candidate| candidate.identity.reference() == Some(device))?
        .descriptor
        .capabilities
        .modes
        .get(index)
        .map(|mode| mode.name.clone())
}

fn done(result: Result<(), OutputError>) -> ItemOutcome {
    match result {
        Ok(()) => ItemOutcome::Applied,
        Err(error) => skip(reason_of(error)),
    }
}

fn skip(reason: SkipReason) -> ItemOutcome {
    ItemOutcome::Skipped { reason }
}

fn reason_of(error: OutputError) -> SkipReason {
    match error {
        OutputError::Session(SessionError::OutputConflict {
            owner: OutputOwner::External { label },
            ..
        }) => SkipReason::DeclaredExternal { label },
        OutputError::Session(SessionError::Backend(backend)) => match backend {
            BackendError::DirectModeUnavailable { .. } => SkipReason::NoDirectMode,
            BackendError::AmbiguousDevice { .. } => SkipReason::Ambiguous,
            BackendError::DeviceMissing { .. } | BackendError::UnknownDevice { .. } => {
                SkipReason::Unavailable
            }
            other => SkipReason::Refused {
                error: OutputError::Session(SessionError::Backend(other)),
            },
        },
        error => SkipReason::Refused { error },
    }
}

fn report(
    profile: &Profile,
    resolutions: &[DeviceResolution],
    device: &DeviceRef,
    item: ProfileItem,
    outcome: ItemOutcome,
) -> ItemReport {
    ItemReport {
        device: device.clone(),
        label: profile
            .device(device)
            .map(|stored: &ProfileDevice| stored.label.clone())
            .unwrap_or_default(),
        target: resolutions
            .iter()
            .find(|resolution| &resolution.reference == device)
            .and_then(|resolution| resolution.status.device().cloned()),
        item,
        outcome,
    }
}
