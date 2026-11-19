use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::publisher::ViewSink;
use crate::shell::Shell;

/// Holds the first exit request until the shell has stopped; later requests pass.
#[derive(Default)]
pub struct ExitGuard {
    stopping: AtomicBool,
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
