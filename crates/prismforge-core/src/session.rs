mod batch;
mod diagnostics;
mod identify;
mod intent;
mod live;
mod output;
mod previous;
mod reconnect;
mod runner;
mod snapshot;

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use prismforge_effects::{EffectDefinition, EffectInstance, InstanceError};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot, watch};

use crate::automation::{EventProfiles, LightsOffPlan, LightsOffReport};
use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::identity::KnownDevice;
use crate::ownership::OutputOwner;
use crate::scene::{ApplyMode, Assignment, OutputTarget};
use crate::setup::AcrossPlan;

pub use batch::{BatchReport, MAX_BATCH_TARGETS, TargetOutcome, TargetResult};
pub use diagnostics::{
    Counters, Diagnostics, ErrorRecord, Operation, SessionEvent, SessionEventKind,
};
pub use identify::{IDENTIFY_COLOR, IdentifyOutcome, RestoreFailure, RestoreOutcome};
pub(crate) use intent::{Intent, IntentLog, IntentRecord, Origin, Slot};
use live::LiveChannels;
pub use live::{LIVE_OUTPUT_RATE, LiveDevice, LiveFeed, LiveOutput, LiveZone};
pub use output::{MAX_FRAME_RATE, MIN_FRAME_RATE};
pub use reconnect::ReconnectPolicy;
pub use runner::Session;
pub use snapshot::{
    DeviceOutput, IdentifyTarget, OutputCondition, OutputStatus, PauseReason, SessionDevice,
    SessionSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfig {
    pub command_capacity: usize,
    pub event_capacity: usize,
    pub reconnect: ReconnectPolicy,
    pub max_identify_duration: Duration,
    /// Output ticks per second; clamped to [`MIN_FRAME_RATE`]..=[`MAX_FRAME_RATE`].
    pub frame_rate: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            command_capacity: 8,
            event_capacity: 64,
            reconnect: ReconnectPolicy::default(),
            max_identify_duration: Duration::from_secs(30),
            frame_rate: 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SessionError {
    #[error("the session is busy with earlier commands")]
    Busy,
    #[error("the session has stopped")]
    Stopped,
    #[error("an identify of {device} is still running")]
    IdentifyInProgress { device: DeviceRef },
    #[error("identify duration must be above zero and at most {max_ms} ms")]
    InvalidDuration { max_ms: u64 },
    /// The user said another program controls this device; nothing is sent until they hand it back.
    #[error("{device} is controlled by {owner:?}")]
    OutputConflict {
        device: DeviceRef,
        owner: OutputOwner,
    },
    #[error(transparent)]
    Backend(#[from] BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum OutputError {
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    InvalidEffect(#[from] InstanceError),
    #[error("no effect runs on {target:?}")]
    NoEffect { target: OutputTarget },
    #[error("a batch holds at most {limit} targets")]
    TooManyTargets { limit: usize },
}

impl From<BackendError> for OutputError {
    fn from(error: BackendError) -> Self {
        Self::Session(error.into())
    }
}

type Reply<T, E = SessionError> = oneshot::Sender<Result<T, E>>;

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
    Output(OutputCommand),
    Intents {
        after: u64,
        reply: Reply<IntentLog>,
    },
    EventProfiles(Reply<Option<EventProfiles>>),
    Resync(Reply<()>),
}

pub(crate) enum OutputCommand {
    Apply {
        assignment: Assignment,
        origin: Origin,
        reply: Reply<(), OutputError>,
    },
    Stop {
        target: OutputTarget,
        reply: Reply<Option<RestoreOutcome>, OutputError>,
    },
    ApplyBatch {
        targets: Vec<OutputTarget>,
        effect: EffectInstance,
        reply: Reply<BatchReport, OutputError>,
    },
    ApplyAcross {
        plan: AcrossPlan,
        effect: EffectInstance,
        origin: Origin,
        reply: Reply<BatchReport, OutputError>,
    },
    /// `None` stops every effect PrismForge runs.
    StopBatch {
        targets: Option<Vec<OutputTarget>>,
        reply: Reply<BatchReport, OutputError>,
    },
    SelectMode {
        device: DeviceRef,
        mode: usize,
        origin: Origin,
        reply: Reply<(), OutputError>,
    },
    DeclareExternal {
        device: DeviceRef,
        label: String,
        origin: Origin,
        reply: Reply<(), OutputError>,
    },
    ClearExternal {
        device: DeviceRef,
        reply: Reply<(), OutputError>,
    },
    SetFrameRate {
        frame_rate: u32,
        origin: Origin,
        reply: Reply<u32, OutputError>,
    },
    LightsOff {
        plan: LightsOffPlan,
        reply: Reply<LightsOffReport, OutputError>,
    },
    KeepLights {
        reply: Reply<(), OutputError>,
    },
}

/// Cheap, cloneable access to a running [`Session`]; commands are queued or refused at the call.
#[derive(Clone)]
pub struct SessionHandle {
    commands: mpsc::Sender<Command>,
    snapshot: Arc<watch::Sender<SessionSnapshot>>,
    known: Arc<watch::Sender<Vec<KnownDevice>>>,
    refused_busy: Arc<AtomicU64>,
    live: LiveChannels,
    id: u64,
}

impl SessionHandle {
    /// Unique per session in this process; clones of one handle share it.
    pub fn session_id(&self) -> u64 {
        self.id
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        self.snapshot.borrow().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<SessionSnapshot> {
        self.snapshot.subscribe()
    }

    /// The backend's known-device registry; it changes only after an inventory refresh.
    pub fn known_devices(&self) -> watch::Receiver<Vec<KnownDevice>> {
        self.known.subscribe()
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

    /// The colours PrismForge last wrote to `targets`; reading them sends nothing and never changes the output cadence.
    pub fn live_output(&self, targets: Vec<OutputTarget>) -> LiveFeed {
        static FEEDS: AtomicU64 = AtomicU64::new(1);
        self.live
            .feed(FEEDS.fetch_add(1, Ordering::Relaxed), targets)
    }

    pub fn effect_definitions(&self) -> Vec<EffectDefinition> {
        prismforge_effects::definitions()
    }

    /// Runs `effect` on `target`, replacing any effect it overlaps; takes the device over on first use.
    pub fn apply_effect(
        &self,
        target: OutputTarget,
        effect: EffectInstance,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.apply_effect_as(Origin::User, target, effect)
    }

    pub(crate) fn apply_effect_as(
        &self,
        origin: Origin,
        target: OutputTarget,
        effect: EffectInstance,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        let assignment = Assignment {
            target,
            effect,
            mode: ApplyMode::EachDevice,
        };
        self.send(move |reply| {
            Command::Output(OutputCommand::Apply {
                assignment,
                origin,
                reply,
            })
        })
    }

    /// Stops the effect on `target`; once a device has none left, restores what it showed before.
    pub fn stop_effect(
        &self,
        target: OutputTarget,
    ) -> impl Future<Output = Result<Option<RestoreOutcome>, OutputError>> + Send + use<> {
        self.send(move |reply| Command::Output(OutputCommand::Stop { target, reply }))
    }

    /// Runs `effect` on every target with one shared start time; each target gets its own result.
    pub fn apply_batch(
        &self,
        targets: Vec<OutputTarget>,
        effect: EffectInstance,
    ) -> impl Future<Output = Result<BatchReport, OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::ApplyBatch {
                targets,
                effect,
                reply,
            })
        })
    }

    /// Runs `effect` as one picture across the plan's targets, with the batch rules and one shared start time.
    pub fn apply_across(
        &self,
        plan: AcrossPlan,
        effect: EffectInstance,
    ) -> impl Future<Output = Result<BatchReport, OutputError>> + Send + use<> {
        self.apply_across_as(Origin::User, plan, effect)
    }

    pub(crate) fn apply_across_as(
        &self,
        origin: Origin,
        plan: AcrossPlan,
        effect: EffectInstance,
    ) -> impl Future<Output = Result<BatchReport, OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::ApplyAcross {
                plan,
                effect,
                origin,
                reply,
            })
        })
    }

    /// Stops the effect on every target, with the same rules as [`Self::stop_effect`] for each.
    pub fn stop_batch(
        &self,
        targets: Vec<OutputTarget>,
    ) -> impl Future<Output = Result<BatchReport, OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::StopBatch {
                targets: Some(targets),
                reply,
            })
        })
    }

    /// Stops every PrismForge effect, one whole-device result per device it ran on.
    pub fn stop_all_effects(
        &self,
    ) -> impl Future<Output = Result<BatchReport, OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::StopBatch {
                targets: None,
                reply,
            })
        })
    }

    /// Re-applies an existing hardware mode with its reported settings and stops any effect on the device.
    pub fn select_mode(
        &self,
        device: DeviceRef,
        mode: usize,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.select_mode_as(Origin::User, device, mode)
    }

    pub(crate) fn select_mode_as(
        &self,
        origin: Origin,
        device: DeviceRef,
        mode: usize,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::SelectMode {
                device,
                mode,
                origin,
                reply,
            })
        })
    }

    /// Records the user's statement that another program controls `device`; PrismForge stops writing to it.
    pub fn declare_external(
        &self,
        device: DeviceRef,
        label: String,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.declare_external_as(Origin::User, device, label)
    }

    pub(crate) fn declare_external_as(
        &self,
        origin: Origin,
        device: DeviceRef,
        label: String,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::DeclareExternal {
                device,
                label,
                origin,
                reply,
            })
        })
    }

    /// Records that the user handed `device` back to PrismForge.
    pub fn clear_external(
        &self,
        device: DeviceRef,
    ) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.send(move |reply| Command::Output(OutputCommand::ClearExternal { device, reply }))
    }

    /// Returns the rate in use after clamping.
    pub fn set_frame_rate(
        &self,
        frame_rate: u32,
    ) -> impl Future<Output = Result<u32, OutputError>> + Send + use<> {
        self.set_frame_rate_as(Origin::User, frame_rate)
    }

    pub(crate) fn set_frame_rate_as(
        &self,
        origin: Origin,
        frame_rate: u32,
    ) -> impl Future<Output = Result<u32, OutputError>> + Send + use<> {
        self.send(move |reply| {
            Command::Output(OutputCommand::SetFrameRate {
                frame_rate,
                origin,
                reply,
            })
        })
    }

    /// Writes black or the safe "Off" mode on each device of `plan`; effects end without a restore.
    pub fn lights_off(
        &self,
        plan: LightsOffPlan,
    ) -> impl Future<Output = Result<LightsOffReport, OutputError>> + Send + use<> {
        self.send(move |reply| Command::Output(OutputCommand::LightsOff { plan, reply }))
    }

    /// Writes each effect's due frame once, then ends every effect where it is, so nothing is restored.
    pub fn keep_lights(&self) -> impl Future<Output = Result<(), OutputError>> + Send + use<> {
        self.send(move |reply| Command::Output(OutputCommand::KeepLights { reply }))
    }

    /// Treats the state of every device as unknown, so the next frames set Direct Mode and all colours again.
    pub fn resync(&self) -> impl Future<Output = Result<(), SessionError>> + Send + use<> {
        self.send(Command::Resync)
    }

    /// Reads OpenRGB's own event profiles; `None` when the server cannot tell.
    pub fn event_profiles(
        &self,
    ) -> impl Future<Output = Result<Option<EventProfiles>, SessionError>> + Send + use<> {
        self.send(Command::EventProfiles)
    }

    /// The user's intent records after `after`, answered by the session loop in command order.
    pub(crate) fn intents_after(
        &self,
        after: u64,
    ) -> impl Future<Output = Result<IntentLog, SessionError>> + Send + use<> {
        self.send(move |reply| Command::Intents { after, reply })
    }

    fn send<T, E, F>(&self, command: F) -> impl Future<Output = Result<T, E>> + Send + use<T, E, F>
    where
        T: Send + 'static,
        E: From<SessionError> + Send + 'static,
        F: FnOnce(Reply<T, E>) -> Command,
    {
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
        async move {
            queued
                .map_err(E::from)?
                .await
                .unwrap_or_else(|_| Err(SessionError::Stopped.into()))
        }
    }
}

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub fn new(backend: B, config: SessionConfig) -> (SessionHandle, Self) {
        let (commands, receiver) = mpsc::channel(config.command_capacity.max(1));
        let refused_busy = Arc::new(AtomicU64::new(0));
        static SESSIONS: AtomicU64 = AtomicU64::new(0);
        let live = LiveChannels::new();
        let session = Self::start(
            backend,
            config,
            receiver,
            refused_busy.clone(),
            live.clone(),
        );
        let handle = SessionHandle {
            commands,
            snapshot: session.publisher(),
            known: session.registry(),
            refused_busy,
            live,
            id: SESSIONS.fetch_add(1, Ordering::Relaxed),
        };
        (handle, session)
    }
}
