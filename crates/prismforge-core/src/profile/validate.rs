use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::profile::model::Profile;
use crate::scene::OutputTarget;
use crate::session::{MAX_FRAME_RATE, MIN_FRAME_RATE};
use crate::store::envelope::FileProblem;

/// Longest profile name, device name, zone name or program label, in characters.
pub const NAME_LIMIT: usize = 60;
const TEXT_LIMIT: usize = 256;
const DEVICE_LIMIT: usize = 512;
const ITEM_LIMIT: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum NameProblem {
    #[error("the name is empty")]
    Empty,
    #[error("the name is longer than {max} characters")]
    TooLong { max: usize },
    #[error("the name contains a control character")]
    ControlCharacter,
}

/// Trims `name` and checks it; a name the user gave is kept exactly as trimmed.
pub fn clean_name(name: &str) -> Result<String, NameProblem> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameProblem::Empty);
    }
    if name.chars().count() > NAME_LIMIT {
        return Err(NameProblem::TooLong { max: NAME_LIMIT });
    }
    if name.chars().any(char::is_control) {
        return Err(NameProblem::ControlCharacter);
    }
    Ok(name.to_owned())
}

impl Profile {
    /// Checks names, limits, references and that each device has at most one owner.
    pub fn validate(&self) -> Result<(), FileProblem> {
        check_name("profile name", &self.name)?;
        if !(MIN_FRAME_RATE..=MAX_FRAME_RATE).contains(&self.frame_rate) {
            return Err(FileProblem::invalid(format!(
                "frame rate {} is outside {MIN_FRAME_RATE} to {MAX_FRAME_RATE}",
                self.frame_rate
            )));
        }
        if self.devices.len() > DEVICE_LIMIT
            || self.assignments.len() + self.modes.len() + self.external.len() > ITEM_LIMIT
        {
            return Err(FileProblem::invalid("the profile has too many entries"));
        }
        self.check_devices()?;
        self.check_owners()
    }

    fn check_devices(&self) -> Result<(), FileProblem> {
        let mut references = BTreeSet::new();
        for device in &self.devices {
            if !references.insert(&device.reference) {
                return Err(FileProblem::invalid(format!(
                    "device {} is listed twice",
                    device.reference
                )));
            }
            let identity = &device.identity;
            let texts = [
                Some(device.reference.as_str()),
                Some(device.label.as_str()),
                Some(identity.vendor.as_str()),
                Some(identity.model.as_str()),
                identity.serial.as_deref(),
                identity.location.as_deref(),
            ];
            let zone_names = identity.zones.iter().map(|zone| zone.name.as_str());
            if texts.into_iter().flatten().chain(zone_names).any(too_long) {
                return Err(FileProblem::invalid(format!(
                    "device {} has a text longer than {TEXT_LIMIT} characters",
                    device.reference
                )));
            }
            if identity.zones.len() > DEVICE_LIMIT {
                return Err(FileProblem::invalid(format!(
                    "device {} has too many zones",
                    device.reference
                )));
            }
        }
        Ok(())
    }

    fn check_owners(&self) -> Result<(), FileProblem> {
        let mut targets: Vec<OutputTarget> = Vec::new();
        for assignment in &self.assignments {
            let device = self.listed(&assignment.device)?;
            if let Some(zone) = assignment
                .zone
                .filter(|&zone| zone >= device.identity.zones.len())
            {
                return Err(FileProblem::invalid(format!(
                    "zone {zone} does not exist on device {}",
                    assignment.device
                )));
            }
            let target = OutputTarget {
                device: assignment.device.clone(),
                zone: assignment.zone,
            };
            if targets.iter().any(|other| other.overlaps(&target)) {
                return Err(FileProblem::invalid(format!(
                    "two effects overlap on device {}",
                    assignment.device
                )));
            }
            if assignment.effect.effect.chars().count() > TEXT_LIMIT {
                return Err(FileProblem::invalid("an effect id is too long"));
            }
            targets.push(target);
        }
        let mut owned: BTreeSet<&DeviceRef> = targets.iter().map(|target| &target.device).collect();
        for mode in &self.modes {
            self.listed(&mode.device)?;
            if too_long(&mode.name) || !owned.insert(&mode.device) {
                return Err(conflicting(&mode.device));
            }
        }
        for external in &self.external {
            self.listed(&external.device)?;
            if external.label.chars().count() > NAME_LIMIT || !owned.insert(&external.device) {
                return Err(conflicting(&external.device));
            }
        }
        Ok(())
    }

    fn listed(&self, reference: &DeviceRef) -> Result<&crate::profile::ProfileDevice, FileProblem> {
        self.device(reference).ok_or_else(|| {
            FileProblem::invalid(format!("device {reference} is used but not listed"))
        })
    }
}

fn check_name(what: &str, name: &str) -> Result<(), FileProblem> {
    match clean_name(name) {
        Ok(clean) if clean == name => Ok(()),
        Ok(_) => Err(FileProblem::invalid(format!(
            "the {what} has spaces at its ends"
        ))),
        Err(problem) => Err(FileProblem::invalid(format!("the {what}: {problem}"))),
    }
}

fn too_long(text: &str) -> bool {
    text.chars().count() > TEXT_LIMIT
}

fn conflicting(device: &DeviceRef) -> FileProblem {
    FileProblem::invalid(format!(
        "device {device} has more than one owner or an invalid entry"
    ))
}
