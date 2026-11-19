#![allow(dead_code)]

pub mod logged;
pub mod temp;

use std::time::Duration;

use prismforge_core::ConnectionState;
use prismforge_core::session::SessionSnapshot;
use prismforge_desktop::{ShellView, Stores, ViewSink};
use tokio::sync::watch;
use tokio::time::Instant;

use temp::TempDir;

const PATIENCE: Duration = Duration::from_secs(3600);

#[derive(Clone)]
pub struct Recorder {
    views: watch::Sender<Vec<(Instant, ShellView)>>,
}

impl Default for Recorder {
    fn default() -> Self {
        Self {
            views: watch::channel(Vec::new()).0,
        }
    }
}

impl ViewSink for Recorder {
    fn deliver(&self, view: &ShellView) {
        self.views
            .send_modify(|views| views.push((Instant::now(), view.clone())));
    }
}

impl Recorder {
    pub fn views(&self) -> Vec<ShellView> {
        self.timed().into_iter().map(|(_, view)| view).collect()
    }

    pub fn timed(&self) -> Vec<(Instant, ShellView)> {
        self.views.borrow().clone()
    }

    pub async fn wait_for(&self, mut condition: impl FnMut(&SessionSnapshot) -> bool) -> ShellView {
        let mut views = self.views.subscribe();
        let reached = views.wait_for(|views| {
            views
                .last()
                .and_then(|(_, view)| view.session.as_ref())
                .is_some_and(&mut condition)
        });
        tokio::time::timeout(PATIENCE, reached)
            .await
            .expect("condition never reached")
            .expect("recorder dropped")
            .last()
            .map(|(_, view)| view.clone())
            .expect("a view")
    }
}

impl Recorder {
    pub async fn wait_view(&self, mut condition: impl FnMut(&ShellView) -> bool) -> ShellView {
        let mut views = self.views.subscribe();
        let reached = views.wait_for(|views| views.last().is_some_and(|(_, view)| condition(view)));
        tokio::time::timeout(PATIENCE, reached)
            .await
            .expect("condition never reached")
            .expect("recorder dropped")
            .last()
            .map(|(_, view)| view.clone())
            .expect("a view")
    }
}

pub fn is_connected(snapshot: &SessionSnapshot) -> bool {
    snapshot.connection == ConnectionState::Connected
}

pub fn is_retrying(snapshot: &SessionSnapshot) -> bool {
    matches!(snapshot.connection, ConnectionState::Retrying { .. })
}

/// A store folder that lives as long as the returned [`TempDir`].
pub fn stores(label: &str) -> (Stores, TempDir) {
    let dir = TempDir::new(label);
    (Stores::new(dir.path().to_path_buf()), dir)
}
