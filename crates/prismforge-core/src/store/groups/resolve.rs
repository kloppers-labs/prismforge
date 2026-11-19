use serde::{Deserialize, Serialize};

use crate::inventory::DeviceIdentity;
use crate::scene::OutputTarget;
use crate::session::SessionDevice;
use crate::store::groups::model::GroupMember;

/// How a member relates to the devices of today; only `Resolved` can be selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum MemberState {
    Resolved,
    /// The device is known but not reported now.
    Unavailable,
    Missing,
    /// More than one device could be this one, so none is used.
    Ambiguous,
    /// The zone at this index is not the zone that joined the group.
    Changed,
}

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
    let ambiguous = devices.iter().any(|device| {
        matches!(&device.identity, DeviceIdentity::Ambiguous { candidates } if candidates.contains(&member.device))
    });
    if ambiguous {
        return MemberState::Ambiguous;
    }
    let Some(device) = devices
        .iter()
        .find(|device| device.identity.reference() == Some(&member.device))
    else {
        return MemberState::Missing;
    };
    if let Some(zone) = member.zone {
        let now = device.descriptor.zones.get(zone).map(|zone| &zone.name);
        if now != member.zone_name.as_ref() {
            return MemberState::Changed;
        }
    }
    if device.available {
        MemberState::Resolved
    } else {
        MemberState::Unavailable
    }
}
