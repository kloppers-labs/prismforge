use prismforge_effects::EffectInstance;

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::scene::ApplyMode;
use crate::session::output::Claim;
use crate::session::runner::Session;
use crate::session::{Intent, Origin, Slot};

/// What one slot holds in the session's ownership state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Held {
    Effect(EffectInstance, ApplyMode),
    Mode(usize),
    External(String),
    FrameRate(u32),
}

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// The ownership state before a user command; `None` for a profile's command, which records nothing.
    pub(super) fn held_for(&self, origin: Origin) -> Option<Vec<(Slot, Held)>> {
        (origin == Origin::User).then(|| self.held())
    }

    /// Records the command's own slots and, under the same sequence number and first, every other slot it changed.
    pub(super) fn record_user(
        &mut self,
        before: Option<Vec<(Slot, Held)>>,
        own: Vec<(Slot, Intent)>,
    ) {
        let Some(before) = before.filter(|_| !own.is_empty()) else {
            return;
        };
        let is_own = |slot: &Slot| own.iter().any(|(mine, _)| mine == slot);
        let after = self.held();
        let mut entries: Vec<(Slot, Intent)> = Vec::new();
        for (slot, was) in &before {
            if is_own(slot) {
                continue;
            }
            match after.iter().find(|(other, _)| other == slot) {
                None => entries.push((slot.clone(), ended(was))),
                Some((_, now)) if now != was => {
                    entries.push((slot.clone(), self.intent_for(slot, now)))
                }
                Some(_) => {}
            }
        }
        for (slot, now) in &after {
            if !is_own(slot) && !before.iter().any(|(other, _)| other == slot) {
                entries.push((slot.clone(), self.intent_for(slot, now)));
            }
        }
        entries.extend(own);
        self.intents.record(entries);
    }

    pub(super) fn mode_name(&self, device: &DeviceRef, mode: usize) -> String {
        self.seen
            .iter()
            .find(|seen| seen.identity.reference() == Some(device))
            .and_then(|seen| seen.descriptor.capabilities.modes.get(mode))
            .map(|descriptor| descriptor.name.clone())
            .unwrap_or_default()
    }

    fn held(&self) -> Vec<(Slot, Held)> {
        let mut held = vec![(Slot::FrameRate, Held::FrameRate(self.output.frame_rate))];
        for (device, claim) in &self.output.claims {
            match claim {
                Claim::Effect(output) => held.extend(output.effects.iter().map(|running| {
                    (
                        Slot::Effect(running.target.clone()),
                        Held::Effect(running.effect.instance().clone(), running.layout.mode()),
                    )
                })),
                Claim::HardwareMode { mode } => {
                    held.push((Slot::Mode(device.clone()), Held::Mode(*mode)))
                }
                Claim::External { label } => {
                    held.push((
                        Slot::External(device.clone()),
                        Held::External(label.clone()),
                    ));
                }
            }
        }
        held
    }

    fn intent_for(&self, slot: &Slot, held: &Held) -> Intent {
        match held {
            Held::Effect(effect, mode) => Intent::Applied {
                effect: effect.clone(),
                mode: mode.clone(),
            },
            Held::Mode(mode) => Intent::ModeSelected {
                index: *mode,
                name: match slot {
                    Slot::Mode(device) => self.mode_name(device, *mode),
                    _ => String::new(),
                },
            },
            Held::External(label) => Intent::Declared {
                label: label.clone(),
            },
            Held::FrameRate(rate) => Intent::FrameRate(*rate),
        }
    }
}

fn ended(held: &Held) -> Intent {
    match held {
        Held::Effect(..) => Intent::Stopped,
        Held::Mode(_) => Intent::ModeEnded,
        Held::External(_) => Intent::HandedBack,
        Held::FrameRate(rate) => Intent::FrameRate(*rate),
    }
}
