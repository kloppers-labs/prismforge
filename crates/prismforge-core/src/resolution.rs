use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::inventory::DeviceIdentity;
use crate::session::SessionDevice;

/// How a stored device or zone, a group member or a setup output, relates to the devices of today; only `Resolved` is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum MemberState {
    Resolved,
    /// The device is known but not reported now.
    Unavailable,
    Missing,
    /// More than one device could be this one, so none is used.
    Ambiguous,
    /// The zone at this index is not the zone that was stored.
    Changed,
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
