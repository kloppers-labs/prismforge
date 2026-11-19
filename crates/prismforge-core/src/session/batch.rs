use serde::{Deserialize, Serialize};

use crate::scene::OutputTarget;
use crate::session::{OutputError, RestoreOutcome};

/// Most targets one batch command takes.
pub const MAX_BATCH_TARGETS: usize = 256;

/// What a batch did with each target it was given, in the order given; a partial result is kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct BatchReport {
    pub results: Vec<TargetResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct TargetResult {
    pub target: OutputTarget,
    pub outcome: TargetOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum TargetOutcome {
    Applied,
    Stopped {
        restore: Option<RestoreOutcome>,
    },
    /// Another target of the same batch already holds these LEDs: the whole device, or the same target listed earlier.
    Covered {
        by: OutputTarget,
    },
    /// Nothing was sent to this target; `error` says why.
    Skipped {
        error: OutputError,
    },
}

/// For each target, the target of the batch that covers it; a whole device wins over its zones wherever it is listed.
pub(crate) fn coverage(targets: &[OutputTarget]) -> Vec<Option<OutputTarget>> {
    targets
        .iter()
        .enumerate()
        .map(|(index, target)| {
            let whole = OutputTarget {
                device: target.device.clone(),
                zone: None,
            };
            if target.zone.is_some() && targets.contains(&whole) {
                return Some(whole);
            }
            targets[..index].contains(target).then(|| target.clone())
        })
        .collect()
}
