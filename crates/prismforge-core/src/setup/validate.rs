use std::collections::BTreeSet;
use std::ops::Range;

use crate::profile::{NameProblem, clean_name};
use crate::session::{MAX_BATCH_TARGETS, SessionDevice};
use crate::setup::model::{Setup, SetupOutput};
use crate::setup::problem::{SetupPart, SetupProblem};
use crate::store::{MemberState, target_state};

/// Most components one setup holds.
pub const COMPONENT_LIMIT: usize = 512;
/// Most outputs one setup holds; each is at most one target of a batch.
pub const OUTPUT_LIMIT: usize = MAX_BATCH_TARGETS;
/// Most views one setup holds.
pub const VIEW_LIMIT: usize = 16;
/// Most branches one output has.
pub const BRANCH_LIMIT: usize = 16;
const TEXT_LIMIT: usize = 256;

impl Setup {
    /// Checks ids, names, shapes, wiring and placements; needs no device.
    pub fn validate(&self) -> Result<(), SetupProblem> {
        limit(
            self.components.len(),
            COMPONENT_LIMIT,
            SetupPart::Components,
        )?;
        limit(self.outputs.len(), OUTPUT_LIMIT, SetupPart::Outputs)?;
        limit(self.views.len(), VIEW_LIMIT, SetupPart::Views)?;
        self.check_ids()?;
        for component in &self.components {
            check_name(&component.name)?;
            if !component.geometry.is_valid() {
                return Err(SetupProblem::InvalidGeometry {
                    component: component.id.clone(),
                });
            }
        }
        self.check_wiring()?;
        self.check_views()
    }

    /// Checks each output of a device the session knows against the LED counts the backend reports for it.
    pub fn check_devices(&self, devices: &[SessionDevice]) -> Result<(), SetupProblem> {
        self.outputs
            .iter()
            .find_map(|output| self.device_problem(output, devices))
            .map_or(Ok(()), Err)
    }

    /// Why the parts of `output` do not fit the device or zone it names now, if it names one the session knows.
    pub(crate) fn device_problem(
        &self,
        output: &SetupOutput,
        devices: &[SessionDevice],
    ) -> Option<SetupProblem> {
        let state = target_state(
            &output.device,
            output.zone,
            output.zone_name.as_ref(),
            devices,
        );
        if !matches!(state, MemberState::Resolved | MemberState::Unavailable) {
            return None;
        }
        let (max, available) = capacity(output, devices)?;
        for component in output.branches.iter().flatten() {
            let leds = self.led_count(component);
            if u64::from(leds) > max {
                return Some(SetupProblem::LedCountOutsideBounds {
                    component: component.clone(),
                    leds,
                    min: 1,
                    max,
                });
            }
        }
        let needed = u64::from(output.first_led) + self.span(output);
        (needed > available).then(|| SetupProblem::ExceedsOutput {
            output: output.id.clone(),
            needed,
            available,
        })
    }

    fn check_ids(&self) -> Result<(), SetupProblem> {
        let components = self
            .components
            .iter()
            .map(|item| (item.id.is_valid(), item.id.as_str()));
        let outputs = self
            .outputs
            .iter()
            .map(|item| (item.id.is_valid(), item.id.as_str()));
        let views = self
            .views
            .iter()
            .map(|item| (item.id.is_valid(), item.id.as_str()));
        let mut seen = BTreeSet::new();
        for (valid, id) in components.chain(outputs).chain(views) {
            if !valid {
                return Err(SetupProblem::InvalidId { id: id.to_owned() });
            }
            if !seen.insert(id) {
                return Err(SetupProblem::DuplicateId { id: id.to_owned() });
            }
        }
        Ok(())
    }

    fn check_wiring(&self) -> Result<(), SetupProblem> {
        let mut wired = BTreeSet::new();
        for output in &self.outputs {
            check_text(&output.label)?;
            if let Some(zone_name) = &output.zone_name {
                check_text(zone_name)?;
            }
            limit(output.branches.len(), BRANCH_LIMIT, SetupPart::Branches)?;
            if output.branches.is_empty() || output.branches.iter().any(Vec::is_empty) {
                return Err(SetupProblem::EmptyOutput {
                    output: output.id.clone(),
                });
            }
            for component in output.branches.iter().flatten() {
                if self.component(component).is_none() {
                    return Err(SetupProblem::UnknownComponent {
                        component: component.clone(),
                    });
                }
                if !wired.insert(component) {
                    return Err(SetupProblem::WiredTwice {
                        component: component.clone(),
                    });
                }
            }
        }
        for (index, output) in self.outputs.iter().enumerate() {
            if let Some(other) = self.outputs[..index]
                .iter()
                .find(|other| self.overlap(other, output))
            {
                return Err(SetupProblem::OverlappingParts {
                    first: other.id.clone(),
                    second: output.id.clone(),
                });
            }
        }
        Ok(())
    }

    fn check_views(&self) -> Result<(), SetupProblem> {
        for view in &self.views {
            check_name(&view.name)?;
            limit(
                view.placements.len(),
                COMPONENT_LIMIT,
                SetupPart::Placements,
            )?;
            let mut placed = BTreeSet::new();
            for placement in &view.placements {
                let component = &placement.component;
                if self.component(component).is_none() {
                    return Err(SetupProblem::UnknownComponent {
                        component: component.clone(),
                    });
                }
                if !placed.insert(component) {
                    return Err(SetupProblem::PlacedTwice {
                        view: view.id.clone(),
                        component: component.clone(),
                    });
                }
                if placement.size == 0 || placement.rotation >= 360 {
                    return Err(SetupProblem::InvalidPlacement {
                        view: view.id.clone(),
                        component: component.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn overlap(&self, left: &SetupOutput, right: &SetupOutput) -> bool {
        if left.device != right.device {
            return false;
        }
        match (left.zone, right.zone) {
            (Some(one), Some(other)) if one != other => false,
            (Some(_), None) | (None, Some(_)) => true,
            _ => {
                let (one, other) = (self.leds(left), self.leds(right));
                one.start < other.end && other.start < one.end
            }
        }
    }

    fn leds(&self, output: &SetupOutput) -> Range<u64> {
        let first = u64::from(output.first_led);
        first..first + self.span(output)
    }
}

/// The most LEDs the backend allows for the output, and how many it has now.
pub(crate) fn capacity(output: &SetupOutput, devices: &[SessionDevice]) -> Option<(u64, u64)> {
    let device = devices
        .iter()
        .find(|device| device.identity.reference() == Some(&output.device))?;
    let descriptor = &device.descriptor;
    match output.zone {
        Some(zone) => {
            let zone = descriptor.zones.get(zone)?;
            Some((u64::from(zone.leds_max), zone.led_count as u64))
        }
        None => Some((
            descriptor
                .zones
                .iter()
                .map(|zone| u64::from(zone.leds_max))
                .sum(),
            descriptor.led_count as u64,
        )),
    }
}

fn limit(count: usize, limit: usize, part: SetupPart) -> Result<(), SetupProblem> {
    if count > limit {
        return Err(SetupProblem::TooMany { part, limit });
    }
    Ok(())
}

fn check_name(name: &str) -> Result<(), SetupProblem> {
    clean_name(name)
        .map(drop)
        .map_err(|problem| SetupProblem::InvalidName { problem })
}

fn check_text(text: &str) -> Result<(), SetupProblem> {
    if text.chars().count() > TEXT_LIMIT {
        return Err(SetupProblem::InvalidName {
            problem: NameProblem::TooLong { max: TEXT_LIMIT },
        });
    }
    Ok(())
}
