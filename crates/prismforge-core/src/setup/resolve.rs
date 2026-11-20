use serde::{Deserialize, Serialize};

use crate::resolution::{MemberState, target_state};
use crate::scene::OutputTarget;
use crate::session::SessionDevice;
use crate::setup::ids::OutputId;
use crate::setup::model::Setup;
use crate::setup::problem::SetupProblem;

/// One output against the devices of today, matched by its stored reference only; `problem` says why its parts do not fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "SetupOutputStatus"))]
pub struct OutputStatus {
    pub output: OutputId,
    pub target: OutputTarget,
    pub label: String,
    pub zone_name: Option<String>,
    pub state: MemberState,
    pub problem: Option<SetupProblem>,
}

impl OutputStatus {
    /// False when the zone is no longer the one the setup was made for, or the parts no longer fit it.
    pub fn matches_setup(&self) -> bool {
        self.state != MemberState::Changed && self.problem.is_none()
    }
}

pub fn resolve_outputs(setup: &Setup, devices: &[SessionDevice]) -> Vec<OutputStatus> {
    setup
        .outputs
        .iter()
        .map(|output| OutputStatus {
            output: output.id.clone(),
            target: output.target(),
            label: output.label.clone(),
            zone_name: output.zone_name.clone(),
            state: target_state(
                &output.device,
                output.zone,
                output.zone_name.as_ref(),
                devices,
            ),
            problem: setup.device_problem(output, devices),
        })
        .collect()
}
