use std::collections::VecDeque;

use tokio::time::Instant;

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::scene::OutputTarget;
use crate::session::diagnostics::Operation;
use crate::session::output::{EffectOutput, Step, Tick};
use crate::session::runner::{Link, Session};
use crate::session::snapshot::PauseReason;

enum Flow {
    Next,
    SkipDevice,
    Lost,
}

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// When output is next due; a time not in the future always means at least one backend call to make.
    pub(super) fn output_due(&self) -> Option<Instant> {
        if self.output_pause().is_some() {
            return None;
        }
        if let Some(tick) = &self.output.tick {
            return Some(tick.fired);
        }
        self.output
            .effects()
            .filter_map(|(device, output)| self.device_due(device, output))
            .min()
    }

    pub(super) fn output_pause(&self) -> Option<PauseReason> {
        if !matches!(self.link, Link::Connected) || self.inventory.is_none() {
            return Some(PauseReason::NotConnected);
        }
        self.backend_stale().then_some(PauseReason::InventoryStale)
    }

    pub(super) fn device_pause(
        &self,
        device: &DeviceRef,
        output: &EffectOutput,
    ) -> Option<PauseReason> {
        if self
            .identify
            .as_ref()
            .is_some_and(|active| &active.plan.device == device)
        {
            return Some(PauseReason::Identify);
        }
        let inventory = self.inventory.as_ref()?;
        output.effects.iter().find_map(|running| {
            let error = inventory.output_target(device, running.target.zone).err()?;
            Some(PauseReason::Unavailable { error })
        })
    }

    fn device_due(&self, device: &DeviceRef, output: &EffectOutput) -> Option<Instant> {
        if self.device_pause(device, output).is_some() {
            return None;
        }
        if !output.has_work() {
            return None;
        }
        match &output.refusal {
            Some(refusal) => Some(refusal.retry_at),
            None => Some(self.output.next_tick),
        }
    }

    /// Runs the due backend calls one at a time; a waiting command goes between two calls, and passed ticks are skipped.
    pub(super) async fn on_frame(&mut self) {
        self.counters.output_wakeups += 1;
        self.output.frame_owed = false;
        let mut tick = match self.output.tick.take() {
            Some(tick) => tick,
            None => self.start_tick(),
        };
        while let Some(step) = tick.steps.pop_front() {
            let device = step.device();
            match self.run_step(&step).await {
                Flow::Next if !tick.steps.iter().any(|later| later.device() == device) => {
                    if let Some(output) = self.output.effect_mut(device) {
                        output.refusal = None;
                    }
                }
                Flow::Next => {}
                Flow::SkipDevice => tick.steps.retain(|later| later.device() != device),
                Flow::Lost => return,
            }
            if !tick.steps.is_empty() && !self.commands.is_empty() {
                self.output.tick = Some(tick);
                return;
            }
        }
        if tick.on_cadence {
            self.counters.ticks_skipped += self.output.advance(tick.fired, Instant::now());
        }
    }

    fn start_tick(&self) -> Tick {
        let fired = Instant::now();
        let mut steps = VecDeque::new();
        for (device, output) in self.output.effects() {
            if !self
                .device_due(device, output)
                .is_some_and(|due| due <= fired)
            {
                continue;
            }
            steps.extend(output.steps(device));
        }
        Tick {
            fired,
            on_cadence: self.output.next_tick <= fired,
            steps,
        }
    }

    async fn run_step(&mut self, step: &Step) -> Flow {
        let device = step.device();
        let due = self.output.effect(device).is_some_and(|output| {
            self.device_due(device, output)
                .is_some_and(|due| due <= Instant::now())
        });
        if !due {
            return Flow::SkipDevice;
        }
        match step {
            Step::DirectMode(device) => self.enable_direct_mode(device).await,
            Step::Frame(target) => self.write_frame(target).await,
        }
    }

    async fn enable_direct_mode(&mut self, device: &DeviceRef) -> Flow {
        if self
            .output
            .effect(device)
            .is_some_and(|output| output.direct_mode)
        {
            return Flow::Next;
        }
        if let Err(error) = self.backend.enable_direct_mode(device).await {
            return self.refused(device, error);
        }
        if let Some(output) = self.output.effect_mut(device) {
            output.direct_mode = true;
        }
        Flow::Next
    }

    /// One batched write for one target, rendered for the moment it starts.
    async fn write_frame(&mut self, target: &OutputTarget) -> Flow {
        let device = &target.device;
        let now = Instant::now();
        let (Some(inventory), Some(output)) = (&self.inventory, self.output.effect(device)) else {
            return Flow::SkipDevice;
        };
        let Some(index) = output
            .effects
            .iter()
            .position(|running| running.target == *target)
        else {
            return Flow::Next;
        };
        let running = &output.effects[index];
        if !running.needs_frame() {
            return Flow::Next;
        }
        let Ok(entry) = inventory.output_target(device, target.zone) else {
            return Flow::SkipDevice;
        };
        if !output.direct_mode {
            return Flow::SkipDevice;
        }
        let descriptor = &entry.descriptor;
        let led_count = descriptor.led_count;
        let (start, leds) = target.zone.map_or((0, led_count), |zone| {
            let start = descriptor.zones[..zone]
                .iter()
                .map(|zone| zone.led_count)
                .sum();
            (start, descriptor.zones[zone].led_count)
        });
        let time = now.saturating_duration_since(running.started);
        let colors = running.effect.render_strip(time, leds);
        let written = match target.zone {
            Some(zone) => self.backend.write_zone_frame(device, zone, &colors).await,
            None => self.backend.write_device_frame(device, &colors).await,
        };
        if let Err(error) = written {
            return self.refused(device, error);
        }
        self.counters.frames_written += 1;
        if let Some(output) = self.output.effect_mut(device) {
            output.show(start, &colors, led_count);
            if let Some(running) = output.effects.get_mut(index) {
                running.written = true;
            }
        }
        Flow::Next
    }

    /// A refusal waits for its own retry time; a stale inventory waits for the next refresh instead.
    fn refused(&mut self, device: &DeviceRef, error: BackendError) -> Flow {
        let lost = self.lost_reason();
        self.record_error(Operation::Output, &error, lost.clone());
        if lost.is_some() {
            self.counters.output_writes_failed += 1;
            self.lose_if_disconnected();
            return Flow::Lost;
        }
        self.counters.output_writes_refused += 1;
        if error != BackendError::StaleInventory
            && let Some(output) = self.output.effect_mut(device)
        {
            output.refuse(error, Instant::now());
        }
        Flow::SkipDevice
    }
}
