use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use prismforge_core::session::{
    IdentifyOutcome, Session, SessionConfig, SessionError, SessionHandle, SessionSnapshot,
};
use prismforge_core::{ConnectionState, DeviceRef, RgbBackend};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::forward::forward;
use crate::publisher::{Publisher, ViewSink};
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
pub struct Shell<S> {
    config: ShellConfig,
    publisher: Arc<Publisher<S>>,
    active: Mutex<Option<Active>>,
}

struct Active {
    handle: SessionHandle,
    session: JoinHandle<()>,
    forwarder: JoinHandle<()>,
}

impl<S: ViewSink> Shell<S> {
    pub fn new(sink: S, config: ShellConfig) -> Self {
        Self {
            config,
            publisher: Arc::new(Publisher::new(sink)),
            active: Mutex::new(None),
        }
    }

    pub fn view(&self) -> ShellView {
        self.publisher.current()
    }

    pub async fn start<B: RgbBackend + Send + 'static>(
        &self,
        backend: B,
    ) -> Result<ShellView, CommandError> {
        let mut active = self.active.lock().await;
        end(active.take()).await;
        let (handle, session) = Session::new(backend, self.config.session.clone());
        let session = tokio::spawn(session.run());
        let mut updates = handle.subscribe();
        drop(handle.connect());
        let first = updates
            .wait_for(|snapshot| !is_unstarted(snapshot))
            .await
            .map(|snapshot| snapshot.clone());
        let Ok(first) = first else {
            drop(handle);
            let _ = session.await;
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
            session,
            forwarder,
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
        let duration = self.config.identify_duration;
        let reply = self
            .queue(|handle| handle.identify(device, zone, duration))
            .await?;
        Ok(reply.await?)
    }

    /// Queues while holding the lock, so a command never reaches a session that is being replaced.
    async fn queue<F: Future>(
        &self,
        command: impl FnOnce(&SessionHandle) -> F,
    ) -> Result<F, CommandError> {
        let active = self.active.lock().await;
        let active = active.as_ref().ok_or(CommandError::NoSession)?;
        Ok(command(&active.handle))
    }
}

async fn end(active: Option<Active>) {
    let Some(Active {
        handle,
        session,
        forwarder,
    }) = active
    else {
        return;
    };
    forwarder.abort();
    let _ = forwarder.await;
    drop(handle);
    let _ = session.await;
}

fn is_unstarted(snapshot: &SessionSnapshot) -> bool {
    snapshot.connection == ConnectionState::Disconnected { reason: None }
}
