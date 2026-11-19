use std::sync::Mutex;

use prismforge_core::session::SessionSnapshot;

use crate::view::{ApplyingProfile, ConnectApply, ShellView};

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
                applying: None,
                connect_apply: None,
            }),
        }
    }

    pub fn current(&self) -> ShellView {
        self.lock().clone()
    }

    pub fn publish(&self, session: Option<SessionSnapshot>) -> ShellView {
        self.change(|view| view.session = session)
    }

    pub fn set_applying(&self, applying: Option<ApplyingProfile>) -> ShellView {
        self.change(|view| view.applying = applying)
    }

    pub fn set_connect_apply(&self, connect_apply: Option<ConnectApply>) -> ShellView {
        self.change(|view| view.connect_apply = connect_apply)
    }

    fn change(&self, edit: impl FnOnce(&mut ShellView)) -> ShellView {
        let mut current = self.lock();
        edit(&mut current);
        current.revision += 1;
        self.sink.deliver(&current);
        current.clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ShellView> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
