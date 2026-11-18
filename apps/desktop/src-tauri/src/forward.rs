use std::sync::Arc;
use std::time::Duration;

use prismforge_core::session::SessionSnapshot;
use tokio::sync::watch;

use crate::publisher::{Publisher, ViewSink};

/// Publishes the latest snapshot after each change, at most once per `spacing`.
pub async fn forward<S: ViewSink>(
    mut updates: watch::Receiver<SessionSnapshot>,
    publisher: Arc<Publisher<S>>,
    spacing: Duration,
) {
    while updates.changed().await.is_ok() {
        let snapshot = updates.borrow_and_update().clone();
        publisher.publish(Some(snapshot));
        tokio::time::sleep(spacing).await;
    }
}
