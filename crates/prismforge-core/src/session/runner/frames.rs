use prismforge_effects::linear_positions;
use tokio::time::Instant;

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::session::diagnostics::Operation;
use crate::session::output::{EffectOutput, Tick};
use crate::session::runner::{Link, Session};
use crate::session::snapshot::PauseReason;

const SEED: u64 = 0;

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// When the next frame or retry is due; `None` while nothing can be written, so the loop never spins.
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
        match &output.refusal {
            Some(refusal) => Some(refusal.retry_at),
            None => output.pending().then_some(self.output.next_tick),
        }
    }

    /// Writes the frame for now to each due device; waiting commands go between two devices, and passed ticks are skipped.
    pub(super) async fn on_frame(&mut self) {
        self.output.frame_owed = false;
        let mut tick = match self.output.tick.take() {
            Some(tick) => tick,
            None => self.start_tick(),
        };
        while let Some(device) = tick.remaining.pop_front() {
            let still_due = self.output.effect(&device).is_some_and(|output| {
                self.device_due(&device, output)
                    .is_some_and(|due| due <= Instant::now())
            });
            if still_due && !self.write_device(&device).await {
                return;
            }
            if !tick.remaining.is_empty() && !self.commands.is_empty() {
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
        Tick {
            fired,
            on_cadence: self.output.next_tick <= fired,
            remaining: self
                .output
                .effects()
                .filter(|(device, output)| {
                    self.device_due(device, output)
                        .is_some_and(|due| due <= fired)
                })
                .map(|(device, _)| device.clone())
                .collect(),
        }
    }

    /// One batched write per assigned target, rendered for the moment it starts; false once the connection is gone.
    async fn write_device(&mut self, device: &DeviceRef) -> bool {
        let now = Instant::now();
        let (Some(inventory), Some(output)) = (&self.inventory, self.output.effect(device)) else {
            return true;
        };
        let mut frames = Vec::new();
        let mut led_count = output.shown.len();
        for (index, running) in output.effects.iter().enumerate() {
            if running.written && running.effect.is_static() {
                continue;
            }
            let Ok(target) = inventory.output_target(device, running.target.zone) else {
                return true;
            };
            let descriptor = &target.descriptor;
            let leds = running.target.zone.map_or(descriptor.led_count, |zone| {
                descriptor.zones[zone].led_count
            });
            let start = running.target.zone.map_or(0, |zone| {
                descriptor.zones[..zone]
                    .iter()
                    .map(|zone| zone.led_count)
                    .sum()
            });
            let time = now.saturating_duration_since(running.started);
            let colors = running.effect.render(time, &linear_positions(leds), SEED);
            frames.push((index, running.target.zone, start, colors));
            led_count = descriptor.led_count;
        }
        if !output.direct_mode {
            if let Err(error) = self.backend.enable_direct_mode(device).await {
                return self.refused(device, error);
            }
            if let Some(output) = self.output.effect_mut(device) {
                output.direct_mode = true;
            }
        }
        for (index, zone, start, colors) in frames {
            let written = match zone {
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
        }
        if let Some(output) = self.output.effect_mut(device) {
            output.refusal = None;
        }
        true
    }

    /// A refusal waits for its own retry time; a stale inventory waits for the next refresh instead.
    fn refused(&mut self, device: &DeviceRef, error: BackendError) -> bool {
        let lost = self.lost_reason();
        self.record_error(Operation::Output, &error, lost.clone());
        if lost.is_some() {
            self.counters.output_writes_failed += 1;
            self.lose_if_disconnected();
            return false;
        }
        self.counters.output_writes_refused += 1;
        if error != BackendError::StaleInventory
            && let Some(output) = self.output.effect_mut(device)
        {
            output.refuse(error, Instant::now());
        }
        true
    }
}
