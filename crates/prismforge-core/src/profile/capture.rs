use crate::device::{DeviceDescriptor, DeviceRef};
use crate::ownership::OutputOwner;
use crate::profile::apply::ProfileItem;
use crate::profile::changes::{Change, KeepReason, SettingChange};
use crate::profile::history::{SessionHistory, Target};
use crate::profile::model::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, StoredEffect,
    StoredIdentity,
};
use crate::profile::resolve::{DeviceStatus, resolve};
use crate::session::SessionSnapshot;

/// A profile built from the session, what happened to each setting, and the history for the next update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Captured {
    pub profile: Profile,
    pub changes: Vec<SettingChange>,
    pub history: SessionHistory,
}

impl Profile {
    /// The session's scene, modes, declarations and frame rate; a stored setting of `base` is only
    /// replaced or removed where the user changed its target in this session, as `history` shows.
    pub fn capture(
        snapshot: &SessionSnapshot,
        name: &str,
        base: Option<&Profile>,
        history: &SessionHistory,
        names: &dyn Fn(&DeviceRef) -> Option<String>,
    ) -> Captured {
        let empty = Profile {
            name: name.to_owned(),
            frame_rate: snapshot.output.frame_rate,
            devices: Vec::new(),
            assignments: Vec::new(),
            modes: Vec::new(),
            external: Vec::new(),
            extra: Extra::new(),
        };
        let base = base.unwrap_or(&empty);
        let mut capture = Capture {
            snapshot,
            history,
            names,
            base,
            statuses: resolve(base, &snapshot.devices)
                .into_iter()
                .map(|resolution| (resolution.reference, resolution.status))
                .collect(),
            session: session_settings(snapshot),
            used: Vec::new(),
            kept: Vec::new(),
            keys: Vec::new(),
            settings: Vec::new(),
            changes: Vec::new(),
        };
        capture.used = vec![false; capture.session.len()];
        for setting in settings_of(base) {
            capture.decide(setting);
        }
        capture.add_session_settings();
        let live = capture.session.iter().map(Setting::target).collect();
        Captured {
            profile: capture.build(name, base),
            changes: capture.changes,
            history: history.after_capture(snapshot, live),
        }
    }
}

/// One stored setting of any kind; its `device` is a key in the profile it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Setting {
    Effect(ProfileAssignment),
    Mode(ModeSelection),
    External(ExternalControl),
}

struct Capture<'a> {
    snapshot: &'a SessionSnapshot,
    history: &'a SessionHistory,
    names: &'a dyn Fn(&DeviceRef) -> Option<String>,
    base: &'a Profile,
    statuses: Vec<(DeviceRef, DeviceStatus)>,
    session: Vec<Setting>,
    used: Vec<bool>,
    kept: Vec<Target>,
    keys: Vec<(DeviceRef, DeviceRef)>,
    settings: Vec<Setting>,
    changes: Vec<SettingChange>,
}

