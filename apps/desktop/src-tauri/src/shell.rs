use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use prismforge_core::session::{
    IdentifyOutcome, MAX_BATCH_TARGETS, Session, SessionConfig, SessionError, SessionHandle,
    SessionSnapshot,
};
use prismforge_core::{ConnectionState, DeviceRef, OutputTarget, RgbBackend};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::apply_lock::{ApplyLock, Permit};
use crate::forward::forward;
use crate::on_connect::{Report, apply_on_connect};
use crate::publisher::{Publisher, ViewSink};
use crate::stores::{StoreKind, Stores};
use crate::view::{CommandError, OnConnect, ShellView};

#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub session: SessionConfig,
    pub view_spacing: Duration,
    pub identify_duration: Duration,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            session: SessionConfig::default(),
            view_spacing: Duration::from_millis(50),
            identify_duration: Duration::from_secs(3),
        }
    }
}

/// Owns at most one session; a new session starts only after the previous one has fully stopped.
pub struct Shell<S: ViewSink> {
    config: ShellConfig,
    pub(crate) publisher: Arc<Publisher<S>>,
    // Lock order: active, apply slot, output gate, on-connect report, store; a session handle holder never waits for active.
    active: Mutex<Option<Active<S>>>,
    pub(crate) stores: Stores,
    pub(crate) lock: Arc<ApplyLock<S>>,
}

struct Active<S> {
    handle: SessionHandle,
    kind: StoreKind,
    report: Option<Arc<Report<S>>>,
    session: JoinHandle<()>,
    forwarder: JoinHandle<()>,
    keeper: Option<JoinHandle<()>>,
    watcher: Option<JoinHandle<()>>,
    live: Option<JoinHandle<()>>,
}

impl<S: ViewSink> Shell<S> {
    pub fn new(sink: S, config: ShellConfig, stores: Stores) -> Self {
        let publisher = Arc::new(Publisher::new(sink));
        Self {
            config,
            lock: Arc::new(ApplyLock::new(publisher.clone())),
            publisher,
            active: Mutex::new(None),
            stores,
        }
    }

    pub fn view(&self) -> ShellView {
        self.publisher.current()
    }

    pub fn changes(&self) -> tokio::sync::watch::Receiver<u64> {
        self.publisher.changes()
    }

    /// The profile applied or saved in this session, from a store that is already open.
    pub async fn active_profile_name(&self) -> Option<String> {
        let (handle, kind) = self.current().await;
        let profiles = self.stores.opened(kind).await?;
        profiles.active_name(&handle?).await
    }

    /// The session's latest snapshot, which may be newer than the last published view.
    pub async fn snapshot(&self) -> Option<SessionSnapshot> {
        self.current().await.0.map(|handle| handle.snapshot())
    }

    /// The profile that connect at start applies: the mark in the real store, which opens if needed.
    pub async fn start_profile(&self) -> crate::lifecycle::StartProfile {
        use crate::lifecycle::StartProfile;
        use crate::on_connect::{Mark, marked};
        match self.stores.profiles(StoreKind::Real).await {
            Err(reason) => StartProfile::Unavailable { reason },
            Ok(profiles) => match marked(&profiles).await {
                None => StartProfile::NotMarked,
                Some(Mark::Ready(_, name)) => StartProfile::Marked { name },
                Some(Mark::Unreadable(_, problem)) => StartProfile::Unreadable { problem },
            },
        }
    }

    pub fn folder(&self) -> String {
        self.stores.root().display().to_string()
    }

    pub async fn settings(
        &self,
    ) -> Result<crate::settings::SettingsStore, crate::stores::StoreUnavailable> {
        self.stores.settings().await
    }

