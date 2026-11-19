use std::collections::BTreeMap;

use serde_json::Value;

use crate::device::DeviceRef;
use crate::profile::apply::ProfileItem;
use crate::profile::changes::{Change, FrameRateChange, SettingChange};
use crate::profile::model::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, StoredEffect,
    StoredIdentity,
};
use crate::profile::resolve::resolve;
use crate::session::{Intent, IntentRecord, SessionDevice, Slot};

/// A profile after an update, with every setting the update changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Updated {
    pub profile: Profile,
    pub changes: Vec<SettingChange>,
    pub frame_rate: Option<FrameRateChange>,
}

/// Applies the user's records in order; everything they do not touch stays exactly as stored.
pub(crate) fn update(
    stored: &Profile,
    records: &[IntentRecord],
    devices: &[SessionDevice],
    names: &dyn Fn(&DeviceRef) -> Option<String>,
) -> Updated {
    let mut links: Vec<(DeviceRef, DeviceRef)> = resolve(stored, devices)
        .into_iter()
        .filter_map(|resolution| Some((resolution.status.device()?.clone(), resolution.reference)))
        .collect();
    let mut profile = stored.clone();
    let mut added: Vec<DeviceRef> = Vec::new();
    for record in records {
        if let (Slot::FrameRate, Intent::FrameRate(frame_rate)) = (&record.slot, &record.intent) {
            profile.frame_rate = *frame_rate;
            continue;
        }
        let device = match &record.slot {
            Slot::Effect(target) => &target.device,
            Slot::Mode(device) | Slot::External(device) => device,
            Slot::FrameRate => continue,
        };
        let key = match links.iter().find(|(session, _)| session == device) {
            Some((_, key)) => key.clone(),
            None => match list_new(&mut profile, device, devices, names) {
                Some(key) => {
                    links.push((device.clone(), key.clone()));
                    added.push(key.clone());
                    key
                }
                None => continue,
            },
        };
        let zone = match &record.slot {
            Slot::Effect(target) => target.zone,
            _ => None,
        };
        perform(&mut profile, &key, zone, &record.intent);
    }
    let used: Vec<DeviceRef> = settings_of(&profile)
        .into_iter()
        .map(|(key, _, _)| key.device().clone())
        .collect();
    profile
        .devices
        .retain(|device| !added.contains(&device.reference) || used.contains(&device.reference));
    Updated {
        changes: changes(stored, &profile),
        frame_rate: (profile.frame_rate != stored.frame_rate).then_some(FrameRateChange {
            was: stored.frame_rate,
            now: profile.frame_rate,
        }),
        profile,
    }
}

/// Every setting of `profile` as an empty profile would gain it.
pub(crate) fn all_added(profile: &Profile) -> Vec<SettingChange> {
    let empty = Profile {
        devices: profile.devices.clone(),
        assignments: Vec::new(),
        modes: Vec::new(),
        external: Vec::new(),
        ..profile.clone()
    };
    changes(&empty, profile)
}

fn perform(profile: &mut Profile, key: &DeviceRef, zone: Option<usize>, intent: &Intent) {
    let on_device = |device: &DeviceRef| device == key;
    match intent {
        Intent::Applied {
            effect: instance, ..
        } => {
            profile.modes.retain(|mode| !on_device(&mode.device));
            profile
                .external
                .retain(|external| !on_device(&external.device));
            let effect = StoredEffect::from_instance(instance);
            let same = profile
                .assignments
                .iter()
                .position(|assignment| on_device(&assignment.device) && assignment.zone == zone);
            if let Some(index) = same {
                let assignment = &mut profile.assignments[index];
                let extra = if assignment.effect.effect == effect.effect {
                    assignment.effect.extra.clone()
                } else {
                    Extra::new()
                };
                assignment.effect = StoredEffect { extra, ..effect };
                let kept = assignment.clone();
                profile.assignments.retain(|other| {
                    other == &kept || !(on_device(&other.device) && overlaps(other.zone, zone))
                });
            } else {
                profile
                    .assignments
                    .retain(|other| !(on_device(&other.device) && overlaps(other.zone, zone)));
                profile.assignments.push(ProfileAssignment {
                    device: key.clone(),
                    zone,
                    effect,
                    extra: Extra::new(),
                });
            }
        }
        Intent::Stopped => profile.assignments.retain(|assignment| {
            !(on_device(&assignment.device) && (zone.is_none() || assignment.zone == zone))
        }),
        Intent::ModeSelected { index, name } => {
            profile
                .assignments
                .retain(|assignment| !on_device(&assignment.device));
            profile
                .external
                .retain(|external| !on_device(&external.device));
            match profile
                .modes
                .iter_mut()
                .find(|mode| on_device(&mode.device))
            {
                Some(mode) => {
                    mode.index = *index;
                    mode.name = name.clone();
                }
                None => profile.modes.push(ModeSelection {
                    device: key.clone(),
                    index: *index,
                    name: name.clone(),
                    extra: Extra::new(),
                }),
            }
        }
        Intent::ModeEnded => profile.modes.retain(|mode| !on_device(&mode.device)),
        Intent::Declared { label } => {
            profile
                .assignments
                .retain(|assignment| !on_device(&assignment.device));
            profile.modes.retain(|mode| !on_device(&mode.device));
            match profile
                .external
                .iter_mut()
                .find(|external| on_device(&external.device))
            {
                Some(external) => external.label = label.clone(),
                None => profile.external.push(ExternalControl {
                    device: key.clone(),
                    label: label.clone(),
                    extra: Extra::new(),
                }),
            }
        }
        Intent::HandedBack => profile
            .external
            .retain(|external| !on_device(&external.device)),
        Intent::FrameRate(_) => {}
    }
}

