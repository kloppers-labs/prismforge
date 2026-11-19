use serde::{Deserialize, Serialize};

use crate::resolution::{MemberState, target_state};
use crate::scene::OutputTarget;
use crate::session::SessionDevice;
use crate::store::groups::model::GroupMember;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MemberStatus {
    pub target: OutputTarget,
    pub label: String,
    pub zone_name: Option<String>,
    pub state: MemberState,
}

/// Matches members by their stored reference only, never by name or position.
pub fn resolve_members(members: &[GroupMember], devices: &[SessionDevice]) -> Vec<MemberStatus> {
    members
        .iter()
        .map(|member| MemberStatus {
            target: member.target(),
            label: member.label.clone(),
            zone_name: member.zone_name.clone(),
            state: state(member, devices),
        })
        .collect()
}

fn state(member: &GroupMember, devices: &[SessionDevice]) -> MemberState {
    target_state(
        &member.device,
        member.zone,
        member.zone_name.as_ref(),
        devices,
    )
}
