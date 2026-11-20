use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use tokio::sync::{mpsc, watch};
use tokio::time::{Instant, sleep_until};

use crate::backend::{BackendEvent, RgbBackend};
use crate::connection::{ConnectionState, LossCause, StateReason};
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::identity::KnownDevice;
use crate::inventory::{Inventory, InventoryDevice};
use crate::scene::Scene;
use crate::session::diagnostics::{
    Counters, Diagnostics, ErrorRecord, EventRing, Operation, SessionEventKind,
};
use crate::session::identify::{IdentifyOutcome, IdentifyPlan, RestoreFailure, RestoreOutcome};
use crate::session::intent::Intents;
use crate::session::live::{self, LiveChannels, LiveRequest};
use crate::session::output::Output;
use crate::session::snapshot::{OutputStatus, SessionSnapshot};
use crate::session::{Command, Reply, SessionConfig, SessionError};

mod automation;
mod batches;
mod claims;
mod frames;
mod identifying;
mod intents;
mod publishing;
mod restoring;

/// Sole owner of a backend: runs every backend call to completion in one task.
pub struct Session<B> {
    backend: B,
    config: SessionConfig,
    commands: mpsc::Receiver<Command>,
    snapshot: Arc<watch::Sender<SessionSnapshot>>,
    known: Arc<watch::Sender<Vec<KnownDevice>>>,
    refused_busy: Arc<AtomicU64>,
    started: Instant,
    link: Link,
    inventory: Option<Inventory>,
    seen: Vec<InventoryDevice>,
    identify: Option<ActiveIdentify>,
    unrestored: Vec<DeviceRef>,
    tainted: Option<DeviceRef>,
    last_auto_refresh: Option<Instant>,
    last_error: Option<ErrorRecord>,
    counters: Counters,
    events: EventRing,
    output: Output,
    intents: Intents,
    live: LiveChannels,
    live_requests: watch::Receiver<LiveRequest>,
    live_open: bool,
}

enum Link {
    Idle {
        reason: Option<StateReason>,
    },
    Connecting,
    Connected,
    Retrying {
        attempt: u32,
        reason: StateReason,
        at: Instant,
    },
}

struct ActiveIdentify {
    plan: IdentifyPlan,
    until: Instant,
    reply: Reply<IdentifyOutcome>,
}

