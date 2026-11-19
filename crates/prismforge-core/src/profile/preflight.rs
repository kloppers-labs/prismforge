use std::collections::BTreeMap;

use crate::device::DeviceRef;
use crate::ownership::OutputOwner;
use crate::profile::apply::{ProfileItem, SkipReason, mode_now, target_of};
use crate::profile::model::Profile;
use crate::profile::resolve::resolve;
use crate::scene::{ApplyMode, OutputTarget};
use crate::session::{DeviceOutput, SessionDevice};
use crate::setup::{Setup, ViewId, plan_across};

/// One stored setting with the reason apply would skip it today, in apply order.
pub(crate) struct Checked {
    pub device: DeviceRef,
    pub item: ProfileItem,
    pub skip: Option<SkipReason>,
}

/// What `apply` would do with each setting against these devices and owners, without sending anything.
pub(crate) fn preflight(
    profile: &Profile,
    devices: &[SessionDevice],
    owners: &[DeviceOutput],
    setup: Option<&Setup>,
) -> Vec<Checked> {
    let resolutions = resolve(profile, devices);
    let external = |device: &DeviceRef| {
        owners
            .iter()
            .find(|output| &output.device == device)
            .and_then(|output| match &output.owner {
                OutputOwner::External { label } => Some(SkipReason::DeclaredExternal {
                    label: label.clone(),
                }),
                _ => None,
            })
    };
    let mut checked = Vec::new();
    for statement in &profile.external {
        checked.push(Checked {
            device: statement.device.clone(),
            item: ProfileItem::External {
                label: statement.label.clone(),
            },
            skip: target_of(&resolutions, &statement.device).err(),
        });
    }
    for mode in &profile.modes {
        let skip = match target_of(&resolutions, &mode.device) {
            Err(reason) => Some(reason),
            Ok(device) => match mode_now(devices, &device, mode.index) {
                Some(found) if found == mode.name => external(&device),
                found => Some(SkipReason::ModeListChanged { found }),
            },
        };
        checked.push(Checked {
            device: mode.device.clone(),
            item: ProfileItem::HardwareMode {
                index: mode.index,
                name: mode.name.clone(),
            },
            skip,
        });
    }
    let mut placed: BTreeMap<ViewId, Vec<OutputTarget>> = BTreeMap::new();
    for assignment in &profile.assignments {
        let skip = match (
            target_of(&resolutions, &assignment.device),
            assignment.effect.instance(),
        ) {
            (Err(reason), _) => Some(reason),
            (Ok(_), Err(problem)) => Some(SkipReason::NotAvailable { problem }),
            (Ok(device), Ok(_)) => {
                let target = OutputTarget {
                    device: device.clone(),
                    zone: assignment.zone,
                };
                across(&assignment.mode, &target, setup, devices, &mut placed)
                    .or_else(|| external(&device))
                    .or_else(|| {
                        let direct = devices
                            .iter()
                            .find(|candidate| candidate.identity.reference() == Some(&device))
                            .is_some_and(|candidate| {
                                candidate.descriptor.capabilities.direct_mode.is_some()
                            });
                        (!direct).then_some(SkipReason::NoDirectMode)
                    })
            }
        };
        checked.push(Checked {
            device: assignment.device.clone(),
            item: ProfileItem::Effect {
                zone: assignment.zone,
                effect: assignment.effect.effect.clone(),
            },
            skip,
        });
    }
    checked
}

/// Why an effect across the setup cannot run on `target`, checked before the session's own reasons as apply does.
fn across(
    mode: &ApplyMode,
    target: &OutputTarget,
    setup: Option<&Setup>,
    devices: &[SessionDevice],
    placed: &mut BTreeMap<ViewId, Vec<OutputTarget>>,
) -> Option<SkipReason> {
    let ApplyMode::AcrossSetup { view, flow } = mode else {
        return None;
    };
    let Some(setup) = setup else {
        return Some(SkipReason::NewerSetup);
    };
    let targets = placed.entry(view.clone()).or_insert_with(|| {
        plan_across(setup, view, *flow, devices).map_or_else(
            |_| Vec::new(),
            |plan| {
                plan.targets
                    .into_iter()
                    .map(|placed| placed.target)
                    .collect()
            },
        )
    });
    (!targets.contains(target)).then_some(SkipReason::SetupChanged)
}
