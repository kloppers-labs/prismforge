#![allow(dead_code)]

pub mod events;
pub mod lifecycle;
pub mod logged;
pub mod setups;
pub mod temp;

use std::time::Duration;

use prismforge_core::ConnectionState;
use prismforge_core::automation::{RuleAction, Trigger};
use prismforge_core::profile::ProfileId;
use prismforge_core::session::{LiveOutput, SessionSnapshot};
use prismforge_desktop::{
    Automation, AutomationView, CommandError, EventSupport, Shell, ShellView, Stores, ViewSink,
};
use tokio::sync::watch;
use tokio::time::Instant;

use temp::TempDir;

const PATIENCE: Duration = Duration::from_secs(3600);

#[derive(Clone)]
pub struct Recorder {
    views: watch::Sender<Vec<(Instant, ShellView)>>,
    live: watch::Sender<Vec<LiveOutput>>,
}

impl Default for Recorder {
    fn default() -> Self {
        Self {
            views: watch::channel(Vec::new()).0,
            live: watch::channel(Vec::new()).0,
        }
    }
}

impl ViewSink for Recorder {
    fn deliver(&self, view: &ShellView) {
        self.views
            .send_modify(|views| views.push((Instant::now(), view.clone())));
    }

    fn deliver_live(&self, live: &LiveOutput) {
        self.live.send_modify(|lives| lives.push(live.clone()));
    }
}

impl Recorder {
    pub fn live(&self) -> Vec<LiveOutput> {
        self.live.borrow().clone()
    }

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

/// The rule "When PrismForge connects": switched on with `id`, or off without one.
pub fn connects_action(id: Option<ProfileId>) -> (bool, RuleAction) {
    match id {
        Some(profile) => (true, RuleAction::ApplyProfile { profile }),
        None => (false, RuleAction::Nothing),
    }
}

/// Sets the connects rule the way the Automation screen does, as the profile mark did before rules.
#[allow(async_fn_in_trait)]
pub trait ConnectsRule {
    async fn set_apply_on_connect(
        &self,
        id: Option<ProfileId>,
    ) -> Result<AutomationView, CommandError>;
}

impl<S: ViewSink> ConnectsRule for Shell<S> {
    async fn set_apply_on_connect(
        &self,
        id: Option<ProfileId>,
    ) -> Result<AutomationView, CommandError> {
        let (enabled, action) = connects_action(id);
        Automation::new(EventSupport::NotOnThisSystem)
            .set_rule(self, Trigger::Connects, enabled, action)
            .await
    }
}
