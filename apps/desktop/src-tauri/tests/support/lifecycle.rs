use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use prismforge_desktop::{
    ExitControl, Lifecycle, LifecycleSink, LifecycleView, StartEntry, WindowControl,
};
use tokio::sync::watch;

#[derive(Clone)]
pub struct LifecycleRecorder {
    views: watch::Sender<Vec<LifecycleView>>,
}

impl Default for LifecycleRecorder {
    fn default() -> Self {
        Self {
            views: watch::channel(Vec::new()).0,
        }
    }
}

impl LifecycleSink for LifecycleRecorder {
    fn deliver(&self, view: &LifecycleView) {
        self.views.send_modify(|views| views.push(view.clone()));
    }
}

impl LifecycleRecorder {
    pub fn views(&self) -> Vec<LifecycleView> {
        self.views.borrow().clone()
    }

    pub fn last(&self) -> LifecycleView {
        self.views().last().cloned().expect("a lifecycle view")
    }
}

#[derive(Clone, Default)]
pub struct FakeWindow {
    pub visible: Arc<AtomicBool>,
    pub shown: Arc<AtomicUsize>,
    pub hidden: Arc<AtomicUsize>,
}

impl WindowControl for FakeWindow {
    fn show(&self) {
        self.shown.fetch_add(1, Ordering::SeqCst);
        self.visible.store(true, Ordering::SeqCst);
    }

    fn hide(&self) {
        self.hidden.fetch_add(1, Ordering::SeqCst);
        self.visible.store(false, Ordering::SeqCst);
    }
}

impl FakeWindow {
    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Default)]
pub struct FakeExit {
    pub requests: Arc<AtomicUsize>,
}

impl ExitControl for FakeExit {
    fn exit(&self) {
        self.requests.fetch_add(1, Ordering::SeqCst);
    }
}

impl FakeExit {
    pub fn requested(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

/// A start entry in memory; `refuse` makes every change fail, `ignore` makes changes succeed without effect.
#[derive(Clone, Default)]
pub struct FakeEntry {
    pub registered: Arc<Mutex<bool>>,
    pub refuse: Arc<Mutex<Option<String>>>,
    pub ignore: Arc<AtomicBool>,
    pub unreadable: Arc<Mutex<Option<String>>>,
    pub machine: Arc<AtomicBool>,
}

impl StartEntry for FakeEntry {
    fn is_registered(&self) -> Result<bool, String> {
        if let Some(detail) = self.unreadable.lock().expect("entry").clone() {
            return Err(detail);
        }
        Ok(*self.registered.lock().expect("entry"))
    }

    fn register(&self) -> Result<(), String> {
        self.set(true)
    }

    fn unregister(&self) -> Result<(), String> {
        self.set(false)
    }

    fn machine_wide(&self) -> bool {
        self.machine.load(Ordering::SeqCst)
    }
}

impl FakeEntry {
    fn set(&self, on: bool) -> Result<(), String> {
        if let Some(detail) = self.refuse.lock().expect("entry").clone() {
            return Err(detail);
        }
        if !self.ignore.load(Ordering::SeqCst) {
            *self.registered.lock().expect("entry") = on;
        }
        Ok(())
    }

    pub fn is_on(&self) -> bool {
        *self.registered.lock().expect("entry")
    }
}

pub fn lifecycle(entry: &FakeEntry) -> (Lifecycle<LifecycleRecorder>, LifecycleRecorder) {
    let recorder = LifecycleRecorder::default();
    (Lifecycle::new(recorder.clone(), entry.clone()), recorder)
}
