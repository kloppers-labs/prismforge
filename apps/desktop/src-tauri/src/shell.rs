use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use prismforge_core::session::{
    IdentifyOutcome, Session, SessionConfig, SessionError, SessionHandle, SessionSnapshot,
};
use prismforge_core::{ConnectionState, DeviceRef, RgbBackend};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::apply_lock::{ApplyLock, Permit};
use crate::forward::forward;
use crate::on_connect::apply_on_connect;
use crate::publisher::{Publisher, ViewSink};
use crate::stores::{StoreKind, Stores};
use crate::view::{CommandError, ShellView};

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
    publisher: Arc<Publisher<S>>,
    active: Mutex<Option<Active>>,
    pub(crate) stores: Stores,
    pub(crate) lock: Arc<ApplyLock<S>>,
}

struct Active {
    handle: SessionHandle,
    kind: StoreKind,
    session: JoinHandle<()>,
    forwarder: JoinHandle<()>,
    keeper: Option<JoinHandle<()>>,
    watcher: Option<JoinHandle<()>>,
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

    /// Seeds the backend with the stored registry and keeps it beside the session, then connects.
    pub async fn start<B: RgbBackend + Send + 'static>(
        &self,
        mut backend: B,
    ) -> Result<ShellView, CommandError> {
        let mut active = self.active.lock().await;
        end(active.take()).await;
        self.publisher.set_connect_apply(None);
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
        let watcher = profiles.map(|profiles| {
            tokio::spawn(apply_on_connect(
                profiles,
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
            session,
            forwarder,
            keeper,
            watcher,
        });
        Ok(view)
    }

    pub async fn stop(&self) -> ShellView {
        let mut active = self.active.lock().await;
        end(active.take()).await;
        self.publisher.publish(None)
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
        let _permit = self.permit()?;
        let duration = self.config.identify_duration;
        let reply = self
            .queue(|handle| handle.identify(device, zone, duration))
            .await?;
        Ok(reply.await?)
    }

    pub(crate) fn permit(&self) -> Result<Permit, CommandError> {
        self.lock.permit()
    }

    /// The session's handle, if any, and the store that belongs to it; no session means the real store.
    pub(crate) async fn current(&self) -> (Option<SessionHandle>, StoreKind) {
        let active = self.active.lock().await;
        active.as_ref().map_or((None, StoreKind::Real), |active| {
            (Some(active.handle.clone()), active.kind)
        })
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

/// The watcher holds a handle, so it stops first; the keeper writes the last registry after the session.
async fn end(active: Option<Active>) {
    let Some(Active {
        handle,
        session,
        forwarder,
        keeper,
        watcher,
        ..
    }) = active
    else {
        return;
    };
    forwarder.abort();
    let _ = forwarder.await;
    stop_beside(watcher).await;
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
