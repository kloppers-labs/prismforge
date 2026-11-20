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
