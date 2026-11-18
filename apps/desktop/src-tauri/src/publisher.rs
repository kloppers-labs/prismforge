use std::sync::Mutex;

use prismforge_core::session::SessionSnapshot;

use crate::view::ShellView;

pub trait ViewSink: Send + Sync + 'static {
    fn deliver(&self, view: &ShellView);
}

/// Numbers each view and delivers views in the order of their numbers.
pub struct Publisher<S> {
    sink: S,
    current: Mutex<ShellView>,
}

impl<S: ViewSink> Publisher<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            current: Mutex::new(ShellView {
                revision: 0,
                session: None,
            }),
        }
    }

    pub fn current(&self) -> ShellView {
        self.lock().clone()
    }

    pub fn publish(&self, session: Option<SessionSnapshot>) -> ShellView {
        let mut current = self.lock();
        *current = ShellView {
            revision: current.revision + 1,
            session,
        };
        self.sink.deliver(&current);
        current.clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ShellView> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
