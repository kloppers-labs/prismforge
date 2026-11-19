use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
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
    target_state(
        &member.device,
        member.zone,
        member.zone_name.as_ref(),
        devices,
    )
}

/// How a stored device or zone relates to the devices of today; `zone_name` is the name the zone had when it was stored.
pub(crate) fn target_state(
    device: &DeviceRef,
    zone: Option<usize>,
    zone_name: Option<&String>,
    devices: &[SessionDevice],
) -> MemberState {
    let ambiguous = devices.iter().any(|candidate| {
        matches!(&candidate.identity, DeviceIdentity::Ambiguous { candidates } if candidates.contains(device))
    });
    if ambiguous {
        return MemberState::Ambiguous;
    }
    let Some(found) = devices
        .iter()
        .find(|candidate| candidate.identity.reference() == Some(device))
    else {
        return MemberState::Missing;
    };
    if let Some(zone) = zone {
        let now = found.descriptor.zones.get(zone).map(|zone| &zone.name);
        if now != zone_name {
            return MemberState::Changed;
        }
    }
    if found.available {
        MemberState::Resolved
    } else {
        MemberState::Unavailable
    }
}
