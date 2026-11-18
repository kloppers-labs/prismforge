use std::time::Duration;

use tokio::time::Instant;

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::ownership::OutputOwner;
use crate::scene::{Assignment, OutputTarget};
use crate::session::diagnostics::Operation;
use crate::session::identify::RestoreOutcome;
use crate::session::output::{Claim, EffectOutput, Running};
use crate::session::previous::PreviousState;
use crate::session::runner::{Session, not_restored_after_loss};
use crate::session::{OutputCommand, OutputError, SessionError};

const RELEASE_LIMIT: Duration = Duration::from_secs(5);

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(super) async fn on_output(&mut self, command: OutputCommand) {
        match command {
            OutputCommand::Apply { assignment, reply } => {
                let _ = reply.send(self.apply(assignment).await);
            }
            OutputCommand::Stop { target, reply } => {
                let _ = reply.send(self.stop_effect(target).await);
            }
            OutputCommand::SelectMode {
                device,
                mode,
                reply,
            } => {
                let _ = reply.send(self.select_mode(device, mode).await);
            }
            OutputCommand::DeclareExternal {
                device,
                label,
                reply,
            } => {
                let _ = reply.send(self.declare_external(device, label));
            }
            OutputCommand::ClearExternal { device, reply } => {
                if matches!(
                    self.output.claims.get(&device),
                    Some(Claim::External { .. })
                ) {
                    self.output.claims.remove(&device);
                }
                let _ = reply.send(Ok(()));
            }
            OutputCommand::SetFrameRate { frame_rate, reply } => {
                let _ = reply.send(Ok(self.output.set_frame_rate(frame_rate)));
            }
        }
    }

    /// Takes the device over on first use: remembers its reported state, then enables direct mode.
    async fn apply(&mut self, assignment: Assignment) -> Result<(), OutputError> {
        let effect = prismforge_effects::prepare(&assignment.effect)?;
        let target = assignment.target;
        let device = target.device.clone();
        self.ensure_free(&device)?;
        let take_over = {
            let inventory = self.live_inventory()?;
            let descriptor = &inventory.output_target(&device, target.zone)?.descriptor;
            let suspect = self.state_suspect(&device, inventory);
            self.output.effect(&device).is_none().then(|| {
                let previous = self
                    .output
                    .remembered
                    .get(&device)
                    .filter(|previous| previous.still_matches(inventory).is_ok())
                    .cloned();
                let previous = previous
                    .unwrap_or_else(|| PreviousState::capture(&device, descriptor, suspect));
                (previous, descriptor.colors.clone())
            })
        };
        if let Some((previous, shown)) = take_over {
            if let Err(error) = self.backend.enable_direct_mode(&device).await {
                let lost = self.lost_reason();
                self.record_error(Operation::Output, &error, lost);
                self.lose_if_disconnected();
                return Err(error.into());
            }
            let running = running(target, effect);
            let output = EffectOutput::new(previous, shown, running);
            self.output.remembered.remove(&device);
            self.output.claims.insert(device, Claim::Effect(output));
        } else if let Some(output) = self.output.effect_mut(&device) {
            output.assign(running(target, effect));
        }
        Ok(())
    }

    async fn stop_effect(
        &mut self,
        target: OutputTarget,
    ) -> Result<Option<RestoreOutcome>, OutputError> {
        let device = target.device.clone();
        self.ensure_not_identifying(&device)?;
        let Some(output) = self.output.effect_mut(&device) else {
            return Err(OutputError::NoEffect { target });
        };
        if !output.remove(&target) {
            return Err(OutputError::NoEffect { target });
        }
        if !output.effects.is_empty() {
            return Ok(None);
        }
        let Some(output) = self.output.take_effect(&device) else {
            return Ok(None);
        };
        let restore = self.release(output).await;
        self.settle(&device, restore != RestoreOutcome::Unchanged)
            .await;
        Ok(Some(restore))
    }

    /// Sets the owner to the hardware mode; any effect on the device stops without a restore.
    async fn select_mode(&mut self, device: DeviceRef, mode: usize) -> Result<(), OutputError> {
        self.ensure_free(&device)?;
        self.live_inventory()?.locate(&device)?;
        if let Err(error) = self.backend.reapply_mode(&device, mode).await {
            let lost = self.lost_reason();
            self.record_error(Operation::SelectMode, &error, lost);
            self.lose_if_disconnected();
            return Err(error.into());
        }
        self.output
            .claims
            .insert(device.clone(), Claim::HardwareMode { mode });
        self.output.remembered.remove(&device);
        self.unrestored.retain(|unrestored| unrestored != &device);
        self.settle(&device, true).await;
        Ok(())
    }

    /// Only records the user's statement; an effect on the device stops without a restore that would fight the other writer.
    fn declare_external(&mut self, device: DeviceRef, label: String) -> Result<(), OutputError> {
        self.ensure_not_identifying(&device)?;
        if !self
            .seen
            .iter()
            .any(|seen| seen.identity.reference() == Some(&device))
        {
            return Err(BackendError::UnknownDevice { device }.into());
        }
        if let Some(output) = self.output.take_effect(&device) {
            self.keep_unrestored(output.previous);
        }
        self.output.claims.insert(device, Claim::External { label });
        Ok(())
    }

    /// Restores every device PrismForge runs effects on, within a fixed time, before a disconnect.
    pub(super) async fn release_all(&mut self) {
        let devices: Vec<DeviceRef> = self
            .output
            .effects()
            .map(|(device, _)| device.clone())
            .collect();
        let release = async {
            for device in &devices {
                if let Some(output) = self.output.take_effect(device) {
                    self.release(output).await;
                }
            }
        };
        // Cutting a restore short costs only the connection, which closes right after.
        let _ = tokio::time::timeout(RELEASE_LIMIT, release).await;
        for device in &devices {
            if let Some(output) = self.output.take_effect(device) {
                self.keep_unrestored(output.previous);
            } else if self.output.remembered.contains_key(device) {
                self.note_unrestored(device);
            }
        }
    }

    /// Keeps a state PrismForge did not restore, so the next take-over can still restore it.
    fn keep_unrestored(&mut self, previous: PreviousState) {
        self.note_unrestored(&previous.device);
        self.output
            .remembered
            .insert(previous.device.clone(), previous);
    }

    fn note_unrestored(&mut self, device: &DeviceRef) {
        if !self.unrestored.contains(device) {
            self.unrestored.push(device.clone());
        }
    }

    async fn release(&mut self, output: EffectOutput) -> RestoreOutcome {
        let previous = output.previous;
        self.output
            .remembered
            .insert(previous.device.clone(), previous.clone());
        let outcome = self.restore_previous(&previous).await;
        if outcome == RestoreOutcome::Restored {
            self.output.remembered.remove(&previous.device);
        }
        outcome
    }

    async fn restore_previous(&mut self, previous: &PreviousState) -> RestoreOutcome {
        if let Some(reason) = self.link_down() {
            let outcome = not_restored_after_loss(&reason);
            self.note_restore(&previous.device, &outcome);
            return outcome;
        }
        if let Some(Err(failure)) = self
            .backend
            .inventory()
            .map(|inventory| previous.still_matches(inventory))
        {
            let outcome = RestoreOutcome::NotRestored { failure };
            self.note_restore(&previous.device, &outcome);
            return outcome;
        }
        let outcome = self.restore(previous).await;
        self.note_restore(&previous.device, &outcome);
        outcome
    }

    fn ensure_free(&self, device: &DeviceRef) -> Result<(), SessionError> {
        self.ensure_not_identifying(device)?;
        self.ensure_not_external(device)
    }

    pub(super) fn ensure_not_external(&self, device: &DeviceRef) -> Result<(), SessionError> {
        match self.output.claims.get(device) {
            Some(Claim::External { label }) => Err(SessionError::OutputConflict {
                device: device.clone(),
                owner: OutputOwner::External {
                    label: label.clone(),
                },
            }),
            _ => Ok(()),
        }
    }

    fn ensure_not_identifying(&self, device: &DeviceRef) -> Result<(), SessionError> {
        match &self.identify {
            Some(active) if &active.plan.device == device => {
                Err(SessionError::IdentifyInProgress {
                    device: device.clone(),
                })
            }
            _ => Ok(()),
        }
    }
}

fn running(target: OutputTarget, effect: prismforge_effects::Effect) -> Running {
    Running {
        target,
        effect,
        started: Instant::now(),
        written: false,
    }
}