    /// Seeds the backend with the stored registry and keeps it beside the session, then connects.
    pub async fn start<B: RgbBackend + Send + 'static>(
        &self,
        mut backend: B,
    ) -> Result<ShellView, CommandError> {
        let mut active = self.active.lock().await;
        end(active.take()).await;
        self.publisher.set_connect_apply(None);
        self.publisher.set_on_connect(OnConnect::NotApplicable);
        let kind = StoreKind::of(backend.info().origin);
        let profiles = self.stores.profiles(kind).await.ok();
        if let Some(profiles) = &profiles {
            backend.seed_known_devices(profiles.known_devices().await);
        }
        let (handle, session) = Session::new(backend, self.config.session.clone());
        let session = tokio::spawn(session.run());
        let keeper = profiles.clone().map(|profiles| {
            let known = handle.known_devices();
            tokio::spawn(async move { profiles.keep_registry(known).await })
        });
        let report =
            profiles.map(|profiles| Arc::new(Report::new(profiles, self.publisher.clone())));
        if let Some(report) = &report {
            report.publish().await;
        }
        let watcher = report.clone().map(|report| {
            tokio::spawn(apply_on_connect(
                report,
                handle.clone(),
                self.lock.clone(),
                self.publisher.clone(),
            ))
        });
        let mut updates = handle.subscribe();
        drop(handle.connect());
        let first = updates
            .wait_for(|snapshot| !is_unstarted(snapshot))
            .await
            .map(|snapshot| snapshot.clone());
        let Ok(first) = first else {
            stop_beside(watcher).await;
            drop(handle);
            let _ = session.await;
            finish(keeper).await;
            self.publisher.publish(None);
            return Err(SessionError::Stopped.into());
        };
        let view = self.publisher.publish(Some(first));
        let forwarder = tokio::spawn(forward(
            updates,
            self.publisher.clone(),
            self.config.view_spacing,
        ));
        *active = Some(Active {
            handle,
            kind,
            report,
            session,
            forwarder,
            keeper,
            watcher,
            live: None,
        });
        Ok(view)
    }

    pub async fn stop(&self) -> ShellView {
        let mut active = self.active.lock().await;
        end(active.take()).await;
        self.publisher.set_connect_apply(None);
        self.publisher.set_on_connect(OnConnect::NotApplicable);
        self.publisher.publish(None)
    }

    /// Publishes the apply-on-connect state again; called after every command that can change the mark.
    pub(crate) async fn report_on_connect(&self) {
        let report = self
            .active
            .lock()
            .await
            .as_ref()
            .and_then(|active| active.report.clone());
        if let Some(report) = report {
            report.publish().await;
        }
    }

    /// Sends the live output of `targets` to the window until the next call; an empty list stops it.
    pub async fn watch_live(&self, targets: Vec<OutputTarget>) -> Result<(), CommandError> {
        if targets.len() > MAX_BATCH_TARGETS {
            return Err(CommandError::TooManyTargets {
                limit: MAX_BATCH_TARGETS,
            });
        }
        let mut active = self.active.lock().await;
        let active = active.as_mut().ok_or(CommandError::NoSession)?;
        if let Some(task) = active.live.take() {
            task.abort();
        }
        if targets.is_empty() {
            return Ok(());
        }
        let mut feed = active.handle.live_output(targets);
        let publisher = self.publisher.clone();
        active.live = Some(tokio::spawn(async move {
            while let Some(live) = feed.next().await {
                publisher.deliver_live(&live);
            }
        }));
        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), CommandError> {
        self.queue(SessionHandle::refresh).await?.await?;
        Ok(())
    }

    pub async fn identify(
        &self,
        device: DeviceRef,
        zone: Option<usize>,
    ) -> Result<IdentifyOutcome, CommandError> {
        let duration = self.config.identify_duration;
        let (_permit, reply) = self
            .queue_output(|handle| handle.identify(device, zone, duration))
            .await?;
        Ok(reply.await?)
    }

    /// The session's handle, if any, and the store that belongs to it; no session means the real store.
    pub(crate) async fn current(&self) -> (Option<SessionHandle>, StoreKind) {
        parts(self.active.lock().await.as_ref())
    }

    /// Like [`Self::current`], with an output permit taken while `active` is held.
    pub(crate) async fn permitted(
        &self,
    ) -> Result<(Permit, Option<SessionHandle>, StoreKind), CommandError> {
        let active = self.active.lock().await;
        let permit = self.lock.permit()?;
        let (handle, kind) = parts(active.as_ref());
        Ok((permit, handle, kind))
    }

    /// Queues an output command and takes its permit while `active` is held.
    pub(crate) async fn queue_output<F: Future>(
        &self,
        command: impl FnOnce(&SessionHandle) -> F,
    ) -> Result<(Permit, F), CommandError> {
        let active = self.active.lock().await;
        let active = active.as_ref().ok_or(CommandError::NoSession)?;
        let permit = self.lock.permit()?;
        Ok((permit, command(&active.handle)))
    }

    /// Queues while holding the lock, so a command never reaches a session that is being replaced.
    pub(crate) async fn queue<F: Future>(
        &self,
        command: impl FnOnce(&SessionHandle) -> F,
    ) -> Result<F, CommandError> {
        let active = self.active.lock().await;
        let active = active.as_ref().ok_or(CommandError::NoSession)?;
        Ok(command(&active.handle))
    }
}

fn parts<S>(active: Option<&Active<S>>) -> (Option<SessionHandle>, StoreKind) {
    active.map_or((None, StoreKind::Real), |active| {
        (Some(active.handle.clone()), active.kind)
    })
}

/// The watcher holds a handle, so it stops first; the keeper writes the last registry after the session.
async fn end<S: ViewSink>(active: Option<Active<S>>) {
    let Some(Active {
        handle,
        report,
        session,
        forwarder,
        keeper,
        watcher,
        live,
        ..
    }) = active
    else {
        return;
    };
    forwarder.abort();
    let _ = forwarder.await;
    stop_beside(live).await;
    stop_beside(watcher).await;
    if let Some(report) = report {
        report.close().await;
    }
    drop(handle);
    let _ = session.await;
    finish(keeper).await;
}

async fn finish(task: Option<JoinHandle<()>>) {
    if let Some(task) = task {
        let _ = task.await;
    }
}

async fn stop_beside(task: Option<JoinHandle<()>>) {
    if let Some(task) = task {
        task.abort();
        let _ = task.await;
    }
}

fn is_unstarted(snapshot: &SessionSnapshot) -> bool {
    snapshot.connection == ConnectionState::Disconnected { reason: None }
}

#[cfg(test)]
mod tests;
