use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::{Mutex, OwnedMutexGuard, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

use crate::publisher::{Publisher, ViewSink};
use crate::view::{ApplyingProfile, CommandError};

/// One profile apply at a time, and no other output command while it runs.
pub(crate) struct ApplyLock<S> {
    slot: Arc<Mutex<()>>,
    gate: Arc<RwLock<()>>,
    running: Arc<AtomicBool>,
    lights_off: Arc<AtomicBool>,
    publisher: Arc<Publisher<S>>,
}

/// Held by an output command until the session answers it, so an apply waits for it.
pub(crate) struct Permit {
    _gate: OwnedRwLockReadGuard<()>,
}

/// Releases the lock and clears `applying` in the view when dropped, on every exit path.
pub(crate) struct ApplyGuard<S: ViewSink> {
    running: Arc<AtomicBool>,
    lights_off: Arc<AtomicBool>,
    publisher: Arc<Publisher<S>>,
    _slot: OwnedMutexGuard<()>,
    gate: Option<OwnedRwLockWriteGuard<()>>,
}

impl<S: ViewSink> ApplyLock<S> {
    pub fn new(publisher: Arc<Publisher<S>>) -> Self {
        Self {
            slot: Arc::new(Mutex::new(())),
            gate: Arc::new(RwLock::new(())),
            running: Arc::new(AtomicBool::new(false)),
            lights_off: Arc::new(AtomicBool::new(false)),
            publisher,
        }
    }

    pub fn permit(&self) -> Result<Permit, CommandError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(self.busy());
        }
        self.gate
            .clone()
            .try_read_owned()
            .map(|gate| Permit { _gate: gate })
            .map_err(|_| self.busy())
    }

    fn busy(&self) -> CommandError {
        if self.lights_off.load(Ordering::SeqCst) {
            CommandError::RuleRunning
        } else {
            CommandError::ProfileApplying
        }
    }

    /// For a rule that switches the lights off: other output commands are refused with that reason.
    pub async fn begin_rule(&self) -> ApplyGuard<S> {
        let slot = self.slot.clone().lock_owned().await;
        self.lights_off.store(true, Ordering::SeqCst);
        self.hold(slot, None).await
    }

    /// For the user: refused at once while another apply runs.
    pub async fn try_begin(
        &self,
        applying: ApplyingProfile,
    ) -> Result<ApplyGuard<S>, CommandError> {
        let slot = self
            .slot
            .clone()
            .try_lock_owned()
            .map_err(|_| CommandError::ProfileApplying)?;
        Ok(self.hold(slot, Some(applying)).await)
    }

    /// For apply on connect: waits for a user's apply to end first; `None` publishes no `applying`.
    pub async fn begin(&self, applying: Option<ApplyingProfile>) -> ApplyGuard<S> {
        let slot = self.slot.clone().lock_owned().await;
        self.hold(slot, applying).await
    }

    async fn hold(
        &self,
        slot: OwnedMutexGuard<()>,
        applying: Option<ApplyingProfile>,
    ) -> ApplyGuard<S> {
        self.running.store(true, Ordering::SeqCst);
        if applying.is_some() {
            self.publisher.set_applying(applying);
        }
        let mut guard = ApplyGuard {
            running: self.running.clone(),
            lights_off: self.lights_off.clone(),
            publisher: self.publisher.clone(),
            _slot: slot,
            gate: None,
        };
        guard.gate = Some(self.gate.clone().write_owned().await);
        guard
    }
}

impl<S: ViewSink> Drop for ApplyGuard<S> {
    fn drop(&mut self) {
        self.gate.take();
        self.running.store(false, Ordering::SeqCst);
        self.lights_off.store(false, Ordering::SeqCst);
        self.publisher.set_applying(None);
    }
}
