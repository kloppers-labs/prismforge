mod diagnostics;
mod identify;
mod reconnect;
mod runner;
mod snapshot;

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot, watch};

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;

pub use diagnostics::{
    Counters, Diagnostics, ErrorRecord, Operation, SessionEvent, SessionEventKind,
};
pub use identify::{IDENTIFY_COLOR, IdentifyOutcome, RestoreFailure, RestoreOutcome};
pub use reconnect::ReconnectPolicy;
pub use runner::Session;
pub use snapshot::{IdentifyTarget, SessionDevice, SessionSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfig {
    pub command_capacity: usize,
    pub event_capacity: usize,
    pub reconnect: ReconnectPolicy,
    pub max_identify_duration: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            command_capacity: 8,
            event_capacity: 64,
            reconnect: ReconnectPolicy::default(),
            max_identify_duration: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum SessionError {
    #[error("the session is busy with earlier commands")]
    Busy,
    #[error("the session has stopped")]
    Stopped,
    #[error("an identify of {device} is still running")]
    IdentifyInProgress { device: DeviceRef },
    #[error("identify duration must be above zero and at most {max_ms} ms")]
    InvalidDuration { max_ms: u64 },
    #[error(transparent)]
    Backend(#[from] BackendError),
}

type Reply<T> = oneshot::Sender<Result<T, SessionError>>;

pub(crate) struct IdentifyRequest {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    pub duration: Duration,
}

pub(crate) enum Command {
    Connect(Reply<()>),
    Disconnect(Reply<()>),
    Refresh(Reply<()>),
    Identify {
        request: IdentifyRequest,
        reply: Reply<IdentifyOutcome>,
    },
}

/// Cheap, cloneable access to a running [`Session`]; commands are queued or refused at the call.
#[derive(Clone)]
pub struct SessionHandle {
    commands: mpsc::Sender<Command>,
    snapshot: Arc<watch::Sender<SessionSnapshot>>,
    refused_busy: Arc<AtomicU64>,
}

impl SessionHandle {
    pub fn snapshot(&self) -> SessionSnapshot {
        self.snapshot.borrow().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<SessionSnapshot> {
        self.snapshot.subscribe()
    }

    pub fn connect(&self) -> impl Future<Output = Result<(), SessionError>> + Send + use<> {
        self.send(Command::Connect)
    }

    pub fn disconnect(&self) -> impl Future<Output = Result<(), SessionError>> + Send + use<> {
        self.send(Command::Disconnect)
    }

    pub fn refresh(&self) -> impl Future<Output = Result<(), SessionError>> + Send + use<> {
        self.send(Command::Refresh)
    }

    /// Lights `device` (or one `zone`) for `duration`, then restores what the inventory reported.
    pub fn identify(
        &self,
        device: DeviceRef,
        zone: Option<usize>,
        duration: Duration,
    ) -> impl Future<Output = Result<IdentifyOutcome, SessionError>> + Send + use<> {
        let request = IdentifyRequest {
            device,
            zone,
            duration,
        };
        self.send(|reply| Command::Identify { request, reply })
    }

    fn send<T: Send + 'static, F: FnOnce(Reply<T>) -> Command>(
        &self,
        command: F,
    ) -> impl Future<Output = Result<T, SessionError>> + Send + use<T, F> {
        let (reply, response) = oneshot::channel();
        let queued = match self.commands.try_send(command(reply)) {
            Ok(()) => Ok(response),
            Err(TrySendError::Full(_)) => {
                let refused = self.refused_busy.fetch_add(1, Ordering::Relaxed) + 1;
                self.snapshot.send_modify(|snapshot| {
                    let counters = &mut snapshot.diagnostics.counters;
                    counters.commands_refused_busy = counters.commands_refused_busy.max(refused);
                });
                Err(SessionError::Busy)
            }
            Err(TrySendError::Closed(_)) => Err(SessionError::Stopped),
        };
        async move { queued?.await.unwrap_or(Err(SessionError::Stopped)) }
    }
}

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub fn new(backend: B, config: SessionConfig) -> (SessionHandle, Self) {
        let (commands, receiver) = mpsc::channel(config.command_capacity.max(1));
        let refused_busy = Arc::new(AtomicU64::new(0));
        let session = Self::start(backend, config, receiver, refused_busy.clone());
        let handle = SessionHandle {
            commands,
            snapshot: session.publisher(),
            refused_busy,
        };
        (handle, session)
    }
}
