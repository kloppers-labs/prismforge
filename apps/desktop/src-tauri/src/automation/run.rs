use std::time::Duration;

use prismforge_core::ConnectionState;
use prismforge_core::automation::{
    RuleAction, RuleRun, RunFailure, RunOutcome, Trigger, plan_lights_off,
};
use prismforge_core::profile::{ApplyError, ProfileId};
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{ApplyOutcome, Profiles};
use tokio::time::timeout;

use crate::automation::{RULE_LOCK_WAIT, now_ms};
use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::view::ApplyingProfile;

enum Act {
    Apply(ProfileId),
    LightsOff,
}

/// Runs the rule for `trigger` once if it is on, records the run and shows it; `None` when no rule is due.
pub(crate) async fn run_rule<S: ViewSink>(
    shell: &Shell<S>,
    trigger: Trigger,
    limit: Duration,
    missing: Vec<String>,
) -> Option<RuleRun> {
    let (handle, kind) = shell.current().await;
    let profiles = shell.stores.profiles(kind).await.ok()?;
    let act = match profiles.rules().await.rule(trigger).due(trigger)? {
        RuleAction::ApplyProfile { profile } => Act::Apply(profile.clone()),
        RuleAction::LightsOff => Act::LightsOff,
        RuleAction::Nothing | RuleAction::Newer => return None,
    };
    let work = act_now(shell, handle, &profiles, trigger, act, missing);
    let outcome = timeout(limit, work).await.unwrap_or(RunOutcome::Failed {
        failure: RunFailure::TimedOut {
            limit_ms: limit.as_millis() as u64,
        },
    });
    let run = RuleRun {
        trigger,
        at_ms: now_ms(),
        outcome,
    };
    record(shell, &profiles, run.clone()).await;
    Some(run)
}

pub(crate) async fn record<S: ViewSink>(shell: &Shell<S>, profiles: &Profiles, run: RuleRun) {
    let _ = profiles.record_run(run.clone()).await;
    shell.publisher.set_rule_run(run);
}

pub(crate) fn failed(failure: RunFailure) -> RunOutcome {
    RunOutcome::Failed { failure }
}

async fn act_now<S: ViewSink>(
    shell: &Shell<S>,
    handle: Option<SessionHandle>,
    profiles: &Profiles,
    trigger: Trigger,
    act: Act,
    missing: Vec<String>,
) -> RunOutcome {
    let Some(handle) = handle.filter(|handle| is_live(&handle.snapshot().connection)) else {
        return failed(RunFailure::NotConnected);
    };
    match act {
        Act::Apply(profile) => apply(shell, &handle, profiles, trigger, profile, missing).await,
        Act::LightsOff => lights_off(shell, &handle).await,
    }
}

async fn apply<S: ViewSink>(
    shell: &Shell<S>,
    handle: &SessionHandle,
    profiles: &Profiles,
    trigger: Trigger,
    profile: ProfileId,
    missing: Vec<String>,
) -> RunOutcome {
    let name = match profiles.profile(&profile).await {
        Ok(stored) => stored.name,
        Err(error) => return failed(RunFailure::of_apply(&profile, ApplyError::Store(error))),
    };
    let applying = ApplyingProfile {
        id: profile.clone(),
        name: name.clone(),
        automatic: true,
    };
    let Ok(_guard) = timeout(RULE_LOCK_WAIT, shell.lock.begin(Some(applying))).await else {
        return failed(RunFailure::Busy);
    };
    match profiles.apply(handle, &profile).await {
        Ok(ApplyOutcome::Applied(report)) => {
            if matches!(trigger, Trigger::Quits | Trigger::SessionEnds) {
                let _ = handle.keep_lights().await;
            }
            RunOutcome::Applied {
                profile,
                name,
                report,
                missing,
            }
        }
        Ok(ApplyOutcome::WaitingForConnection) => failed(RunFailure::NotConnected),
        Err(error) => failed(RunFailure::of_apply(&profile, error)),
    }
}

async fn lights_off<S: ViewSink>(shell: &Shell<S>, handle: &SessionHandle) -> RunOutcome {
    let Ok(_guard) = timeout(RULE_LOCK_WAIT, shell.lock.begin(None)).await else {
        return failed(RunFailure::Busy);
    };
    let plan = plan_lights_off(&handle.snapshot().devices);
    match handle.lights_off(plan).await {
        Ok(report) => RunOutcome::LightsOff { report },
        Err(error) => failed(RunFailure::Output { error }),
    }
}

pub(crate) fn is_live(connection: &ConnectionState) -> bool {
    matches!(
        connection,
        ConnectionState::Connected | ConnectionState::Degraded { .. }
    )
}
