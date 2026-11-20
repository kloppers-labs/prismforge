use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::automation::{Automation, SESSION_END_LIMIT};
use crate::publisher::ViewSink;
use crate::shell::Shell;

/// Holds the first exit request until the shell has stopped; later requests pass.
#[derive(Default)]
pub struct ExitGuard {
    stopping: AtomicBool,
    stopped: AtomicBool,
}

impl ExitGuard {
    pub fn hold(&self) -> bool {
        !self.stopping.swap(true, Ordering::SeqCst)
    }
}

/// Below the 5 seconds after which Windows ends a hidden app at session end or lists it as blocking.
pub const LAST_STOP_LIMIT: Duration = Duration::from_secs(4);

/// A user quit waits this long for the stop, then ends the process anyway.
pub const QUIT_STOP_LIMIT: Duration = Duration::from_secs(10);

pub async fn stop_then_exit<S: ViewSink>(shell: &Shell<S>, exit: impl FnOnce()) {
    stop_within(shell, QUIT_STOP_LIMIT).await;
    exit();
}

/// The last stop when the process ends without an exit request; true when it finished within `limit`.
pub async fn stop_within<S: ViewSink>(shell: &Shell<S>, limit: Duration) -> bool {
    tokio::time::timeout(limit, shell.stop()).await.is_ok()
}

/// A user quit: the bounded stop, then the exit, so the stop at `Exit` has nothing left to wait for.
pub async fn quit_then_exit<S: ViewSink>(shell: &Shell<S>, guard: &ExitGuard, exit: impl FnOnce()) {
    stop_then_exit(shell, || {
        guard.stopped.store(true, Ordering::SeqCst);
        exit();
    })
    .await;
}

/// The stop at `Exit`; true when it finished within `limit` or a user quit already ran its stop.
pub async fn stop_at_exit<S: ViewSink>(
    shell: &Shell<S>,
    guard: &ExitGuard,
    limit: Duration,
) -> bool {
    guard.stopped.load(Ordering::SeqCst) || stop_within(shell, limit).await
}

/// A user quit with its rule first: the rule waits at most [`crate::automation::QUIT_RULE_LIMIT`].
pub async fn quit_with_rule<S: ViewSink>(
    shell: &Shell<S>,
    automation: &Automation,
    guard: &ExitGuard,
    exit: impl FnOnce(),
) {
    automation.at_quit(shell).await;
    quit_then_exit(shell, guard, exit).await;
}

/// The final `Exit` without a user quit: the session-end rule, then the stop, together within [`LAST_STOP_LIMIT`].
pub async fn end_of_session<S: ViewSink>(
    shell: &Shell<S>,
    automation: &Automation,
    guard: &ExitGuard,
) -> bool {
    if guard.stopped.load(Ordering::SeqCst) {
        return true;
    }
    let started = tokio::time::Instant::now();
    if automation.support().delivers_events() {
        let _ = tokio::time::timeout(SESSION_END_LIMIT, automation.at_session_end(shell)).await;
    }
    stop_within(shell, LAST_STOP_LIMIT.saturating_sub(started.elapsed())).await
}
