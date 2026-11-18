use std::sync::atomic::{AtomicBool, Ordering};

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

pub async fn stop_then_exit<S: ViewSink>(shell: &Shell<S>, exit: impl FnOnce()) {
    shell.stop().await;
    exit();
}
