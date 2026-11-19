use crate::device::DeviceRef;
use crate::profile::{Extra, Profile, StoredIdentity};

/// A copy without serial numbers, locations, local references, the user's device names or unknown fields.
pub(crate) fn portable(profile: &Profile) -> (Profile, usize) {
    let mut omitted = profile.extra.len();
    let key = |reference: &DeviceRef| {
        let position = profile
            .devices
            .iter()
            .position(|device| &device.reference == reference)
            .expect("validated profiles list every device they use");
        DeviceRef::new(format!("device-{}", position + 1))
    };
    let devices = profile
        .devices
        .iter()
        .map(|device| {
            omitted += device.extra.len()
                + device
                    .identity
                    .zones
                    .iter()
                    .map(|zone| zone.extra.len())
                    .sum::<usize>();
            let mut copy = device.clone();
            copy.reference = key(&device.reference);
            copy.label = device.identity.model.clone();
            copy.identity = StoredIdentity {
                serial: None,
                location: None,
                ..device.identity.clone()
            };
            for zone in &mut copy.identity.zones {
                zone.extra = Extra::new();
            }
            copy.extra = Extra::new();
            copy
        })
        .collect();
    let assignments = profile
        .assignments
        .iter()
        .map(|assignment| {
            omitted += assignment.extra.len() + assignment.effect.extra.len();
            let mut copy = assignment.clone();
            copy.device = key(&assignment.device);
            copy.extra = Extra::new();
            copy.effect.extra = Extra::new();
            copy
        })
        .collect();
    let modes = profile
        .modes
        .iter()
        .map(|mode| {
            omitted += mode.extra.len();
            let mut copy = mode.clone();
            copy.device = key(&mode.device);
            copy.extra = Extra::new();
            copy
        })
        .collect();
    let external = profile
        .external
        .iter()
        .map(|external| {
            omitted += external.extra.len();
            let mut copy = external.clone();
            copy.device = key(&external.device);
            copy.extra = Extra::new();
            copy
        })
        .collect();
    let copy = Profile {
        name: profile.name.clone(),
        frame_rate: profile.frame_rate,
        devices,
        assignments,
        modes,
        external,
        extra: Extra::new(),
    };
    (copy, omitted)
}
