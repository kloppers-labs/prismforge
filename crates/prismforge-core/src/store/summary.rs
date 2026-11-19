use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::profile::{
    DeviceResolution, DeviceStatus, EffectProblem, Profile, ProfileId, ProfileItem, SkipReason,
    preflight, resolve,
};
use crate::session::{SessionDevice, SessionSnapshot};
use crate::store::StoreError;
use crate::store::envelope::FileProblem;
use crate::store::notice::StoreNotice;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct StoreSummary {
    pub profiles: Vec<ProfileSummary>,
    /// Profiles that need a newer PrismForge; their files are left as they are.
    pub unreadable: Vec<UnreadableProfile>,
    pub apply_on_connect: Option<ProfileId>,
    /// The profile applied or saved in the session the summary was asked for; updates need it.
    pub active: Option<ProfileId>,
    /// Settings an update of the active profile would change now; `None` without an active profile.
    pub unsaved_changes: Option<usize>,
    /// What the store did to damaged or missing files since it opened, and failed background writes.
    pub notices: Vec<StoreNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ProfileSummary {
    pub id: ProfileId,
    pub name: String,
    pub devices: usize,
    pub effects: usize,
    pub modes: usize,
    pub external: usize,
    /// `None` when there are no current devices to match against.
    pub resolution: Option<ResolutionCounts>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ResolutionCounts {
    pub resolved: usize,
    pub unavailable: usize,
    pub missing: usize,
    pub ambiguous: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct UnreadableProfile {
    pub id: ProfileId,
    pub problem: FileProblem,
}

/// One profile with every stored device matched against the current devices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ProfileDetails {
    pub id: ProfileId,
    pub name: String,
    pub frame_rate: u32,
    pub apply_on_connect: bool,
    /// False when there was no session to match against; then every device reads as missing.
    pub checked: bool,
    pub devices: Vec<DeviceResolution>,
    /// Every stored setting in apply order, with what apply would do with it today.
    pub settings: Vec<StoredSetting>,
    /// Stored effects this version cannot run; they stay in the profile.
    pub unsupported: Vec<UnsupportedEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct StoredSetting {
    pub device: DeviceRef,
    pub label: String,
    /// The stored name of the zone an effect targets.
    pub zone_name: Option<String>,
    pub item: ProfileItem,
    /// The effect's name in this version; `None` for an effect this version does not have.
    pub effect_label: Option<String>,
    pub today: Readiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Readiness {
    Ready,
    Skipped { reason: SkipReason },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct UnsupportedEffect {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    pub problem: EffectProblem,
}

/// An export ready to be written; `omitted_fields` counts unknown fields left out of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedProfile {
    pub bytes: Vec<u8>,
    pub omitted_fields: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ExportReport {
    /// Unknown fields from a newer version, left out because they could hold private data.
    pub omitted_fields: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ImportReport {
    /// The name in the file, when another profile already had it.
    pub renamed_from: Option<String>,
    pub details: ProfileDetails,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ImportError {
    #[error("the file cannot be imported: {problem}")]
    Unreadable { problem: FileProblem },
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl ProfileSummary {
    pub(crate) fn of(id: &ProfileId, profile: &Profile, devices: Option<&[SessionDevice]>) -> Self {
        Self {
            id: id.clone(),
            name: profile.name.clone(),
            devices: profile.devices.len(),
            effects: profile.assignments.len(),
            modes: profile.modes.len(),
            external: profile.external.len(),
            resolution: devices.map(|devices| ResolutionCounts::of(&resolve(profile, devices))),
        }
    }
}

impl ResolutionCounts {
    fn of(resolutions: &[DeviceResolution]) -> Self {
        let mut counts = Self::default();
        for resolution in resolutions {
            match resolution.status {
                DeviceStatus::Resolved { .. } => counts.resolved += 1,
                DeviceStatus::Unavailable { .. } => counts.unavailable += 1,
                DeviceStatus::Missing => counts.missing += 1,
                DeviceStatus::Ambiguous => counts.ambiguous += 1,
            }
        }
        counts
    }
}

impl ProfileDetails {
    pub(crate) fn of(
        id: &ProfileId,
        profile: &Profile,
        now: Option<&SessionSnapshot>,
        apply_on_connect: Option<&ProfileId>,
    ) -> Self {
        let (devices, owners) = now.map_or((&[][..], &[][..]), |snapshot| {
            (&snapshot.devices[..], &snapshot.output.devices[..])
        });
        Self {
            id: id.clone(),
            name: profile.name.clone(),
            frame_rate: profile.frame_rate,
            apply_on_connect: apply_on_connect == Some(id),
            checked: now.is_some(),
            devices: resolve(profile, devices),
            settings: preflight(profile, devices, owners)
                .into_iter()
                .map(|checked| StoredSetting::of(profile, checked))
                .collect(),
            unsupported: profile
                .assignments
                .iter()
                .filter_map(|assignment| {
                    let problem = assignment.effect.instance().err()?;
                    Some(UnsupportedEffect {
                        device: assignment.device.clone(),
                        zone: assignment.zone,
                        problem,
                    })
                })
                .collect(),
        }
    }
}

impl StoredSetting {
    fn of(profile: &Profile, checked: crate::profile::Checked) -> Self {
        let stored = profile.device(&checked.device);
        let zone_name = match &checked.item {
            ProfileItem::Effect {
                zone: Some(zone), ..
            } => stored
                .and_then(|device| device.identity.zones.get(*zone))
                .map(|zone| zone.name.clone()),
            _ => None,
        };
        let effect_label = match &checked.item {
            ProfileItem::Effect { effect, .. } => prismforge_effects::definitions()
                .into_iter()
                .find(|definition| &definition.id == effect)
                .map(|definition| definition.label),
            _ => None,
        };
        Self {
            label: stored
                .map(|device| device.label.clone())
                .unwrap_or_default(),
            device: checked.device,
            zone_name,
            item: checked.item,
            effect_label,
            today: checked
                .skip
                .map_or(Readiness::Ready, |reason| Readiness::Skipped { reason }),
        }
    }
}
