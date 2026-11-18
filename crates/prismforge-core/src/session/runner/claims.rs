use std::time::Duration;

use prismforge_effects::EffectInstance;
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
use crate::session::{Intent, Origin, OutputCommand, OutputError, SessionError, Slot};

const RELEASE_LIMIT: Duration = Duration::from_secs(5);

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(super) async fn on_output(&mut self, command: OutputCommand) {
        match command {
            OutputCommand::Apply {
                assignment,
                origin,
                reply,
            } => {
                let target = assignment.target.clone();
                let mode_before = matches!(
                    self.output.claims.get(&target.device),
                    Some(Claim::HardwareMode { .. })
                );
                let result = self.apply(assignment).await;
                if let (Ok(instance), Origin::User) = (&result, origin) {
                    let mut entries = vec![(
                        Slot::Effect(target.clone()),
                        Intent::Applied(instance.clone()),
                    )];
                    if mode_before {
                        entries.push((Slot::Mode(target.device), Intent::ModeEnded));
                    }
                    self.intents.record(entries);
                }
                self.answer(reply, result.map(|_| ()));
            }
            OutputCommand::Stop { target, reply } => {
                let result = self.stop_effect(target.clone()).await;
                if result.is_ok() {
                    self.intents
                        .record(vec![(Slot::Effect(target), Intent::Stopped)]);
                }
                self.answer(reply, result);
            }
            OutputCommand::SelectMode {
                device,
                mode,
                origin,
                reply,
            } => {
                let name = self
                    .live_inventory()
                    .ok()
                    .and_then(|inventory| inventory.device(&device))
                    .and_then(|entry| entry.descriptor.capabilities.modes.get(mode))
                    .map(|descriptor| descriptor.name.clone())
                    .unwrap_or_default();
                let result = self.select_mode(device.clone(), mode).await;
                if result.is_ok() && origin == Origin::User {
                    self.intents.record(vec![
                        (whole_device(&device), Intent::Stopped),
                        (
                            Slot::Mode(device),
                            Intent::ModeSelected { index: mode, name },
                        ),
                    ]);
                }
                self.answer(reply, result);
            }
            OutputCommand::DeclareExternal {
                device,
                label,
                origin,
                reply,
            } => {
                let result = self.declare_external(device.clone(), label.clone());
                if result.is_ok() && origin == Origin::User {
                    self.intents.record(vec![
                        (whole_device(&device), Intent::Stopped),
                        (Slot::Mode(device.clone()), Intent::ModeEnded),
                        (Slot::External(device), Intent::Declared { label }),
                    ]);
                }
                self.answer(reply, result);
            }
            OutputCommand::ClearExternal { device, reply } => {
                if matches!(
                    self.output.claims.get(&device),
                    Some(Claim::External { .. })
                ) {
                    self.output.claims.remove(&device);
                    self.intents
                        .record(vec![(Slot::External(device), Intent::HandedBack)]);
                }
                self.answer(reply, Ok(()));
            }
            OutputCommand::SetFrameRate {
                frame_rate,
                origin,
                reply,
            } => {
                let in_use = self.output.set_frame_rate(frame_rate);
                if origin == Origin::User {
                    self.intents
                        .record(vec![(Slot::FrameRate, Intent::FrameRate(in_use))]);
                }
                self.answer(reply, Ok(in_use));
            }
        }
    }

    /// Takes the device over on first use: remembers its reported state, then enables direct mode.
    async fn apply(&mut self, assignment: Assignment) -> Result<EffectInstance, OutputError> {
        let effect = prismforge_effects::prepare(&assignment.effect)?;
        let complete = effect.instance().clone();
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
        Ok(complete)
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

fn whole_device(device: &DeviceRef) -> Slot {
    Slot::Effect(OutputTarget {
        device: device.clone(),
        zone: None,
    })
}
