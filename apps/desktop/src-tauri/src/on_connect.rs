use std::sync::Arc;

use prismforge_core::profile::ProfileId;
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{ConnectTrigger, Profiles};
use tokio::sync::Mutex;

use crate::apply_lock::ApplyLock;
use crate::publisher::{Publisher, ViewSink};
use crate::view::{ApplyingProfile, ConnectApply, ConnectOutcome, MarkProblem, Marked, OnConnect};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    Waiting,
    Fired { ran: Option<ConnectApply> },
    Closed,
}

/// The profile the store marks now, as far as this version can read it.
pub(crate) enum Mark {
    Ready(ProfileId, String),
    Unreadable(ProfileId, MarkProblem),
}

/// Publishes the session's apply-on-connect state from its trigger and the store's current mark.
pub(crate) struct Report<S> {
    profiles: Profiles,
    publisher: Arc<Publisher<S>>,
    phase: Mutex<Phase>,
}

impl<S: ViewSink> Report<S> {
    pub fn new(profiles: Profiles, publisher: Arc<Publisher<S>>) -> Self {
        Self {
            profiles,
            publisher,
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

    async fn mark(&self) -> Option<Mark> {
        marked(&self.profiles).await
    }

    async fn publish_with(&self, phase: &Phase) {
        let state = match phase {
            Phase::Closed => return,
            Phase::Waiting => match self.mark().await {
                None => OnConnect::NotArmed,
                Some(Mark::Ready(id, name)) => OnConnect::Armed { id, name },
                Some(Mark::Unreadable(id, problem)) => OnConnect::Unreadable { id, problem },
            },
            Phase::Fired { ran } => OnConnect::Fired {
                ran: ran.clone(),
                next: self.mark().await.map(|mark| match mark {
                    Mark::Ready(_, name) => Marked::Ready { name },
                    Mark::Unreadable(_, problem) => Marked::Unreadable { problem },
                }),
            },
        };
        self.publisher.set_on_connect(state);
    }
}

pub(crate) async fn marked(profiles: &Profiles) -> Option<Mark> {
    let summary = profiles.summary(None).await;
    let id = summary.apply_on_connect?;
    if let Some(profile) = summary.profiles.iter().find(|profile| profile.id == id) {
        return Some(Mark::Ready(id, profile.name.clone()));
    }
    let problem = if summary.unreadable.iter().any(|locked| locked.id == id) {
        MarkProblem::NewerVersion
    } else {
        MarkProblem::Missing
    };
    Some(Mark::Unreadable(id, problem))
}

/// Runs beside one session: applies the marked profile once each time the session becomes live.
pub(crate) async fn apply_on_connect<S: ViewSink>(
    report: Arc<Report<S>>,
    handle: SessionHandle,
    lock: Arc<ApplyLock<S>>,
    publisher: Arc<Publisher<S>>,
) {
    let mut trigger = ConnectTrigger::default();
    let mut updates = handle.subscribe();
    loop {
        let snapshot = updates.borrow_and_update().clone();
        if trigger.due(&snapshot) {
            let ran = apply_marked(&report, &handle, &lock, &publisher).await;
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
    handle: &SessionHandle,
    lock: &ApplyLock<S>,
    publisher: &Publisher<S>,
) -> Option<ConnectApply> {
    let (id, name) = match report.mark().await? {
        Mark::Ready(id, name) => (id, Some(name)),
        Mark::Unreadable(id, _) => (id, None),
    };
    let applying = name.clone().map(|name| ApplyingProfile {
        id: id.clone(),
        name,
        automatic: true,
    });
    let _guard = lock.begin(applying).await;
    let outcome = match report.profiles.apply_marked(handle).await {
        Ok(Some(report)) => ConnectOutcome::Applied { report },
        Ok(None) => return None,
        Err(error) => ConnectOutcome::Failed { error },
    };
    let ran = ConnectApply { id, name, outcome };
    publisher.set_connect_apply(Some(ran.clone()));
    Some(ran)
}
