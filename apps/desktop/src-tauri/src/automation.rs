//! Runs the user's rules on system events; every wait has a limit, and nothing runs without a rule.

mod checks;
pub(crate) mod run;
mod view;
mod wake;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use prismforge_core::DeviceRef;
use prismforge_core::automation::{RuleAction, RuleRun, Trigger};

use tokio::sync::OnceCell;

use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::system_events::{EventSupport, SystemEvent};
use crate::view::CommandError;

pub use checks::OpenRgbCheck;
pub(crate) use checks::OpenRgbChecks;
pub use prismforge_system_events::{SESSION_END_LIMIT, SLEEP_LIMIT};
pub use view::{AutomationView, ProfileChoice, RuleProfile, RuleView};

/// A quit rule runs before the quit's own stop.
pub const QUIT_RULE_LIMIT: Duration = Duration::from_secs(2);
/// The wake rule's own work, after the devices are back.
pub const WAKE_APPLY_LIMIT: Duration = Duration::from_secs(10);
/// How long a rule waits for a profile apply of the user to end before it gives up.
pub const RULE_LOCK_WAIT: Duration = Duration::from_millis(250);
/// The end of each rule's time that is kept for recording its run.
pub const RECORD_LIMIT: Duration = Duration::from_millis(200);
/// Each step of the wake sequence that needs the session slot gives up after this long.
pub const WAKE_STEP_LIMIT: Duration = Duration::from_secs(4);

/// The automation state of the app; the rules themselves live in each store.
pub struct Automation {
    support: EventSupport,
    session_end: OnceCell<Option<RuleRun>>,
    sleeps: AtomicU64,
    wakes: AtomicU64,
    before_sleep: Mutex<Vec<(DeviceRef, String)>>,
}

impl Automation {
    pub fn new(support: EventSupport) -> Self {
        Self {
            support,
            session_end: OnceCell::new(),
            sleeps: AtomicU64::new(0),
            wakes: AtomicU64::new(0),
            before_sleep: Mutex::new(Vec::new()),
        }
    }

    pub fn support(&self) -> &EventSupport {
        &self.support
    }

    pub async fn on_event<S: ViewSink>(&self, shell: &Shell<S>, event: SystemEvent) {
        match event {
            SystemEvent::Sleep => {
                self.at_sleep(shell).await;
            }
            SystemEvent::Wake => {
                self.at_wake(shell).await;
            }
            SystemEvent::SessionEnd => {
                self.at_session_end(shell).await;
            }
        }
    }

    /// Remembers the devices for the wake, then runs the sleep rule within [`SLEEP_LIMIT`].
    pub async fn at_sleep<S: ViewSink>(&self, shell: &Shell<S>) -> Option<RuleRun> {
        self.sleeps.fetch_add(1, Ordering::SeqCst);
        if let Some(snapshot) = shell.view().session {
            *self.lock_before_sleep() = snapshot
                .devices
                .iter()
                .filter(|device| device.available)
                .filter_map(|device| {
                    let reference = device.identity.reference()?.clone();
                    Some((reference, device.descriptor.name.clone()))
                })
                .collect();
        }
        let wakes = self.wakes.load(Ordering::SeqCst);
        let fresh = || self.wakes.load(Ordering::SeqCst) == wakes;
        run::run_rule(shell, Trigger::Sleeps, SLEEP_LIMIT, Vec::new(), fresh).await
    }

    /// Runs the session-end rule once; a second signal waits for the same run and gets its result.
    pub async fn at_session_end<S: ViewSink>(&self, shell: &Shell<S>) -> Option<RuleRun> {
        self.session_end
            .get_or_init(|| {
                run::run_rule(
                    shell,
                    Trigger::SessionEnds,
                    SESSION_END_LIMIT,
                    Vec::new(),
                    || true,
                )
            })
            .await
            .clone()
    }

    pub async fn at_quit<S: ViewSink>(&self, shell: &Shell<S>) -> Option<RuleRun> {
        run::run_rule(shell, Trigger::Quits, QUIT_RULE_LIMIT, Vec::new(), || true).await
    }

    /// Waits for OpenRGB and the devices with capped retries, then runs the wake rule once.
    pub async fn at_wake<S: ViewSink>(&self, shell: &Shell<S>) -> Option<RuleRun> {
        self.wakes.fetch_add(1, Ordering::SeqCst);
        let sleeps = self.sleeps.load(Ordering::SeqCst);
        let before = std::mem::take(&mut *self.lock_before_sleep());
        wake::run_wake(shell, &before, || {
            self.sleeps.load(Ordering::SeqCst) == sleeps
        })
        .await
    }

    pub async fn view<S: ViewSink>(
        &self,
        shell: &Shell<S>,
    ) -> Result<AutomationView, CommandError> {
        view::build(shell, self.support.clone()).await
    }

    /// The user saw these failed runs, on the screen or in their notice.
    pub async fn mark_runs_seen<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        triggers: Vec<Trigger>,
    ) -> Result<AutomationView, CommandError> {
        let (_, profiles) = shell.store().await?;
        profiles.mark_runs_seen(triggers).await?;
        self.view(shell).await
    }

    /// Switches one rule and sets its action; the store refuses a profile it does not have.
    pub async fn set_rule<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        trigger: Trigger,
        enabled: bool,
        action: RuleAction,
    ) -> Result<AutomationView, CommandError> {
        let (_, profiles) = shell.store().await?;
        let changed = profiles.change_rule(trigger, enabled, action).await;
        shell.report_on_connect().await;
        changed?;
        self.view(shell).await
    }

    fn lock_before_sleep(&self) -> std::sync::MutexGuard<'_, Vec<(DeviceRef, String)>> {
        self.before_sleep
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

#[cfg(test)]
mod tests;
