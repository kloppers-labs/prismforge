use serde::{Deserialize, Serialize};

use crate::automation::{LightsOffReport, Trigger};
use crate::profile::{ApplyError, ApplyReport, Extra, ProfileId};
use crate::session::OutputError;
use crate::store::StoreError;

/// One automatic change, kept until the same rule runs again; it survives a restart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct RuleRun {
    pub trigger: Trigger,
    /// Milliseconds since the Unix epoch.
    pub at_ms: u64,
    pub outcome: RunOutcome,
    /// False until the user saw a failed run on the screen or in its notice.
    #[serde(default)]
    pub seen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RunOutcome {
    Applied {
        profile: ProfileId,
        name: String,
        report: ApplyReport,
        /// Devices that were there before a sleep and did not come back in time.
        missing: Vec<String>,
        /// Why the profile's last frame could not be kept as PrismForge ended; the stop may put back the earlier lighting.
        #[serde(default)]
        not_kept: Option<OutputError>,
    },
    LightsOff {
        report: LightsOffReport,
    },
    Failed {
        failure: RunFailure,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RunFailure {
    /// The rule names a profile the store no longer has; PrismForge never picks another one.
    ProfileMissing {
        profile: ProfileId,
    },
    ProfileNeedsNewer {
        profile: ProfileId,
    },
    /// PrismForge was applying a profile for the user and the rule did not wait longer.
    Busy,
    NotConnected,
    /// The event was over before the rule could write, for example a wake came first; nothing was written.
    TooLate,
    /// The work did not finish within the time the event allows.
    TimedOut {
        limit_ms: u64,
    },
    /// OpenRGB did not answer after a wake.
    OpenRgbNotBack {
        waited_ms: u64,
    },
    Output {
        error: OutputError,
    },
    Store {
        error: StoreError,
    },
    /// The store could not be opened, so PrismForge could not read its rules and none ran.
    StoreUnavailable {
        detail: String,
    },
}

impl RunFailure {
    pub fn of_apply(profile: &ProfileId, error: ApplyError) -> Self {
        match error {
            ApplyError::NotConnected => Self::NotConnected,
            ApplyError::Store(StoreError::UnknownProfile { .. }) => Self::ProfileMissing {
                profile: profile.clone(),
            },
            ApplyError::Store(StoreError::NewerFile { .. }) => Self::ProfileNeedsNewer {
                profile: profile.clone(),
            },
            ApplyError::Store(error) => Self::Store { error },
        }
    }
}

/// The last run of each rule, in the order the rules first ran.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct RunLog {
    #[serde(default)]
    pub runs: Vec<RuleRun>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl RunLog {
    pub fn mark_seen(&mut self, triggers: &[Trigger]) -> bool {
        let mut changed = false;
        for run in &mut self.runs {
            if triggers.contains(&run.trigger) && !run.seen {
                run.seen = true;
                changed = true;
            }
        }
        changed
    }

    pub fn record(&mut self, run: RuleRun) {
        match self
            .runs
            .iter_mut()
            .find(|kept| kept.trigger == run.trigger)
        {
            Some(kept) => *kept = run,
            None => self.runs.push(run),
        }
    }
}
