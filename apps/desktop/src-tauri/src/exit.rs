use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
