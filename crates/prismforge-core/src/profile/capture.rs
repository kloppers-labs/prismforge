use crate::device::{DeviceDescriptor, DeviceRef};
use crate::ownership::OutputOwner;
use crate::profile::model::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, StoredEffect,
    StoredIdentity,
};
use crate::profile::resolve::{DeviceStatus, resolve};
use crate::scene::OutputTarget;
use crate::session::SessionSnapshot;

impl Profile {
    /// The session's scene, hardware modes, declarations and frame rate as a profile named `name`.
    /// From `base` it keeps devices that do not resolve now and stored settings this version cannot run.
    pub fn capture(
        snapshot: &SessionSnapshot,
        name: &str,
        base: Option<&Profile>,
        names: &dyn Fn(&DeviceRef) -> Option<String>,
    ) -> Profile {
        let mut capture = Capture {
            snapshot,
            names,
            profile: Profile {
                name: name.to_owned(),
                frame_rate: snapshot.output.frame_rate,
                devices: Vec::new(),
                assignments: Vec::new(),
                modes: Vec::new(),
                external: Vec::new(),
                extra: base.map(|base| base.extra.clone()).unwrap_or_default(),
            },
        };
        capture.session_items();
        if let Some(base) = base {
            capture.keep_from(base);
        }
        capture.profile
    }
}

struct Capture<'a> {
    snapshot: &'a SessionSnapshot,
    names: &'a dyn Fn(&DeviceRef) -> Option<String>,
    profile: Profile,
}