impl Capture<'_> {
    fn decide(&mut self, stored: Setting) {
        let device = stored.device().clone();
        let status = self
            .statuses
            .iter()
            .find(|(reference, _)| reference == &device)
            .map_or(DeviceStatus::Missing, |(_, status)| status.clone());
        let linked = match &status {
            DeviceStatus::Resolved { device } | DeviceStatus::Unavailable { device } => device,
            DeviceStatus::Missing => return self.keep(stored, KeepReason::Missing, None),
            DeviceStatus::Ambiguous => return self.keep(stored, KeepReason::Ambiguous, None),
        };
        let here = stored.with_device(linked);
        let target = here.target();
        if let Some(index) = self.unused_equal(&here) {
            self.used[index] = true;
            return self.keep_as(stored, Some(linked), Change::Unchanged);
        }
        let covered = self
            .session
            .iter()
            .any(|setting| setting.target().overlaps(&target));
        let untouched = self.history.untouched(linked, self.snapshot);
        let change = match (untouched, covered) {
            (false, true) => Change::Replaced,
            (false, false) if self.history.was_live(&target) => Change::Removed,
            _ => {
                let reason = match status {
                    DeviceStatus::Unavailable { .. } => KeepReason::Unavailable,
                    _ => KeepReason::NotRunning {
                        skipped: self.history.skip_reason(&target),
                    },
                };
                self.kept.push(target);
                return self.keep(stored, reason, Some(linked));
            }
        };
        let label = self.stored_label(&device);
        self.changes.push(SettingChange {
            device,
            label,
            item: stored.item(),
            change,
        });
    }

    fn keep(&mut self, stored: Setting, reason: KeepReason, linked: Option<&DeviceRef>) {
        self.keep_as(stored, linked, Change::Kept { reason });
    }

    fn keep_as(&mut self, stored: Setting, linked: Option<&DeviceRef>, change: Change) {
        let key = self.key_for(stored.device(), linked);
        let label = self.stored_label(stored.device());
        self.changes.push(SettingChange {
            device: key.clone(),
            label,
            item: stored.item(),
            change,
        });
        self.settings.push(stored.with_device(&key));
    }

    /// Adds what the session runs that no stored setting kept; a state that blocked a kept setting stays out.
    fn add_session_settings(&mut self) {
        for (index, setting) in self.session.clone().into_iter().enumerate() {
            let target = setting.target();
            if self.used[index] || self.kept.iter().any(|kept| kept.overlaps(&target)) {
                continue;
            }
            let device = setting.device().clone();
            let label = (self.names)(&device)
                .or_else(|| {
                    self.descriptor(&device)
                        .map(|descriptor| descriptor.name.clone())
                })
                .unwrap_or_default();
            self.changes.push(SettingChange {
                device,
                label,
                item: setting.item(),
                change: Change::Added,
            });
            self.settings.push(setting);
        }
    }

    fn build(&self, name: &str, base: &Profile) -> Profile {
        let mut profile = Profile {
            name: name.to_owned(),
            frame_rate: self.snapshot.output.frame_rate,
            devices: Vec::new(),
            assignments: Vec::new(),
            modes: Vec::new(),
            external: Vec::new(),
            extra: base.extra.clone(),
        };
        for stored in &base.devices {
            let Some((_, key)) = self.keys.iter().find(|(from, _)| from == &stored.reference)
            else {
                continue;
            };
            if self.uses(key) {
                profile.devices.push(self.refreshed(stored, key));
            }
        }
        for setting in &self.settings {
            let device = setting.device();
            if profile.device(device).is_none()
                && let Some(descriptor) = self.descriptor(device)
            {
                profile.devices.push(ProfileDevice {
                    reference: device.clone(),
                    label: (self.names)(device).unwrap_or_else(|| descriptor.name.clone()),
                    identity: StoredIdentity::from_fingerprint(&descriptor.fingerprint()),
                    extra: Extra::new(),
                });
            }
            match setting.clone() {
                Setting::Effect(assignment) => profile.assignments.push(assignment),
                Setting::Mode(mode) => profile.modes.push(mode),
                Setting::External(external) => profile.external.push(external),
            }
        }
        profile
    }

    /// A stored device under its key now; a linked device gets today's identifying fields and the user's name.
    fn refreshed(&self, stored: &ProfileDevice, key: &DeviceRef) -> ProfileDevice {
        let mut device = stored.clone();
        device.reference = key.clone();
        if let Some(descriptor) = self.descriptor(key) {
            device.identity = stored.identity.refreshed(&descriptor.fingerprint());
            if let Some(name) = (self.names)(key) {
                device.label = name;
            }
        }
        device
    }

    fn key_for(&mut self, stored: &DeviceRef, linked: Option<&DeviceRef>) -> DeviceRef {
        if let Some((_, key)) = self.keys.iter().find(|(from, _)| from == stored) {
            return key.clone();
        }
        let key = match linked {
            Some(device) => device.clone(),
            None => self.free_key(stored),
        };
        self.keys.push((stored.clone(), key.clone()));
        key
    }

    fn free_key(&self, wanted: &DeviceRef) -> DeviceRef {
        let taken = |key: &DeviceRef| {
            self.keys.iter().any(|(_, used)| used == key)
                || self
                    .snapshot
                    .devices
                    .iter()
                    .any(|device| device.identity.reference() == Some(key))
        };
        if !taken(wanted) {
            return wanted.clone();
        }
        (2..)
            .map(|number| DeviceRef::new(format!("{wanted}-kept-{number}")))
            .find(|key| !taken(key))
            .expect("some number is free")
    }

    fn unused_equal(&self, stored: &Setting) -> Option<usize> {
        self.session
            .iter()
            .enumerate()
            .find(|(index, setting)| !self.used[*index] && setting.runs(stored))
            .map(|(index, _)| index)
    }

    fn uses(&self, key: &DeviceRef) -> bool {
        self.settings.iter().any(|setting| setting.device() == key)
    }

    fn stored_label(&self, device: &DeviceRef) -> String {
        self.base
            .device(device)
            .map(|stored| stored.label.clone())
            .unwrap_or_default()
    }

    fn descriptor(&self, device: &DeviceRef) -> Option<&DeviceDescriptor> {
        self.snapshot
            .devices
            .iter()
            .find(|candidate| candidate.identity.reference() == Some(device))
            .map(|candidate| &candidate.descriptor)
    }
}

