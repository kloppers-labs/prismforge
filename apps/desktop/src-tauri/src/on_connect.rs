use std::sync::Arc;

use prismforge_core::session::SessionHandle;
use prismforge_core::store::{ConnectTrigger, Profiles};
use tokio::sync::Mutex;

use crate::apply_lock::ApplyLock;
use crate::publisher::{Publisher, ViewSink};
use crate::view::{ApplyingProfile, ConnectApply, ConnectOutcome, OnConnect};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    Waiting,
    Fired { applied: Option<String> },
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

    async fn set(&self, next: Phase) {
        let mut phase = self.phase.lock().await;
        if *phase != next {
            *phase = next;
            self.publish_with(&phase).await;
        }
    }

    async fn marked(&self) -> Option<(prismforge_core::profile::ProfileId, String)> {
        let summary = self.profiles.summary(None).await;
        let id = summary.apply_on_connect?;
        let name = summary
            .profiles
            .iter()
            .find(|profile| profile.id == id)
            .map(|profile| profile.name.clone())
            .unwrap_or_default();
        Some((id, name))
    }

    async fn publish_with(&self, phase: &Phase) {
        let marked = self.marked().await;
        let state = match phase {
            Phase::Waiting => marked.map_or(OnConnect::NotArmed, |(id, name)| OnConnect::Armed {
                id,
                name,
            }),
            Phase::Fired { applied } => OnConnect::Fired {
                applied: applied.clone(),
                next: marked.map(|(_, name)| name),
            },
        };
        self.publisher.set_on_connect(state);
    }
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
            let applied = apply_marked(&report, &handle, &lock, &publisher).await;
            report.set(Phase::Fired { applied }).await;
        } else if !trigger.fired() {
            report.set(Phase::Waiting).await;
        }
        if updates.changed().await.is_err() {
            return;
        }
    }
}

/// Returns the name of the profile it applied, or tried to apply.
async fn apply_marked<S: ViewSink>(
    report: &Report<S>,
    handle: &SessionHandle,
    lock: &ApplyLock<S>,
    publisher: &Publisher<S>,
) -> Option<String> {
    let (id, name) = report.marked().await?;
    report
        .set(Phase::Fired {
            applied: Some(name.clone()),
        })
        .await;
    let _guard = lock
        .begin(ApplyingProfile {
            id: id.clone(),
            name: name.clone(),
            automatic: true,
        })
        .await;
    let outcome = match report.profiles.apply_marked(handle).await {
        Ok(Some(report)) => ConnectOutcome::Applied { report },
        Ok(None) => return None,
        Err(error) => ConnectOutcome::Failed { error },
    };
    publisher.set_connect_apply(Some(ConnectApply {
        id,
        name: name.clone(),
        outcome,
    }));
    Some(name)
}
