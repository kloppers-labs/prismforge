use std::collections::{BTreeMap, VecDeque};
use std::time::Duration;

use prismforge_effects::{Effect, Rgb};
use tokio::time::Instant;

use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::inventory::Inventory;
use crate::ownership::OutputOwner;
use crate::scene::{Assignment, OutputTarget, Scene};
use crate::session::previous::PreviousState;

pub const MIN_FRAME_RATE: u32 = 1;
pub const MAX_FRAME_RATE: u32 = 60;
const FIRST_RETRY: Duration = Duration::from_secs(1);
const LAST_RETRY: Duration = Duration::from_secs(16);

/// Output ownership and effect state of one session; holds no backend.
pub(crate) struct Output {
    pub frame_rate: u32,
    pub next_tick: Instant,
    pub claims: BTreeMap<DeviceRef, Claim>,
    /// States of devices PrismForge changed and has not restored; dropped once the device reports another mode.
    pub remembered: BTreeMap<DeviceRef, PreviousState>,
    /// A tick that stopped between two backend calls to let a command through.
    pub tick: Option<Tick>,
    /// Set when a command ran while output was due, so output goes next.
    pub frame_owed: bool,
}

pub(crate) struct Tick {
    pub fired: Instant,
    pub on_cadence: bool,
    pub steps: VecDeque<Step>,
}

/// One backend call of a tick.
pub(crate) enum Step {
    DirectMode(DeviceRef),
    Frame(OutputTarget),
}

impl Step {
    pub fn device(&self) -> &DeviceRef {
        match self {
            Self::DirectMode(device) => device,
            Self::Frame(target) => &target.device,
        }
    }
}

pub(crate) enum Claim {
    Effect(EffectOutput),
    HardwareMode { mode: usize },
    External { label: String },
}

pub(crate) struct EffectOutput {
    pub previous: PreviousState,
    /// Per LED, what PrismForge last wrote, or what the inventory reported at take-over.
    pub shown: Vec<Rgb>,
    pub effects: Vec<Running>,
    pub direct_mode: bool,
    pub refusal: Option<Refusal>,
}

pub(crate) struct Running {
    pub target: OutputTarget,
    pub effect: Effect,
    pub started: Instant,
    pub written: bool,
}

pub(crate) struct Refusal {
    pub error: BackendError,
    pub attempts: u64,
    pub retry_at: Instant,
}

impl Output {
    pub fn new(frame_rate: u32) -> Self {
        let mut output = Self {
            frame_rate: MIN_FRAME_RATE,
            next_tick: Instant::now(),
            claims: BTreeMap::new(),
            remembered: BTreeMap::new(),
            tick: None,
            frame_owed: false,
        };
        output.set_frame_rate(frame_rate);
        output
    }

    pub fn set_frame_rate(&mut self, frame_rate: u32) -> u32 {
        self.frame_rate = frame_rate.clamp(MIN_FRAME_RATE, MAX_FRAME_RATE);
        self.frame_rate
    }

    pub fn effect(&self, device: &DeviceRef) -> Option<&EffectOutput> {
        match self.claims.get(device) {
            Some(Claim::Effect(output)) => Some(output),
            _ => None,
        }
    }

    pub fn effect_mut(&mut self, device: &DeviceRef) -> Option<&mut EffectOutput> {
        match self.claims.get_mut(device) {
            Some(Claim::Effect(output)) => Some(output),
            _ => None,
        }
    }

    pub fn effects(&self) -> impl Iterator<Item = (&DeviceRef, &EffectOutput)> {
        self.claims
            .iter()
            .filter_map(|(device, claim)| match claim {
                Claim::Effect(output) => Some((device, output)),
                _ => None,
            })
    }

    pub fn take_effect(&mut self, device: &DeviceRef) -> Option<EffectOutput> {
        match self.claims.remove(device)? {
            Claim::Effect(output) => Some(output),
            other => {
                self.claims.insert(device.clone(), other);
                None
            }
        }
    }

    pub fn scene(&self) -> Scene {
        Scene {
            assignments: self
                .effects()
                .flat_map(|(_, output)| &output.effects)
                .map(|running| Assignment {
                    target: running.target.clone(),
                    effect: running.effect.instance().clone(),
                })
                .collect(),
        }
    }