impl Capture<'_> {
    fn session_items(&mut self) {
        let snapshot = self.snapshot;
        for assignment in &snapshot.output.scene.assignments {
            if self.list(&assignment.target.device) {
                self.profile.assignments.push(ProfileAssignment {
                    device: assignment.target.device.clone(),
                    zone: assignment.target.zone,
                    effect: StoredEffect::from_instance(&assignment.effect),
                    extra: Extra::new(),
                });
            }
        }
        for output in &snapshot.output.devices {
            match &output.owner {
                OutputOwner::HardwareMode { mode } => {
                    let name = self
                        .descriptor(&output.device)
                        .and_then(|descriptor| descriptor.capabilities.modes.get(*mode))
                        .map(|descriptor| descriptor.name.clone());
                    if let Some(name) = name.filter(|_| self.list(&output.device)) {
                        self.profile.modes.push(ModeSelection {
                            device: output.device.clone(),
                            index: *mode,
                            name,
                            extra: Extra::new(),
                        });
                    }
                }
                OutputOwner::External { label } if self.list(&output.device) => {
                    self.profile.external.push(ExternalControl {
                        device: output.device.clone(),
                        label: label.clone(),
                        extra: Extra::new(),
                    });
                }
                OutputOwner::External { .. } | OutputOwner::Effect => {}
            }
        }
    }

    fn keep_from(&mut self, base: &Profile) {
        for resolution in resolve(base, &self.snapshot.devices) {
            let Some(stored) = base.device(&resolution.reference) else {
                continue;
            };
            match &resolution.status {
                DeviceStatus::Resolved { device } => self.keep_unrunnable(base, stored, device),
                DeviceStatus::Unavailable { device } if !self.owns(device) => {
                    self.keep_all(base, stored, Some(device));
                }
                DeviceStatus::Unavailable { device } => self.keep_unrunnable(base, stored, device),
                DeviceStatus::Missing | DeviceStatus::Ambiguous => {
                    self.keep_all(base, stored, None)
                }
            }
        }
    }

    /// Keeps the stored settings this version cannot run where the session has nothing of its own.
    fn keep_unrunnable(&mut self, base: &Profile, stored: &ProfileDevice, device: &DeviceRef) {
        self.carry_extras(base, stored, device);
        let descriptor = self.descriptor(device).cloned();
        for assignment in base.items_of(&stored.reference) {
            let target = OutputTarget {
                device: device.clone(),
                zone: assignment.zone,
            };
            if assignment.effect.instance().is_err() && !self.covers(&target) {
                self.list(device);
                self.profile.assignments.push(ProfileAssignment {
                    device: device.clone(),
                    ..assignment.clone()
                });
            }
        }
        for mode in base
            .modes
            .iter()
            .filter(|mode| mode.device == stored.reference)
        {
            let current = descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.capabilities.modes.get(mode.index));
            let runnable = current.is_some_and(|current| current.name == mode.name);
            if !runnable && !self.owns(device) {
                self.list(device);
                self.profile.modes.push(ModeSelection {
                    device: device.clone(),
                    ..mode.clone()
                });
            }
        }
        if let Some(listed) = self
            .profile
            .devices
            .iter_mut()
            .find(|listed| &listed.reference == device)
        {
            listed.extra = stored.extra.clone();
        }
    }

    /// Unknown fields of a stored setting stay with the captured setting when both are the same.
    fn carry_extras(&mut self, base: &Profile, stored: &ProfileDevice, device: &DeviceRef) {
        for old in base.items_of(&stored.reference) {
            if let Some(new) = self.profile.assignments.iter_mut().find(|new| {
                &new.device == device
                    && new.zone == old.zone
                    && same_effect(&new.effect, &old.effect)
            }) {
                new.extra = old.extra.clone();
                new.effect.extra = old.effect.extra.clone();
            }
        }
        for old in base
            .modes
            .iter()
            .filter(|mode| mode.device == stored.reference)
        {
            if let Some(new) =
                self.profile.modes.iter_mut().find(|new| {
                    &new.device == device && new.index == old.index && new.name == old.name
                })
            {
                new.extra = old.extra.clone();
            }
        }
        for old in base
            .external
            .iter()
            .filter(|external| external.device == stored.reference)
        {
            if let Some(new) = self
                .profile
                .external
                .iter_mut()
                .find(|new| &new.device == device && new.label == old.label)
            {
                new.extra = old.extra.clone();
            }
        }
    }

    /// Keeps the stored device and all its settings unchanged, under a key no captured device uses.
    fn keep_all(&mut self, base: &Profile, stored: &ProfileDevice, linked: Option<&DeviceRef>) {
        let key = match linked {
            Some(device) => device.clone(),
            None => self.free_key(&stored.reference),
        };
        let mut device = stored.clone();
        device.reference = key.clone();
        self.profile.devices.push(device);
        for assignment in base.items_of(&stored.reference) {
            self.profile.assignments.push(ProfileAssignment {
                device: key.clone(),
                ..assignment.clone()
            });
        }
        for mode in base
            .modes
            .iter()
            .filter(|mode| mode.device == stored.reference)
        {
            self.profile.modes.push(ModeSelection {
                device: key.clone(),
                ..mode.clone()
            });
        }
        for external in base
            .external
            .iter()
            .filter(|external| external.device == stored.reference)
        {
            self.profile.external.push(ExternalControl {
                device: key.clone(),
                ..external.clone()
            });
        }
    }

    /// Lists the session device once; false when the session has no description of it.
    fn list(&mut self, device: &DeviceRef) -> bool {
        if self.profile.device(device).is_some() {
            return true;
        }
        let Some(descriptor) = self.descriptor(device) else {
            return false;
        };
        let label = (self.names)(device).unwrap_or_else(|| descriptor.name.clone());
        let identity = StoredIdentity::from_fingerprint(&descriptor.fingerprint());
        self.profile.devices.push(ProfileDevice {
            reference: device.clone(),
            label,
            identity,
            extra: Extra::new(),
        });
        true
    }

    fn descriptor(&self, device: &DeviceRef) -> Option<&DeviceDescriptor> {
        self.snapshot
            .devices
            .iter()
            .find(|candidate| candidate.identity.reference() == Some(device))
            .map(|candidate| &candidate.descriptor)
    }

    fn owns(&self, device: &DeviceRef) -> bool {
        self.profile
            .assignments
            .iter()
            .any(|assignment| &assignment.device == device)
            || self.profile.modes.iter().any(|mode| &mode.device == device)
            || self
                .profile
                .external
                .iter()
                .any(|external| &external.device == device)
    }

    fn covers(&self, target: &OutputTarget) -> bool {
        self.profile.assignments.iter().any(|assignment| {
            OutputTarget {
                device: assignment.device.clone(),
                zone: assignment.zone,
            }
            .overlaps(target)
        }) || self
            .profile
            .modes
            .iter()
            .any(|mode| mode.device == target.device)
            || self
                .profile
                .external
                .iter()
                .any(|external| external.device == target.device)
    }

    fn free_key(&self, wanted: &DeviceRef) -> DeviceRef {
        let taken = |key: &DeviceRef| {
            self.profile.device(key).is_some()
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
}

fn same_effect(new: &StoredEffect, old: &StoredEffect) -> bool {
    new.effect == old.effect
        && old
            .values
            .iter()
            .all(|(control, value)| new.values.get(control) == Some(value))
}

impl Profile {
    fn items_of<'a>(
        &'a self,
        device: &'a DeviceRef,
    ) -> impl Iterator<Item = &'a ProfileAssignment> {
        self.assignments
            .iter()
            .filter(move |assignment| &assignment.device == device)
    }
}