impl Setting {
    fn device(&self) -> &DeviceRef {
        match self {
            Self::Effect(assignment) => &assignment.device,
            Self::Mode(mode) => &mode.device,
            Self::External(external) => &external.device,
        }
    }

    fn with_device(&self, device: &DeviceRef) -> Self {
        let mut copy = self.clone();
        match &mut copy {
            Self::Effect(assignment) => assignment.device = device.clone(),
            Self::Mode(mode) => mode.device = device.clone(),
            Self::External(external) => external.device = device.clone(),
        }
        copy
    }

    fn item(&self) -> ProfileItem {
        match self {
            Self::Effect(assignment) => ProfileItem::Effect {
                zone: assignment.zone,
                effect: assignment.effect.effect.clone(),
            },
            Self::Mode(mode) => ProfileItem::HardwareMode {
                index: mode.index,
                name: mode.name.clone(),
            },
            Self::External(external) => ProfileItem::External {
                label: external.label.clone(),
            },
        }
    }

    fn target(&self) -> Target {
        Target::of(&self.item(), self.device())
    }

    /// True when this session setting is the stored setting `stored`, ignoring unknown fields.
    fn runs(&self, stored: &Self) -> bool {
        match (self, stored) {
            (Self::Effect(now), Self::Effect(stored)) => {
                now.device == stored.device
                    && now.zone == stored.zone
                    && same_effect(&now.effect, &stored.effect)
            }
            (Self::Mode(now), Self::Mode(stored)) => {
                now.device == stored.device && now.index == stored.index && now.name == stored.name
            }
            (Self::External(now), Self::External(stored)) => {
                now.device == stored.device && now.label == stored.label
            }
            _ => false,
        }
    }
}

fn same_effect(now: &StoredEffect, stored: &StoredEffect) -> bool {
    now.effect == stored.effect
        && stored
            .values
            .iter()
            .all(|(control, value)| now.values.get(control) == Some(value))
}

fn settings_of(profile: &Profile) -> Vec<Setting> {
    let effects = profile.assignments.iter().cloned().map(Setting::Effect);
    let modes = profile.modes.iter().cloned().map(Setting::Mode);
    let external = profile.external.iter().cloned().map(Setting::External);
    effects.chain(modes).chain(external).collect()
}

fn session_settings(snapshot: &SessionSnapshot) -> Vec<Setting> {
    let described = |device: &DeviceRef| {
        snapshot
            .devices
            .iter()
            .find(|candidate| candidate.identity.reference() == Some(device))
            .map(|candidate| &candidate.descriptor)
    };
    let effects = snapshot
        .output
        .scene
        .assignments
        .iter()
        .filter(|assignment| described(&assignment.target.device).is_some())
        .map(|assignment| {
            Setting::Effect(ProfileAssignment {
                device: assignment.target.device.clone(),
                zone: assignment.target.zone,
                effect: StoredEffect::from_instance(&assignment.effect),
                extra: Extra::new(),
            })
        });
    let owners = snapshot.output.devices.iter().filter_map(|output| {
        let descriptor = described(&output.device)?;
        match &output.owner {
            OutputOwner::HardwareMode { mode } => {
                let name = descriptor.capabilities.modes.get(*mode)?.name.clone();
                Some(Setting::Mode(ModeSelection {
                    device: output.device.clone(),
                    index: *mode,
                    name,
                    extra: Extra::new(),
                }))
            }
            OutputOwner::External { label } => Some(Setting::External(ExternalControl {
                device: output.device.clone(),
                label: label.clone(),
                extra: Extra::new(),
            })),
            OutputOwner::Effect => None,
        }
    });
    effects.chain(owners).collect()
}
