use std::sync::Arc;

use prismforge_core::automation::{RuleAction, RuleRun, RunFailure, RunOutcome, Trigger};
use prismforge_core::profile::{ApplyError, ProfileId};
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{ApplyOutcome, ConnectTrigger, Profiles};
use tokio::sync::Mutex;

use crate::apply_lock::ApplyLock;
use crate::automation::{OpenRgbChecks, now_ms};
use crate::publisher::{Publisher, ViewSink};
use crate::view::{ApplyingProfile, ConnectApply, ConnectOutcome, MarkProblem, Marked, OnConnect};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    Waiting,
    Fired { ran: Option<ConnectApply> },
    Closed,
}

/// The profile a rule applies when the session becomes live, as far as this version can read it.
pub(crate) enum Mark {
    Ready(Trigger, ProfileId, String),
    Unreadable(Trigger, ProfileId, MarkProblem),
}

impl Mark {
    fn trigger(&self) -> Trigger {
        match self {
            Self::Ready(trigger, ..) | Self::Unreadable(trigger, ..) => *trigger,
        }
    }
}

/// Publishes the session's apply-on-connect state from its trigger and the store's current rules.
pub(crate) struct Report<S> {
    profiles: Profiles,
    publisher: Arc<Publisher<S>>,
    launch: bool,
    phase: Mutex<Phase>,
}

impl<S: ViewSink> Report<S> {
    /// `launch` is true for the connect at start, where the starts rule applies in place of the connects rule.
    pub fn new(profiles: Profiles, publisher: Arc<Publisher<S>>, launch: bool) -> Self {
        Self {
            profiles,
            publisher,
            launch,
            phase: Mutex::new(Phase::Waiting),
        }
    }

    pub async fn publish(&self) {
        let phase = self.phase.lock().await;
        self.publish_with(&phase).await;
    }

    /// Ends the report with its session; a publish that started earlier finishes first.
    pub async fn close(&self) {
        *self.phase.lock().await = Phase::Closed;
    }

    async fn set(&self, next: Phase) {
        let mut phase = self.phase.lock().await;
        if *phase != next && *phase != Phase::Closed {
            *phase = next;
            self.publish_with(&phase).await;
        }
    }

    async fn publish_with(&self, phase: &Phase) {
        let state = match phase {
            Phase::Closed => return,
            Phase::Waiting => match marked(&self.profiles, self.launch).await {
                None => OnConnect::NotArmed,
                Some(Mark::Ready(_, id, name)) => OnConnect::Armed { id, name },
                Some(Mark::Unreadable(_, id, problem)) => OnConnect::Unreadable { id, problem },
            },
            Phase::Fired { ran } => OnConnect::Fired {
                ran: ran.clone(),
                next: marked(&self.profiles, false).await.map(|mark| match mark {
                    Mark::Ready(_, _, name) => Marked::Ready { name },
                    Mark::Unreadable(_, _, problem) => Marked::Unreadable { problem },
                }),
            },
        };
        self.publisher.set_on_connect(state);
    }
}

/// The profile the next live session applies: at launch the starts rule if it applies one, else the connects rule.
pub(crate) async fn marked(profiles: &Profiles, launch: bool) -> Option<Mark> {
    let rules = profiles.rules().await;
    let starts = match rules.rule(Trigger::Starts).due(Trigger::Starts) {
        Some(RuleAction::ApplyProfile { profile }) if launch => Some(profile.clone()),
        _ => None,
    };
    let (trigger, id) = match starts {
        Some(id) => (Trigger::Starts, id),
        None => (Trigger::Connects, rules.mark()?.clone()),
    };
    let summary = profiles.summary(None).await;
    if let Some(profile) = summary.profiles.iter().find(|profile| profile.id == id) {
        return Some(Mark::Ready(trigger, id, profile.name.clone()));
    }
    let problem = if summary.unreadable.iter().any(|locked| locked.id == id) {
        MarkProblem::NewerVersion
    } else {
        MarkProblem::Missing
    };
    Some(Mark::Unreadable(trigger, id, problem))
}

/// Runs beside one session: applies the rule's profile once each time the session becomes live.
pub(crate) async fn apply_on_connect<S: ViewSink>(
    report: Arc<Report<S>>,
    handle: SessionHandle,
    lock: Arc<ApplyLock<S>>,
    publisher: Arc<Publisher<S>>,
    checks: Arc<OpenRgbChecks>,
) {
    let mut trigger = ConnectTrigger::default();
    let mut launch = report.launch;
    let mut updates = handle.subscribe();
    loop {
        let snapshot = updates.borrow_and_update().clone();
        if trigger.due(&snapshot) {
            checks.refresh(&handle).await;
            let ran = apply_marked(&report, launch, &handle, &lock, &publisher).await;
            launch = false;
            report.set(Phase::Fired { ran }).await;
        } else if !trigger.fired() {
            report.set(Phase::Waiting).await;
        }
        if updates.changed().await.is_err() {
            return;
        }
    }
}

/// An unreadable profile has no name and no `applying` state; its apply fails before anything is sent.
async fn apply_marked<S: ViewSink>(
    report: &Report<S>,
    launch: bool,
    handle: &SessionHandle,
    lock: &ApplyLock<S>,
    publisher: &Publisher<S>,
) -> Option<ConnectApply> {
    let mark = marked(&report.profiles, launch).await?;
    let rule = mark.trigger();
    let (id, name) = match mark {
        Mark::Ready(_, id, name) => (id, Some(name)),
        Mark::Unreadable(_, id, _) => (id, None),
    };
    let applying = name.clone().map(|name| ApplyingProfile {
        id: id.clone(),
        name,
        automatic: true,
    });
    let _guard = lock.begin(applying).await;
    let applied = match report.profiles.apply(handle, &id).await {
        Ok(ApplyOutcome::Applied(report)) => Ok(report),
        Ok(ApplyOutcome::WaitingForConnection) => Err(ApplyError::NotConnected),
        Err(error) => Err(error),
    };
    let outcome = match &applied {
        Ok(report) => RunOutcome::Applied {
            profile: id.clone(),
            name: name.clone().unwrap_or_default(),
            report: report.clone(),
            missing: Vec::new(),
        },
        Err(error) => RunOutcome::Failed {
            failure: RunFailure::of_apply(&id, error.clone()),
        },
    };
    let run = RuleRun {
        trigger: rule,
        at_ms: now_ms(),
        outcome,
    };
    let _ = report.profiles.record_run(run.clone()).await;
    publisher.set_rule_run(run);
    let outcome = match applied {
        Ok(report) => ConnectOutcome::Applied { report },
        Err(error) => ConnectOutcome::Failed { error },
    };
    let ran = ConnectApply { id, name, outcome };
    publisher.set_connect_apply(Some(ran.clone()));
    Some(ran)
}
