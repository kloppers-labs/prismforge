use crate::device::DeviceRef;
use crate::ownership::OutputOwner;
use crate::profile::model::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, StoredEffect,
    StoredIdentity,
};
use crate::session::SessionSnapshot;

impl Profile {
    /// The session's scene, hardware modes, declarations and frame rate as a new profile named `name`.
    pub fn capture(
        snapshot: &SessionSnapshot,
        name: &str,
        names: &dyn Fn(&DeviceRef) -> Option<String>,
    ) -> Profile {
        let mut profile = Profile {
            name: name.to_owned(),
            frame_rate: snapshot.output.frame_rate,
            devices: Vec::new(),
            assignments: Vec::new(),
            modes: Vec::new(),
            external: Vec::new(),
            extra: Extra::new(),
        };
        let list = |profile: &mut Profile, device: &DeviceRef| -> bool {
            if profile.device(device).is_some() {
                return true;
            }
            let Some(entry) = snapshot
                .devices
                .iter()
                .find(|candidate| candidate.identity.reference() == Some(device))
            else {
                return false;
            };
            profile.devices.push(ProfileDevice {
                reference: device.clone(),
                label: names(device).unwrap_or_else(|| entry.descriptor.name.clone()),
                identity: StoredIdentity::from_fingerprint(&entry.descriptor.fingerprint()),
                extra: Extra::new(),
            });
            true
        };
        for assignment in &snapshot.output.scene.assignments {
            if list(&mut profile, &assignment.target.device) {
                profile.assignments.push(ProfileAssignment {
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
                    let name = snapshot
                        .devices
                        .iter()
                        .find(|candidate| candidate.identity.reference() == Some(&output.device))
                        .and_then(|entry| entry.descriptor.capabilities.modes.get(*mode))
                        .map(|descriptor| descriptor.name.clone());
                    if let Some(name) = name
                        && list(&mut profile, &output.device)
                    {
                        profile.modes.push(ModeSelection {
                            device: output.device.clone(),
                            index: *mode,
                            name,
                            extra: Extra::new(),
                        });
                    }
                }
                OutputOwner::External { label } => {
                    if list(&mut profile, &output.device) {
                        profile.external.push(ExternalControl {
                            device: output.device.clone(),
                            label: label.clone(),
                            extra: Extra::new(),
                        });
                    }
                }
                OutputOwner::Effect => {}
            }
        }
        profile
    }
}
