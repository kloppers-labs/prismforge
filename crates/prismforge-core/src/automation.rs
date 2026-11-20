//! Rules of the form "When <event>, <action>", and the parts that carry them out.

mod lights_off;
mod openrgb;
mod rules;
mod runs;
mod wake;

pub use lights_off::{
    DeviceLightsOff, LightsOffOutcome, LightsOffPlan, LightsOffReport, LightsOffSkip,
    LightsOffStep, LightsOffWrite, plan_lights_off,
};
pub use openrgb::{EventProfile, EventProfiles, OpenRgbAct, OpenRgbSetting};
pub use rules::{Rule, RuleAction, RuleProblem, Rules, Trigger};
pub(crate) use runs::RunLog;
pub use runs::{RuleRun, RunFailure, RunOutcome};
pub use wake::{
    WAKE_BACKOFF, WAKE_RECONNECT_LIMIT, WAKE_SETTLE_INTERVAL, WAKE_SETTLE_LIMIT, WakeReady,
    WakeTarget, wait_after_wake,
};
