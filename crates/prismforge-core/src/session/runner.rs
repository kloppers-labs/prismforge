use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::{mpsc, watch};
use tokio::time::{Instant, sleep_until};

use crate::backend::{BackendEvent, RgbBackend};
use crate::connection::{ConnectionState, LossCause, StateReason};
use crate::error::BackendError;
use crate::inventory::{Inventory, InventoryDevice};
use crate::session::diagnostics::{
    Counters, Diagnostics, ErrorRecord, EventRing, Operation, SessionEventKind,
};
use crate::session::identify::{IdentifyOutcome, IdentifyPlan, RestoreFailure, RestoreOutcome};
use crate::session::snapshot::{IdentifyTarget, SessionDevice, SessionSnapshot};
use crate::session::{Command, IdentifyRequest, Reply, SessionConfig, SessionError};

/// Sole owner of a backend: runs every backend call to completion in one task.
pub struct Session<B> {
    backend: B,
    config: SessionConfig,
    commands: mpsc::Receiver<Command>,
    snapshot: Arc<watch::Sender<SessionSnapshot>>,
    refused_busy: Arc<AtomicU64>,
    started: Instant,
    link: Link,
    inventory: Option<Inventory>,
    seen: Vec<InventoryDevice>,
    identify: Option<ActiveIdentify>,
    last_error: Option<ErrorRecord>,
    counters: Counters,
    events: EventRing,
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
    Event(Option<BackendEvent>),
    Command(Command),
    Closed,
}

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(crate) fn start(
        backend: B,
        config: SessionConfig,
        commands: mpsc::Receiver<Command>,
        refused_busy: Arc<AtomicU64>,
    ) -> Self {
        let snapshot = SessionSnapshot {
            connection: ConnectionState::Disconnected { reason: None },
            backend: backend.info(),
            devices: Vec::new(),
            issues: Vec::new(),
            identify: None,
            diagnostics: Diagnostics::default(),
        };
        Self {
            events: EventRing::new(config.event_capacity),
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
            last_error: None,
            counters: Counters::default(),
        }
    }

    pub(crate) fn publisher(&self) -> Arc<watch::Sender<SessionSnapshot>> {
        self.snapshot.clone()
    }

    /// Runs until every [`SessionHandle`](crate::session::SessionHandle) is dropped.
    pub async fn run(mut self) {
        loop {
            let deadline = self.deadline();
            let listening = matches!(self.link, Link::Connected);
            let wake = tokio::select! {
                biased;
                () = sleep_until(deadline.unwrap_or_else(Instant::now)), if deadline.is_some() => {
                    Wake::Deadline
                }
                event = self.backend.next_event(), if listening => Wake::Event(event),
                command = self.commands.recv() => command.map_or(Wake::Closed, Wake::Command),
            };
            match wake {
                Wake::Deadline => self.on_deadline().await,
                Wake::Event(event) => self.on_event(event).await,
                Wake::Command(command) => self.on_command(command).await,
                Wake::Closed => break,
            }
            self.publish();
        }
        self.stop().await;
        self.publish();
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
            Some(BackendEvent::InventoryChanged) => {
                self.push(SessionEventKind::InventoryChanged);
                let _ = self.refresh_or_lose(1).await;
            }
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
                let _ = reply.send(result.map_err(SessionError::from));
            }
            Command::Disconnect(reply) => {
                self.stop().await;
                self.push(SessionEventKind::Disconnected);
                let _ = reply.send(Ok(()));
            }
            Command::Refresh(reply) => {
                let result = match self.link {
                    Link::Connected => self.refresh_or_lose(1).await,
                    _ => Err(BackendError::NotConnected),
                };
                let _ = reply.send(result.map_err(SessionError::from));
            }
            Command::Identify { request, reply } => self.start_identify(request, reply).await,
        }
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
            if matches!(error, BackendError::UnsupportedProtocol { .. }) {
                self.link = Link::Idle {
                    reason: Some(reason),
                };
            } else {
                self.schedule_retry(failures + 1, reason);
            }
            return Err(error);
        }
        self.link = Link::Connected;
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
        self.inventory = Some(inventory);
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
        self.backend.disconnect();
        self.link = Link::Idle { reason: None };
    }

    async fn start_identify(&mut self, request: IdentifyRequest, reply: Reply<IdentifyOutcome>) {
        let plan = match self.plan_identify(&request) {
            Ok(plan) => plan,
            Err(error) => {
                let _ = reply.send(Err(error));
                return;
            }
        };
        self.push(SessionEventKind::IdentifyStarted {
            device: plan.device.clone(),
            zone: plan.zone,
        });
        let Err(error) = plan.light(&mut self.backend).await else {
            self.identify = Some(ActiveIdentify {
                plan,
                until: Instant::now() + request.duration,
                reply,
            });
            return;
        };
        let lost = self.lost_reason();
        self.record_error(Operation::Identify, &error, lost.clone());
        match lost {
            Some(reason) => {
                let restore = not_restored_after_loss(&reason);
                self.reply_identify(plan, reply, Some(error), restore);
                self.connection_lost(reason, 1);
            }
            None => {
                let restore = self.restore(&plan).await;
                self.reply_identify(plan, reply, Some(error), restore);
                self.lose_if_disconnected();
            }
        }
    }

    fn plan_identify(&self, request: &IdentifyRequest) -> Result<IdentifyPlan, SessionError> {
        if let Some(active) = &self.identify {
            return Err(SessionError::IdentifyInProgress {
                device: active.plan.device.clone(),
            });
        }
        let max = self.config.max_identify_duration;
        if request.duration.is_zero() || request.duration > max {
            return Err(SessionError::InvalidDuration {
                max_ms: max.as_millis() as u64,
            });
        }
        let inventory = match (&self.link, self.backend.inventory()) {
            (Link::Connected, Some(inventory)) => inventory,
            _ => return Err(BackendError::NotConnected.into()),
        };
        Ok(IdentifyPlan::new(inventory, &request.device, request.zone)?)
    }

    async fn finish_identify(&mut self, active: ActiveIdentify) {
        let restore = self.restore(&active.plan).await;
        self.reply_identify(active.plan, active.reply, None, restore);
        self.lose_if_disconnected();
    }

    async fn restore(&mut self, plan: &IdentifyPlan) -> RestoreOutcome {
        let Err(failed) = plan.restore(&mut self.backend).await else {
            return RestoreOutcome::Restored;
        };
        let lost = self.lost_reason();
        self.record_error(Operation::Restore, &failed.error, lost.clone());
        let failure = match lost {
            Some(reason) => RestoreFailure::ConnectionLost { reason },
            None => RestoreFailure::Refused {
                error: failed.error,
            },
        };
        if failed.steps_done == 0 {
            RestoreOutcome::NotRestored { failure }
        } else {
            RestoreOutcome::PartlyRestored { failure }
        }
    }

    fn reply_identify(
        &mut self,
        plan: IdentifyPlan,
        reply: Reply<IdentifyOutcome>,
        light_failure: Option<BackendError>,
        restore: RestoreOutcome,
    ) {
        let outcome = IdentifyOutcome {
            device: plan.device,
            zone: plan.zone,
            light_failure,
            restore,
        };
        self.push(SessionEventKind::IdentifyFinished {
            outcome: outcome.clone(),
        });
        let _ = reply.send(Ok(outcome));
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

    fn publish(&self) {
        let connection = match &self.link {
            Link::Idle { reason } => ConnectionState::Disconnected {
                reason: reason.clone(),
            },
            Link::Connecting => ConnectionState::Connecting,
            Link::Connected => self.backend.connection_state().clone(),
            Link::Retrying {
                attempt, reason, ..
            } => ConnectionState::Retrying {
                attempt: *attempt,
                reason: reason.clone(),
            },
        };
        let next = SessionSnapshot {
            connection,
            backend: self.backend.info(),
            devices: self.devices(),
            issues: self
                .inventory
                .as_ref()
                .map(|inventory| inventory.issues.clone())
                .unwrap_or_default(),
            identify: self.identify.as_ref().map(|active| IdentifyTarget {
                device: active.plan.device.clone(),
                zone: active.plan.zone,
            }),
            diagnostics: Diagnostics {
                last_error: self.last_error.clone(),
                counters: self.counters,
                events: self.events.to_vec(),
            },
        };
        let refused = self.refused_busy.load(Ordering::Relaxed);
        self.snapshot.send_modify(|snapshot| {
            let published = snapshot.diagnostics.counters.commands_refused_busy;
            *snapshot = next;
            snapshot.diagnostics.counters.commands_refused_busy = published.max(refused);
        });
    }

    fn devices(&self) -> Vec<SessionDevice> {
        let Some(inventory) = &self.inventory else {
            return Vec::new();
        };
        let available = matches!(self.link, Link::Connected);
        let current = inventory.devices.iter().map(|device| (device, available));
        let missing = inventory.missing.iter().filter_map(|reference| {
            self.seen
                .iter()
                .find(|seen| seen.identity.reference() == Some(reference))
                .map(|device| (device, false))
        });
        current
            .chain(missing)
            .map(|(device, available)| SessionDevice {
                identity: device.identity.clone(),
                descriptor: device.descriptor.clone(),
                available,
            })
            .collect()
    }
}

fn not_restored_after_loss(reason: &StateReason) -> RestoreOutcome {
    RestoreOutcome::NotRestored {
        failure: RestoreFailure::ConnectionLost {
            reason: reason.clone(),
        },
    }
}

const UNEXPLAINED_LOSS: StateReason = StateReason::ConnectionLost {
    cause: LossCause::ServerClosed,
};
