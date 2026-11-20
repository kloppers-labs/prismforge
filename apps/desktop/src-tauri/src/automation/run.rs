use std::time::{Duration, SystemTime};

use prismforge_core::ConnectionState;
use prismforge_core::automation::{
    RuleAction, RuleRun, RunFailure, RunOutcome, Trigger, plan_lights_off,
};
use prismforge_core::profile::{ApplyError, ProfileId};
use prismforge_core::session::{OutputError, SessionHandle};
use prismforge_core::store::{ApplyOutcome, Profiles};
use tokio::time::{Instant, timeout, timeout_at};

use crate::apply_lock::ApplyLock;
use crate::automation::{RECORD_LIMIT, RULE_LOCK_WAIT, now_ms};
use crate::publisher::{Publisher, ViewSink};
use crate::shell::Shell;
use crate::stores::StoreKind;
use crate::view::ApplyingProfile;

enum Act {
    Apply(ProfileId),
    LightsOff,
}

/// Runs the rule for `trigger` once if it is on, records and shows the run; everything, the session slot included, ends by `limit`.
/// `fresh` turns false once the event is over (a wake after a sleep); nothing is written after that.
pub(crate) async fn run_rule<S: ViewSink>(
    shell: &Shell<S>,
    trigger: Trigger,
    limit: Duration,
    missing: Vec<String>,
    fresh: impl Fn() -> bool,
) -> Option<RuleRun> {
    let deadline = Instant::now() + limit;
    let work_deadline = deadline - RECORD_LIMIT.min(limit / 2);
    let not_after = SystemTime::now() + limit;
    let timed_out = || {
        failed(RunFailure::TimedOut {
            limit_ms: limit.as_millis() as u64,
        })
    };
    let profiles = match timeout_at(work_deadline, shell.stores.profiles(store_kind(shell))).await {
        Ok(Ok(profiles)) => profiles,
        Ok(Err(reason)) => {
            let outcome = failed(RunFailure::StoreUnavailable {
                detail: format!("{reason:?}"),
            });
            return Some(publish(&shell.publisher, run_of(trigger, outcome)));
        }
        Err(_) => return Some(publish(&shell.publisher, run_of(trigger, timed_out()))),
    };
    let act = match timeout_at(work_deadline, profiles.rules()).await {
        Err(_) => None,
        Ok(rules) => match rules.rule(trigger).due()? {
            RuleAction::ApplyProfile { profile } => Some(Act::Apply(profile.clone())),
            RuleAction::LightsOff => Some(Act::LightsOff),
            RuleAction::Nothing | RuleAction::Newer => return None,
        },
    };
    let outcome = match act {
        None => timed_out(),
        Some(act) => {
            let work = act_now(shell, &profiles, trigger, act, missing, &fresh, not_after);
            timeout_at(work_deadline, work)
                .await
                .unwrap_or_else(|_| timed_out())
        }
    };
    let run = run_of(trigger, outcome);
    let _ = timeout_at(deadline, record(&profiles, run.clone())).await;
    Some(publish(&shell.publisher, run))
}

/// The store of the session the window shows; read without the session slot.
fn store_kind<S: ViewSink>(shell: &Shell<S>) -> StoreKind {
    shell.view().session.map_or(StoreKind::Real, |session| {
        StoreKind::of(session.backend.origin)
    })
}

pub(crate) fn run_of(trigger: Trigger, outcome: RunOutcome) -> RuleRun {
    RuleRun {
        trigger,
        at_ms: now_ms(),
        outcome,
        seen: false,
    }
}

pub(crate) fn publish<S: ViewSink>(publisher: &Publisher<S>, run: RuleRun) -> RuleRun {
    publisher.set_rule_run(run.clone());
    run
}

/// Records the run; a write that fails becomes a notice of the store, so the screen shows it.
pub(crate) async fn record(profiles: &Profiles, run: RuleRun) {
    if let Err(error) = profiles.record_run(run).await {
        profiles.note_write_failure(error).await;
    }
}

pub(crate) fn failed(failure: RunFailure) -> RunOutcome {
    RunOutcome::Failed { failure }
}

/// A profile applied as PrismForge ends counts as kept only when its last frame was kept.
pub(crate) fn kept(outcome: RunOutcome, keep: Result<(), OutputError>) -> RunOutcome {
    match (outcome, keep) {
        (
            RunOutcome::Applied {
                profile,
                name,
                report,
                missing,
                ..
            },
            Err(error),
        ) => RunOutcome::Applied {
            profile,
            name,
            report,
            missing,
            not_kept: Some(error),
        },
        (outcome, _) => outcome,
    }
}

async fn act_now<S: ViewSink>(
    shell: &Shell<S>,
    profiles: &Profiles,
    trigger: Trigger,
    act: Act,
    missing: Vec<String>,
    fresh: &impl Fn() -> bool,
    not_after: SystemTime,
) -> RunOutcome {
    let (handle, _) = shell.current().await;
    let Some(handle) = handle.filter(|handle| is_live(&handle.snapshot().connection)) else {
        return failed(RunFailure::NotConnected);
    };
    let late = || !fresh() || SystemTime::now() > not_after;
    match act {
        Act::Apply(profile) => {
            let applied = apply(shell, &handle, profiles, profile, missing, &late).await;
            if matches!(trigger, Trigger::Quits | Trigger::SessionEnds) {
                return kept(applied, handle.keep_lights().await);
            }
            applied
        }
        Act::LightsOff => lights_off(&shell.lock, &handle, Some(not_after), &late).await,
    }
}

async fn apply<S: ViewSink>(
    shell: &Shell<S>,
    handle: &SessionHandle,
    profiles: &Profiles,
    profile: ProfileId,
    missing: Vec<String>,
    late: &impl Fn() -> bool,
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
    if late() {
        return failed(RunFailure::TooLate);
    }
    match profiles.apply(handle, &profile).await {
        Ok(ApplyOutcome::Applied(report)) => RunOutcome::Applied {
            profile,
            name,
            report,
            missing,
            not_kept: None,
        },
        Ok(ApplyOutcome::WaitingForConnection) => failed(RunFailure::NotConnected),
        Err(error) => failed(RunFailure::of_apply(&profile, error)),
    }
}

/// Switches the lights off under the apply slot; other output commands are refused with that reason meanwhile.
pub(crate) async fn lights_off<S: ViewSink>(
    lock: &ApplyLock<S>,
    handle: &SessionHandle,
    not_after: Option<SystemTime>,
    late: &impl Fn() -> bool,
) -> RunOutcome {
    let Ok(_guard) = timeout(RULE_LOCK_WAIT, lock.begin_rule()).await else {
        return failed(RunFailure::Busy);
    };
    if late() {
        return failed(RunFailure::TooLate);
    }
    let plan = plan_lights_off(&handle.snapshot().devices);
    let switched = match not_after {
        Some(not_after) => handle.lights_off_until(plan, not_after).await,
        None => handle.lights_off(plan).await,
    };
    match switched {
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
