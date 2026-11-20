use std::sync::Mutex;

use prismforge_core::automation::RuleRun;
use prismforge_core::session::{LiveOutput, SessionSnapshot};
use tokio::sync::watch;

use crate::view::{ApplyingProfile, ConnectApply, OnConnect, ShellView};

pub trait ViewSink: Send + Sync + 'static {
    fn deliver(&self, view: &ShellView);

    fn deliver_live(&self, _live: &LiveOutput) {}
}

/// Numbers each view and delivers views in the order of their numbers.
pub struct Publisher<S> {
    sink: S,
    current: Mutex<ShellView>,
    changes: watch::Sender<u64>,
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
                on_connect: OnConnect::NotApplicable,
                rule_run: None,
            }),
            changes: watch::channel(0).0,
        }
    }

    /// Live output goes straight to the window; it is not part of the numbered view.
    pub fn deliver_live(&self, live: &LiveOutput) {
        self.sink.deliver_live(live);
    }

    pub fn current(&self) -> ShellView {
        self.lock().clone()
    }

    /// Wakes in-process subscribers after every view and after each [`Self::touch`].
    pub fn changes(&self) -> watch::Receiver<u64> {
        self.changes.subscribe()
    }

    /// For a change outside the view that a subscriber reads beside it, such as the active profile.
    pub fn touch(&self) {
        self.changes.send_modify(|count| *count += 1);
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

    pub fn set_on_connect(&self, on_connect: OnConnect) -> ShellView {
        self.change(|view| view.on_connect = on_connect)
    }

    pub fn set_rule_run(&self, run: RuleRun) -> ShellView {
        self.change(|view| view.rule_run = Some(run))
    }

    fn change(&self, edit: impl FnOnce(&mut ShellView)) -> ShellView {
        let mut current = self.lock();
        edit(&mut current);
        current.revision += 1;
        self.sink.deliver(&current);
        self.touch();
        current.clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ShellView> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
