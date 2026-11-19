use std::ops::Range;

use crate::scene::OutputTarget;
use crate::setup::ids::{ComponentId, OutputId};
use crate::setup::model::{Setup, SetupOutput};

/// The LEDs of an output that one component shows, by LED index in the output's device or zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub component: ComponentId,
    pub branch: usize,
    pub leds: Range<usize>,
}

/// What the user controls through one component: its output and LEDs; `shared_with` always shows the same colours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    pub output: OutputId,
    pub target: OutputTarget,
    pub leds: Range<usize>,
    pub shared_with: Vec<ComponentId>,
}

impl Setup {
    /// The parts of `output` in branch order; each branch starts at the output's first LED.
    pub fn parts(&self, output: &SetupOutput) -> Vec<Part> {
        let mut parts = Vec::new();
        for (branch, components) in output.branches.iter().enumerate() {
            let mut start = output.first_led as usize;
            for component in components {
                let end = start + self.led_count(component) as usize;
                parts.push(Part {
                    component: component.clone(),
                    branch,
                    leds: start..end,
                });
                start = end;
            }
        }
        parts
    }

    /// The one control a component belongs to; mirrored components report the same output and LEDs.
    pub fn control(&self, component: &ComponentId) -> Option<Control> {
        self.outputs.iter().find_map(|output| {
            let parts = self.parts(output);
            let own = parts.iter().find(|part| &part.component == component)?;
            let shared_with = parts
                .iter()
                .filter(|other| {
                    other.branch != own.branch
                        && other.leds.start < own.leds.end
                        && own.leds.start < other.leds.end
                })
                .map(|other| other.component.clone())
                .collect();
            Some(Control {
                output: output.id.clone(),
                target: output.target(),
                leds: own.leds.clone(),
                shared_with,
            })
        })
    }
}