enum Wake {
    Deadline,
    StaleInventory,
    Event(Option<BackendEvent>),
    Command(Command),
    Frame,
    LiveRequest(bool),
    Closed,
}

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(crate) fn start(
        backend: B,
        config: SessionConfig,
        commands: mpsc::Receiver<Command>,
        refused_busy: Arc<AtomicU64>,
        live: LiveChannels,
    ) -> Self {
        let output = Output::new(config.frame_rate);
        let snapshot = SessionSnapshot {
            connection: ConnectionState::Disconnected { reason: None },
            backend: backend.info(),
            devices: Vec::new(),
            issues: Vec::new(),
            identify: None,
            output: OutputStatus {
                frame_rate: output.frame_rate,
                scene: Scene::default(),
                devices: Vec::new(),
            },
            diagnostics: Diagnostics::default(),
        };
        Self {
            events: EventRing::new(config.event_capacity),
            known: Arc::new(watch::channel(backend.known_devices().to_vec()).0),
            backend,
            config,
            commands,
            snapshot: Arc::new(watch::channel(snapshot).0),
            refused_busy,
            started: Instant::now(),
            link: Link::Idle { reason: None },
            inventory: None,
            seen: Vec::new(),
            identify: None,
            unrestored: Vec::new(),
            tainted: None,
            last_auto_refresh: None,
            last_error: None,
            counters: Counters::default(),
            output,
            intents: Intents::default(),
            live_requests: live.requests.subscribe(),
            live,
            live_open: true,
        }
    }

    /// Publishes first, so a caller that sees the reply also sees its result in the snapshot.
    pub(super) fn answer<T, E>(&self, reply: Reply<T, E>, result: Result<T, E>) {
        self.publish();
        let _ = reply.send(result);
    }

    pub(crate) fn publisher(&self) -> Arc<watch::Sender<SessionSnapshot>> {
        self.snapshot.clone()
    }

    pub(crate) fn registry(&self) -> Arc<watch::Sender<Vec<KnownDevice>>> {
        self.known.clone()
    }

    /// Runs until every [`SessionHandle`](crate::session::SessionHandle) is dropped.
    pub async fn run(mut self) {
        loop {
            let deadline = self.deadline();
            let listening = matches!(self.link, Link::Connected);
            let auto_refresh = self.auto_refresh_at();
            let frame = self.output_due();
            let frame_first = frame.is_some() && self.output.frame_owed;
            let wake = tokio::select! {
                biased;
                () = sleep_until(deadline.unwrap_or_else(Instant::now)), if deadline.is_some() => {
                    Wake::Deadline
                }
                () = sleep_until(frame.unwrap_or_else(Instant::now)), if frame_first => Wake::Frame,
                command = self.commands.recv() => command.map_or(Wake::Closed, Wake::Command),
                event = self.backend.next_event(), if listening => Wake::Event(event),
                changed = self.live_requests.changed(), if self.live_open => {
                    Wake::LiveRequest(changed.is_ok())
                }
                () = sleep_until(auto_refresh.unwrap_or_else(Instant::now)), if auto_refresh.is_some() => {
                    Wake::StaleInventory
                }
                () = sleep_until(frame.unwrap_or_else(Instant::now)), if frame.is_some() && !frame_first => {
                    Wake::Frame
                }
            };
            match wake {
                Wake::Deadline => self.on_deadline().await,
                Wake::StaleInventory => {
                    self.last_auto_refresh = Some(Instant::now());
                    let _ = self.refresh_or_lose(1).await;
                }
                Wake::Event(event) => self.on_event(event).await,
                Wake::Command(command) => {
                    self.output.frame_owed = frame.is_some_and(|at| at <= Instant::now());
                    self.on_command(command).await;
                }
                Wake::Frame => self.on_frame().await,
                Wake::LiveRequest(open) => self.live_open = open,
                Wake::Closed => break,
            }
            self.publish();
            self.publish_live();
        }
        self.stop().await;
        self.publish();
    }

    /// Copies the output state for the live feed; no backend call, so it cannot change what reaches a device.
    fn publish_live(&mut self) {
        let request = self.live_requests.borrow_and_update().clone();
        let fresh = if request.targets.is_empty() {
            live::LiveOutput::default()
        } else {
            live::collect(&request, &self.output, self.inventory.as_ref())
        };
        self.live.output.send_if_modified(|current| {
            let changed = *current != fresh;
            if changed {
                *current = fresh;
            }
            changed
        });
    }

    fn deadline(&self) -> Option<Instant> {
        match (&self.identify, &self.link) {
            (Some(active), _) => Some(active.until),
            (None, Link::Retrying { at, .. }) => Some(*at),
            _ => None,
        }
    }

    async fn on_deadline(&mut self) {
        if let Some(active) = self.identify.take() {
            self.finish_identify(active).await;
        } else if let Link::Retrying { attempt, .. } = self.link {
            let _ = self.try_connect(attempt).await;
        }
    }

    async fn on_event(&mut self, event: Option<BackendEvent>) {
        match event {
            Some(BackendEvent::InventoryChanged) => self.push(SessionEventKind::InventoryChanged),
            Some(BackendEvent::ConnectionLost { cause, .. }) => {
                self.connection_lost(StateReason::ConnectionLost { cause }, 1);
            }
            None => {
                let reason = self.lost_reason().unwrap_or(UNEXPLAINED_LOSS);
                self.connection_lost(reason, 1);
            }
        }
    }

    async fn on_command(&mut self, command: Command) {
        match command {
            Command::Connect(reply) => {
                let result = self.connect().await;
                self.answer(reply, result.map_err(SessionError::from));
            }
            Command::Intents { after, reply } => {
                let log = self.intents.after(after);
                self.answer(reply, Ok(log));
            }
            Command::Disconnect(reply) => {
                self.stop().await;
                self.push(SessionEventKind::Disconnected);
                self.answer(reply, Ok(()));
            }
            Command::Refresh(reply) => {
                let result = match self.link {
                    Link::Connected => self.refresh_or_lose(1).await,
                    _ => Err(BackendError::NotConnected),
                };
                self.answer(reply, result.map_err(SessionError::from));
            }
            Command::Identify { request, reply } => self.start_identify(request, reply).await,
            Command::EventProfiles(reply) => {
                let result = match self.link {
                    Link::Connected => self.read_event_profiles().await,
                    _ => Err(BackendError::NotConnected),
                };
                self.answer(reply, result.map_err(SessionError::from));
            }
            Command::Resync(reply) => {
                self.forget_device_state();
                self.answer(reply, Ok(()));
            }
            Command::Output(command) => self.on_output(command).await,
        }
    }

    /// Pending while the connected backend reports a stale inventory; output calls can absorb its event.
    fn auto_refresh_at(&self) -> Option<Instant> {
        (self.backend_stale() && matches!(self.link, Link::Connected)).then(|| {
            self.last_auto_refresh
                .map_or_else(Instant::now, |last| last + AUTO_REFRESH_SPACING)
        })
    }

    fn live_inventory(&self) -> Result<&Inventory, BackendError> {
        match (&self.link, self.backend.inventory()) {
            (Link::Connected, Some(inventory)) => Ok(inventory),
            _ => Err(BackendError::NotConnected),
        }
    }

    fn link_down(&self) -> Option<StateReason> {
        match &self.link {
            Link::Connected => None,
            Link::Retrying { reason, .. } => Some(reason.clone()),
            Link::Idle { .. } | Link::Connecting => {
                Some(self.lost_reason().unwrap_or(UNEXPLAINED_LOSS))
            }
        }
    }

    fn backend_stale(&self) -> bool {
        matches!(
            self.backend.connection_state(),
            ConnectionState::Degraded { reasons } if reasons.contains(&StateReason::InventoryStale)
        )
    }

    async fn connect(&mut self) -> Result<(), BackendError> {
        if matches!(self.link, Link::Connected) {
            return Ok(());
        }
        self.link = Link::Connecting;
        self.publish();
        self.try_connect(0).await
    }

    /// `failures` is the number of attempts that failed in a row before this one.
    async fn try_connect(&mut self, failures: u32) -> Result<(), BackendError> {
        self.counters.connect_attempts += 1;
        self.push(SessionEventKind::ConnectAttempt);
        if let Err(error) = self.backend.connect().await {
            self.counters.connect_failures += 1;
            let reason = self.lost_reason().unwrap_or(StateReason::ServerUnreachable);
            self.record_error(Operation::Connect, &error, None);
            self.push(SessionEventKind::ConnectFailed {
                error: error.clone(),
            });
            if matches!(
                error,
                BackendError::UnsupportedProtocol {
                    server_version: Some(_),
                    ..
                }
            ) {
                self.link = Link::Idle {
                    reason: Some(reason),
                };
            } else {
                self.schedule_retry(failures + 1, reason);
            }
            return Err(error);
        }
        self.link = Link::Connected;
        self.output.after_reconnect();
        self.push(SessionEventKind::Connected);
        self.refresh_or_lose(failures + 1).await
    }

    async fn refresh_or_lose(&mut self, retry_attempt: u32) -> Result<(), BackendError> {
        let result = match self.backend.refresh_inventory().await {
            Ok(inventory) => Ok(inventory.clone()),
            Err(error) => Err(error),
        };
        match result {
            Ok(inventory) => {
                self.counters.inventory_refreshes += 1;
                self.push(SessionEventKind::InventoryRefreshed {
                    generation: inventory.generation,
                    devices: inventory.devices.len(),
                });
                self.adopt(inventory);
                Ok(())
            }
            Err(error) => {
                self.counters.refresh_failures += 1;
                let lost = self.lost_reason();
                self.record_error(Operation::Refresh, &error, lost.clone());
                self.push(SessionEventKind::RefreshFailed {
                    error: error.clone(),
                });
                if let Some(reason) = lost {
                    self.connection_lost(reason, retry_attempt);
                }
                Err(error)
            }
        }
    }

    fn adopt(&mut self, inventory: Inventory) {
        for device in &inventory.devices {
            let Some(reference) = device.identity.reference() else {
                continue;
            };
            self.seen
                .retain(|seen| seen.identity.reference() != Some(reference));
            self.seen.push(device.clone());
        }
        self.unrestored.retain(|reference| {
            inventory
                .device(reference)
                .is_none_or(|device| device.descriptor.capabilities.in_direct_mode())
        });
        if self.identify.is_none() {
            self.tainted = None;
        }
        self.output.after_inventory(&inventory);
        self.inventory = Some(inventory);
        let known = self.backend.known_devices();
        self.known.send_if_modified(|published| {
            let changed = published.as_slice() != known;
            if changed {
                *published = known.to_vec();
            }
            changed
        });
    }

    fn connection_lost(&mut self, reason: StateReason, retry_attempt: u32) {
        if let Some(active) = self.identify.take() {
            let restore = not_restored_after_loss(&reason);
            self.reply_identify(active.plan, active.reply, None, restore);
        }
        self.push(SessionEventKind::ConnectionLost {
            reason: reason.clone(),
        });
        self.schedule_retry(retry_attempt, reason);
    }

    fn schedule_retry(&mut self, attempt: u32, reason: StateReason) {
        let delay = self.config.reconnect.delay(attempt);
        self.link = Link::Retrying {
            attempt,
            reason,
            at: Instant::now() + delay,
        };
        self.push(SessionEventKind::RetryScheduled {
            attempt,
            delay_ms: delay.as_millis() as u64,
        });
    }

    async fn stop(&mut self) {
        if let Some(active) = self.identify.take() {
            self.finish_identify(active).await;
        }
        self.release_all().await;
        self.backend.disconnect();
        self.link = Link::Idle { reason: None };
    }

    fn lose_if_disconnected(&mut self) {
        if let (Link::Connected, Some(reason)) = (&self.link, self.lost_reason()) {
            self.connection_lost(reason, 1);
        }
    }

    fn lost_reason(&self) -> Option<StateReason> {
        match self.backend.connection_state() {
            ConnectionState::Disconnected { reason } => {
                Some(reason.clone().unwrap_or(UNEXPLAINED_LOSS))
            }
            _ => None,
        }
    }

    fn record_error(
        &mut self,
        operation: Operation,
        error: &BackendError,
        ended_connection: Option<StateReason>,
    ) {
        self.last_error = Some(ErrorRecord {
            at_ms: self.elapsed_ms(),
            operation,
            error: error.clone(),
            ended_connection,
        });
    }

    fn push(&mut self, kind: SessionEventKind) {
        let at_ms = self.elapsed_ms();
        self.events.push(at_ms, kind);
    }

    fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
}

pub(super) fn not_restored_after_loss(reason: &StateReason) -> RestoreOutcome {
    RestoreOutcome::NotRestored {
        failure: RestoreFailure::ConnectionLost {
            reason: reason.clone(),
        },
    }
}

const AUTO_REFRESH_SPACING: Duration = Duration::from_millis(250);

const UNEXPLAINED_LOSS: StateReason = StateReason::ConnectionLost {
    cause: LossCause::ServerClosed,
};