fn overlaps(left: Option<usize>, right: Option<usize>) -> bool {
    left.is_none() || right.is_none() || left == right
}

/// Lists a session device the profile does not have yet, under a key no stored device uses.
fn list_new(
    profile: &mut Profile,
    device: &DeviceRef,
    devices: &[SessionDevice],
    names: &dyn Fn(&DeviceRef) -> Option<String>,
) -> Option<DeviceRef> {
    let entry = devices
        .iter()
        .find(|candidate| candidate.identity.reference() == Some(device))?;
    let key = std::iter::once(device.clone())
        .chain((2..).map(|number| DeviceRef::new(format!("{device}-{number}"))))
        .find(|key| profile.device(key).is_none())
        .expect("some key is free");
    profile.devices.push(ProfileDevice {
        reference: key.clone(),
        label: names(device).unwrap_or_else(|| entry.descriptor.name.clone()),
        identity: StoredIdentity::from_fingerprint(&entry.descriptor.fingerprint()),
        extra: Extra::new(),
    });
    Some(key)
}

type Values = BTreeMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SettingKey {
    Effect(DeviceRef, Option<usize>),
    Mode(DeviceRef),
    External(DeviceRef),
}

impl SettingKey {
    fn device(&self) -> &DeviceRef {
        match self {
            Self::Effect(device, _) | Self::Mode(device) | Self::External(device) => device,
        }
    }
}

/// Each setting with its key and the content an update compares; unknown fields are not content.
fn settings_of(profile: &Profile) -> Vec<(SettingKey, ProfileItem, Option<Values>)> {
    let effects = profile.assignments.iter().map(|assignment| {
        (
            SettingKey::Effect(assignment.device.clone(), assignment.zone),
            ProfileItem::Effect {
                zone: assignment.zone,
                effect: assignment.effect.effect.clone(),
            },
            Some(assignment.effect.values.clone()),
        )
    });
    let modes = profile.modes.iter().map(|mode| {
        let item = ProfileItem::HardwareMode {
            index: mode.index,
            name: mode.name.clone(),
        };
        (SettingKey::Mode(mode.device.clone()), item, None)
    });
    let external = profile.external.iter().map(|external| {
        let item = ProfileItem::External {
            label: external.label.clone(),
        };
        (SettingKey::External(external.device.clone()), item, None)
    });
    effects.chain(modes).chain(external).collect()
}

fn changes(before: &Profile, after: &Profile) -> Vec<SettingChange> {
    let old = settings_of(before);
    let new = settings_of(after);
    let label = |key: &SettingKey| {
        after
            .device(key.device())
            .or_else(|| before.device(key.device()))
            .map(|device| device.label.clone())
            .unwrap_or_default()
    };
    let mut changes = Vec::new();
    for (key, was, content) in &old {
        let change = match new.iter().find(|(other, _, _)| other == key) {
            None => Change::Removed { was: was.clone() },
            Some((_, now, now_content)) if now != was || now_content != content => {
                Change::Replaced {
                    was: was.clone(),
                    now: now.clone(),
                }
            }
            Some(_) => continue,
        };
        changes.push(SettingChange {
            device: key.device().clone(),
            label: label(key),
            change,
        });
    }
    for (key, now, _) in &new {
        if !old.iter().any(|(other, _, _)| other == key) {
            changes.push(SettingChange {
                device: key.device().clone(),
                label: label(key),
                change: Change::Added { now: now.clone() },
            });
        }
    }
    changes
}
