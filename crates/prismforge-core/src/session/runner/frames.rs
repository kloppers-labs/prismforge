use prismforge_effects::linear_positions;
use tokio::time::Instant;

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::session::diagnostics::Operation;
use crate::session::output::EffectOutput;
use crate::session::runner::{Link, Session};
use crate::session::snapshot::PauseReason;

const SEED: u64 = 0;

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// When the next frame or retry is due; `None` while nothing can be written, so the loop never spins.
    pub(super) fn output_due(&self) -> Option<Instant> {
        if self.output_pause().is_some() {
            return None;
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

    /// Writes the frame for now to every device that is due; ticks that pass meanwhile are skipped, never queued.
    pub(super) async fn on_frame(&mut self) {
        let fired = Instant::now();
        let on_cadence = self.output.next_tick <= fired;
        let due: Vec<DeviceRef> = self
            .output
            .effects()
            .filter(|(device, output)| {
                self.device_due(device, output)
                    .is_some_and(|due| due <= fired)
            })
            .map(|(device, _)| device.clone())
            .collect();
        for device in due {
            if !self.write_device(&device, fired).await {
                break;
            }
        }
        if on_cadence {
            self.counters.ticks_skipped += self.output.advance(fired, Instant::now());
        }
    }

    /// One batched write per assigned target; returns false once the connection is gone.
    async fn write_device(&mut self, device: &DeviceRef, fired: Instant) -> bool {
        let (Some(inventory), Some(output)) = (&self.inventory, self.output.effect(device)) else {
            return true;
        };
        let mut frames = Vec::new();
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
            let time = fired.saturating_duration_since(running.started);
            let colors = running.effect.render(time, &linear_positions(leds), SEED);
            frames.push((index, running.target.zone, colors));
        }
        if !output.direct_mode {
            if let Err(error) = self.backend.enable_direct_mode(device).await {
                return self.refused(device, error);
            }
            if let Some(output) = self.output.effect_mut(device) {
                output.direct_mode = true;
            }
        }
        for (index, zone, colors) in frames {
            let written = match zone {
                Some(zone) => self.backend.write_zone_frame(device, zone, &colors).await,
                None => self.backend.write_device_frame(device, &colors).await,
            };
            if let Err(error) = written {
                return self.refused(device, error);
            }
            self.counters.frames_written += 1;
            if let Some(running) = self
                .output
                .effect_mut(device)
                .and_then(|output| output.effects.get_mut(index))
            {
                running.written = true;
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
