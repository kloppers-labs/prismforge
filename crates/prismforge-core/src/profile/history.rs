use crate::device::DeviceRef;
use crate::ownership::OutputOwner;
use crate::profile::apply::{ApplyReport, ItemOutcome, ProfileItem, SkipReason};
use crate::scene::{Assignment, OutputTarget};
use crate::session::SessionSnapshot;

/// The session-side target of one profile setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    Effect(OutputTarget),
    Mode(DeviceRef),
    External(DeviceRef),
}

/// What a profile put into one session, so an update can tell the user's later changes from skipped settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionHistory {
    live: Vec<Target>,
    skipped: Vec<(Target, SkipReason)>,
    owners: Vec<(DeviceRef, OutputOwner)>,
    scene: Vec<Assignment>,
}

impl SessionHistory {
    /// `snapshot` is taken after the apply, so the apply's own changes count as the starting state.
    pub fn after_apply(report: &ApplyReport, snapshot: &SessionSnapshot) -> Self {
        let mut history = Self::now(snapshot);
        for item in &report.items {
            let Some(device) = &item.target else {
                continue;
            };
            let target = Target::of(&item.item, device);
            match &item.outcome {
                ItemOutcome::Applied => history.live.push(target),
                ItemOutcome::Skipped { reason } => history.skipped.push((target, reason.clone())),
            }
        }
        history
    }

    /// After a capture the session state is the profile; skip reasons of settings still kept stay.
    pub(crate) fn after_capture(&self, snapshot: &SessionSnapshot, live: Vec<Target>) -> Self {
        let mut history = Self::now(snapshot);
        history.skipped = self
            .skipped
            .iter()
            .filter(|(target, _)| !live.contains(target))
            .cloned()
            .collect();
        history.live = live;
        history
    }

    pub(crate) fn was_live(&self, target: &Target) -> bool {
        self.live.contains(target)
    }

    pub(crate) fn skip_reason(&self, target: &Target) -> Option<SkipReason> {
        self.skipped
            .iter()
            .find(|(skipped, _)| skipped == target)
            .map(|(_, reason)| reason.clone())
    }

    /// True when nothing on `device` changed since the apply or the last capture.
    pub(crate) fn untouched(&self, device: &DeviceRef, snapshot: &SessionSnapshot) -> bool {
        let now = Self::now(snapshot);
        self.state_of(device) == now.state_of(device)
    }

    fn now(snapshot: &SessionSnapshot) -> Self {
        Self {
            live: Vec::new(),
            skipped: Vec::new(),
            owners: snapshot
                .output
                .devices
                .iter()
                .map(|output| (output.device.clone(), output.owner.clone()))
                .collect(),
            scene: snapshot.output.scene.assignments.clone(),
        }
    }

    fn state_of(&self, device: &DeviceRef) -> (Option<&OutputOwner>, Vec<&Assignment>) {
        let owner = self
            .owners
            .iter()
            .find(|(owner_of, _)| owner_of == device)
            .map(|(_, owner)| owner);
        let effects = self
            .scene
            .iter()
            .filter(|assignment| &assignment.target.device == device)
            .collect();
        (owner, effects)
    }
}

impl Target {
    pub(crate) fn of(item: &ProfileItem, device: &DeviceRef) -> Self {
        match item {
            ProfileItem::Effect { zone, .. } => Self::Effect(OutputTarget {
                device: device.clone(),
                zone: *zone,
            }),
            ProfileItem::HardwareMode { .. } => Self::Mode(device.clone()),
            ProfileItem::External { .. } => Self::External(device.clone()),
        }
    }

    pub(crate) fn device(&self) -> &DeviceRef {
        match self {
            Self::Effect(target) => &target.device,
            Self::Mode(device) | Self::External(device) => device,
        }
    }

    /// Effects overlap by zone; a mode or an external statement covers the whole device.
    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Effect(left), Self::Effect(right)) => left.overlaps(right),
            _ => self.device() == other.device(),
        }
    }
}