    /// Every effect is written again; direct mode counts as enabled only where the inventory still reports it.
    pub fn after_inventory(&mut self, inventory: &Inventory) {
        self.remembered.retain(|device, _| {
            inventory.device(device).is_none_or(|entry| {
                let capabilities = &entry.descriptor.capabilities;
                capabilities.active_mode.is_none() || capabilities.in_direct_mode()
            })
        });
        for (device, claim) in &mut self.claims {
            let Claim::Effect(output) = claim else {
                continue;
            };
            output.direct_mode &= inventory
                .device(device)
                .is_some_and(|entry| entry.descriptor.capabilities.in_direct_mode());
            output.touch();
        }
    }

    pub fn after_reconnect(&mut self) {
        self.tick = None;
        for claim in self.claims.values_mut() {
            if let Claim::Effect(output) = claim {
                output.direct_mode = false;
            }
        }
    }

    pub fn touch(&mut self, device: &DeviceRef) {
        if let Some(output) = self.effect_mut(device) {
            output.touch();
        }
    }

    /// Schedules the tick after one that fired at `fired` and finished at `done`; returns the ticks skipped.
    pub fn advance(&mut self, fired: Instant, done: Instant) -> u64 {
        let period = Duration::from_secs(1) / self.frame_rate;
        let on_grid = fired.saturating_duration_since(self.next_tick) < period;
        let next = if on_grid { self.next_tick } else { fired } + period;
        if done <= next {
            self.next_tick = next;
            return 0;
        }
        let skipped = (done - next).as_nanos().div_ceil(period.as_nanos());
        let skipped_time = period.as_nanos().saturating_mul(skipped);
        self.next_tick =
            next + Duration::from_nanos(u64::try_from(skipped_time).unwrap_or(u64::MAX));
        u64::try_from(skipped).unwrap_or(u64::MAX)
    }
}

impl Running {
    pub fn needs_frame(&self) -> bool {
        !self.written || !self.effect.is_static()
    }
}

impl Claim {
    pub fn owner(&self) -> OutputOwner {
        match self {
            Self::Effect(_) => OutputOwner::Effect,
            Self::HardwareMode { mode } => OutputOwner::HardwareMode { mode: *mode },
            Self::External { label } => OutputOwner::External {
                label: label.clone(),
            },
        }
    }
}

impl EffectOutput {
    pub fn new(previous: PreviousState, shown: Vec<Rgb>, first: Running) -> Self {
        Self {
            previous,
            shown,
            effects: vec![first],
            direct_mode: true,
            refusal: None,
        }
    }

    /// Replaces what `running` overlaps; the same effect on the same target keeps its start time.
    pub fn assign(&mut self, mut running: Running) {
        if let Some(current) = self.effects.iter().find(|current| {
            current.target == running.target
                && current.effect.instance().effect == running.effect.instance().effect
        }) {
            running.started = current.started;
        }
        self.effects
            .retain(|current| !current.target.overlaps(&running.target));
        self.effects.push(running);
        self.refusal = None;
    }

    /// Removes the effect on `target`; a whole-device target removes every effect on the device.
    pub fn remove(&mut self, target: &OutputTarget) -> bool {
        let before = self.effects.len();
        self.effects
            .retain(|running| target.zone.is_some() && running.target != *target);
        let removed = self.effects.len() < before;
        if removed {
            self.refusal = None;
        }
        removed
    }

    /// The backend calls this device needs now; empty exactly when it has nothing to write.
    pub fn steps(&self, device: &DeviceRef) -> Vec<Step> {
        let direct_mode = (!self.direct_mode).then(|| Step::DirectMode(device.clone()));
        direct_mode
            .into_iter()
            .chain(
                self.effects
                    .iter()
                    .filter(|running| running.needs_frame())
                    .map(|running| Step::Frame(running.target.clone())),
            )
            .collect()
    }

    pub fn has_work(&self) -> bool {
        !self.direct_mode || self.effects.iter().any(Running::needs_frame)
    }

    pub fn show(&mut self, start: usize, colors: &[Rgb], led_count: usize) {
        self.shown.resize(led_count, Rgb::default());
        if let Some(leds) = self.shown.get_mut(start..start + colors.len()) {
            leds.copy_from_slice(colors);
        }
    }

    pub fn refuse(&mut self, error: BackendError, now: Instant) {
        let attempts = self.refusal.as_ref().map_or(0, |refusal| refusal.attempts) + 1;
        let doublings = u32::try_from(attempts - 1).unwrap_or(u32::MAX).min(8);
        self.refusal = Some(Refusal {
            error,
            attempts,
            retry_at: now + (FIRST_RETRY * (1 << doublings)).min(LAST_RETRY),
        });
    }

    fn touch(&mut self) {
        for running in &mut self.effects {
            running.written = false;
        }
    }
}
