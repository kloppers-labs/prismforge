use prismforge_effects::EffectInstance;

use crate::device::DeviceRef;
use crate::scene::OutputTarget;

/// Who asked for an output command; only the user's commands become intent records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    User,
    Profile,
}

/// What one record is about: an effect target, a device's hardware mode or external owner, or the frame rate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Slot {
    Effect(OutputTarget),
    Mode(DeviceRef),
    External(DeviceRef),
    FrameRate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Intent {
    /// The complete validated instance the session runs.
    Applied(EffectInstance),
    Stopped,
    ModeSelected {
        index: usize,
        name: String,
    },
    ModeEnded,
    Declared {
        label: String,
    },
    HandedBack,
    FrameRate(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntentRecord {
    pub sequence: u64,
    pub slot: Slot,
    pub intent: Intent,
}

/// The records after a sequence number, oldest first, and the newest sequence number they include.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntentLog {
    pub sequence: u64,
    pub records: Vec<IntentRecord>,
}

/// The last user command for each slot; a new record replaces the old one, so it never outgrows the slots used.
#[derive(Debug, Default)]
pub(crate) struct Intents {
    sequence: u64,
    records: Vec<IntentRecord>,
}

impl Intents {
    /// Records everything one user command did under one new sequence number.
    pub fn record(&mut self, entries: Vec<(Slot, Intent)>) {
        self.sequence += 1;
        for (slot, intent) in entries {
            self.records.retain(|record| record.slot != slot);
            self.records.push(IntentRecord {
                sequence: self.sequence,
                slot,
                intent,
            });
        }
    }

    pub fn after(&self, sequence: u64) -> IntentLog {
        IntentLog {
            sequence: self.sequence,
            records: self
                .records
                .iter()
                .filter(|record| record.sequence > sequence)
                .cloned()
                .collect(),
        }
    }
}
