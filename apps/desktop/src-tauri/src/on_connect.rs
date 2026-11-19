use std::sync::Arc;

use prismforge_core::session::SessionHandle;
use prismforge_core::store::{ConnectTrigger, Profiles};

use crate::apply_lock::ApplyLock;
use crate::publisher::{Publisher, ViewSink};
use crate::view::{ApplyingProfile, ConnectApply, ConnectOutcome};

/// Runs beside one session: applies the marked profile once each time the session becomes live.
pub(crate) async fn apply_on_connect<S: ViewSink>(
    profiles: Profiles,
    handle: SessionHandle,
    lock: Arc<ApplyLock<S>>,
    publisher: Arc<Publisher<S>>,
) {
    let mut trigger = ConnectTrigger::default();
    let mut updates = handle.subscribe();
    loop {
        let snapshot = updates.borrow_and_update().clone();
        if trigger.due(&snapshot) {
            apply_marked(&profiles, &handle, &lock, &publisher).await;
        }
        if updates.changed().await.is_err() {
            return;
        }
    }
}

async fn apply_marked<S: ViewSink>(
    profiles: &Profiles,
    handle: &SessionHandle,
    lock: &ApplyLock<S>,
    publisher: &Publisher<S>,
) {
    let summary = profiles.summary(None).await;
    let Some(id) = summary.apply_on_connect else {
        return;
    };
    let name = summary
        .profiles
        .iter()
        .find(|profile| profile.id == id)
        .map(|profile| profile.name.clone())
        .unwrap_or_default();
    let _guard = lock
        .begin(ApplyingProfile {
            id: id.clone(),
            name: name.clone(),
            automatic: true,
        })
        .await;
    let outcome = match profiles.apply_marked(handle).await {
        Ok(Some(report)) => ConnectOutcome::Applied { report },
        Ok(None) => return,
        Err(error) => ConnectOutcome::Failed { error },
    };
    publisher.set_connect_apply(Some(ConnectApply { id, name, outcome }));
}
