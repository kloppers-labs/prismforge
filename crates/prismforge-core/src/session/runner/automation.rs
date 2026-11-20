use std::time::SystemTime;

use prismforge_effects::Rgb;
use tokio::time::Instant;

use crate::automation::{
    DeviceLightsOff, EventProfiles, LightsOffOutcome, LightsOffPlan, LightsOffReport,
    LightsOffSkip, LightsOffStep, LightsOffWrite,
};
use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::session::diagnostics::Operation;
use crate::session::output::Claim;
use crate::session::runner::Session;

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(super) async fn read_event_profiles(
        &mut self,
    ) -> Result<Option<EventProfiles>, BackendError> {
        let read = self.backend.event_profiles().await;
        self.lose_if_disconnected();
        read
    }

    /// The next tick enables Direct Mode again and writes every colour, whatever the inventory says.
    pub(super) fn forget_device_state(&mut self) {
        self.output.after_reconnect();
        let devices: Vec<_> = self
            .output
            .effects()
            .map(|(device, _)| device.clone())
            .collect();
        for device in &devices {
            self.output.touch(device);
        }
    }

    pub(super) async fn lights_off(
        &mut self,
        plan: LightsOffPlan,
        not_after: Option<SystemTime>,
    ) -> LightsOffReport {
        let mut devices = plan.skipped;
        for step in plan.steps {
            let late = not_after.is_some_and(|limit| SystemTime::now() > limit);
            let outcome = if late {
                skipped(LightsOffSkip::TooLate)
            } else {
                self.switch_off(&step).await
            };
            devices.push(DeviceLightsOff {
                device: Some(step.device),
                name: step.name,
                outcome,
            });
        }
        LightsOffReport { devices }
    }

    async fn switch_off(&mut self, step: &LightsOffStep) -> LightsOffOutcome {
        let device = &step.device;
        if let Some(Claim::External { label }) = self.output.claims.get(device) {
            return skipped(LightsOffSkip::DeclaredExternal {
                label: label.clone(),
            });
        }
        if self
            .identify
            .as_ref()
            .is_some_and(|active| &active.plan.device == device)
        {
            return skipped(LightsOffSkip::Identifying);
        }
        let leds = match self.live_inventory().and_then(|inventory| {
            let index = inventory.locate(device)?;
            Ok(inventory.devices[index].descriptor.led_count)
        }) {
            Ok(leds) => leds,
            Err(error) => return failed(error),
        };
        if let Some(output) = self.output.take_effect(device) {
            self.output
                .remembered
                .entry(device.clone())
                .or_insert(output.previous);
        }
        let written = match &step.write {
            LightsOffWrite::Black { .. } => self.write_black(device, leds).await,
            LightsOffWrite::OffMode { mode, .. } => self.backend.reapply_mode(device, *mode).await,
        };
        if let Err(error) = written {
            let lost = self.lost_reason();
            self.record_error(Operation::Output, &error, lost);
            self.lose_if_disconnected();
            return failed(error);
        }
        match &step.write {
            LightsOffWrite::Black { .. } => {
                self.output.claims.remove(device);
                LightsOffOutcome::Black
            }
            LightsOffWrite::OffMode { mode, name } => {
                self.output
                    .claims
                    .insert(device.clone(), Claim::HardwareMode { mode: *mode });
                self.output.remembered.remove(device);
                LightsOffOutcome::OffMode {
                    mode: *mode,
                    name: name.clone(),
                }
            }
        }
    }

    async fn write_black(&mut self, device: &DeviceRef, leds: usize) -> Result<(), BackendError> {
        self.backend.enable_direct_mode(device).await?;
        self.backend
            .write_device_frame(device, &vec![Rgb::new(0, 0, 0); leds])
            .await
    }

    /// Effects end where they are: the frames due now go out once, and no device is restored.
    pub(super) async fn keep_lights(&mut self) {
        if self.output.effects().next().is_none() {
            return;
        }
        self.output.next_tick = Instant::now();
        self.on_frame().await;
        let devices: Vec<_> = self
            .output
            .effects()
            .map(|(device, _)| device.clone())
            .collect();
        for device in devices {
            self.output.take_effect(&device);
        }
    }
}

fn skipped(reason: LightsOffSkip) -> LightsOffOutcome {
    LightsOffOutcome::Skipped { reason }
}

fn failed(error: BackendError) -> LightsOffOutcome {
    LightsOffOutcome::Failed {
        error: error.into(),
    }
}
