use std::sync::Arc;
use std::time::Duration;

use prismforge_effects::{Effect, EffectInstance};
use tokio::time::Instant;

use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::ownership::OutputOwner;
use crate::scene::{ApplyMode, Assignment, OutputTarget};
use crate::session::batch::{BatchReport, TargetOutcome};
use crate::session::diagnostics::Operation;
use crate::session::identify::RestoreOutcome;
use crate::session::output::{Across, Claim, EffectOutput, Layout, Running};
use crate::session::previous::PreviousState;
use crate::session::runner::intents::Held;
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
                let before = self.held_for(origin);
                let target = assignment.target.clone();
                let result = self.apply(assignment).await;
                if let Ok(instance) = &result {
                    let own = (
                        Slot::Effect(target),
                        Intent::Applied {
                            effect: instance.clone(),
                            mode: ApplyMode::EachDevice,
                        },
                    );
                    self.record_user(before, vec![own]);
                }
                self.answer(reply, result.map(|_| ()));
            }
            OutputCommand::ApplyBatch {
                targets,
                effect,
                reply,
            } => {
                let before = self.held_for(Origin::User);
                let targets = targets
                    .into_iter()
                    .map(|target| (target, Layout::Strip))
                    .collect();
                let result = self.apply_batch(targets, &effect).await;
                if let Ok((report, instance)) = &result {
                    self.record_applied(before, report, instance, ApplyMode::EachDevice);
                }
                self.answer(reply, result.map(|(report, _)| report));
            }
            OutputCommand::ApplyAcross {
                plan,
                effect,
                origin,
                reply,
            } => {
                let before = self.held_for(origin);
                let (view, flow, plane) = (plan.view().clone(), plan.flow(), plan.plane());
                let mode = ApplyMode::AcrossSetup {
                    view: view.clone(),
                    flow,
                };
                let targets = plan
                    .targets
                    .into_iter()
                    .map(|placed| {
                        let across = Across {
                            view: view.clone(),
                            flow,
                            plane,
                            points: placed.points,
                        };
                        (placed.target, Layout::Across(Arc::new(across)))
                    })
                    .collect();
                let result = self.apply_batch(targets, &effect).await;
                if let Ok((report, instance)) = &result {
                    self.record_applied(before, report, instance, mode);
                }
                self.answer(reply, result.map(|(report, _)| report));
            }
            OutputCommand::StopBatch { targets, reply } => {
                let before = self.held_for(Origin::User);
                let result = self.stop_batch(targets).await;
                if let Ok(report) = &result {
                    let own = report
                        .results
                        .iter()
                        .filter(|result| matches!(result.outcome, TargetOutcome::Stopped { .. }))
                        .map(|result| (Slot::Effect(result.target.clone()), Intent::Stopped))
                        .collect();
                    self.record_user(before, own);
                }
                self.answer(reply, result);
            }
            OutputCommand::Stop { target, reply } => {
                let before = self.held_for(Origin::User);
                let result = self.stop_effect(target.clone()).await;
                if result.is_ok() {
                    self.record_user(before, vec![(Slot::Effect(target), Intent::Stopped)]);
                }
                self.answer(reply, result);
            }
            OutputCommand::SelectMode {
                device,
                mode,
                origin,
                reply,
            } => {
                let before = self.held_for(origin);
                let result = self.select_mode(device.clone(), mode).await;
                if result.is_ok() {
                    let name = self.mode_name(&device, mode);
                    let own = (
                        Slot::Mode(device),
                        Intent::ModeSelected { index: mode, name },
                    );
                    self.record_user(before, vec![own]);
                }
                self.answer(reply, result);
            }
            OutputCommand::DeclareExternal {
                device,
                label,
                origin,
                reply,
            } => {
                let before = self.held_for(origin);
                let result = self.declare_external(device.clone(), label.clone());
                if result.is_ok() {
                    self.record_user(
                        before,
                        vec![(Slot::External(device), Intent::Declared { label })],
                    );
                }
                self.answer(reply, result);
            }
            OutputCommand::ClearExternal { device, reply } => {
                if matches!(
                    self.output.claims.get(&device),
                    Some(Claim::External { .. })
                ) {
                    let before = self.held_for(Origin::User);
                    self.output.claims.remove(&device);
                    self.record_user(before, vec![(Slot::External(device), Intent::HandedBack)]);
                }
                self.answer(reply, Ok(()));
            }
            OutputCommand::LightsOff { plan, reply } => {
                let report = self.lights_off(plan).await;
                self.answer(reply, Ok(report));
            }
            OutputCommand::KeepLights { reply } => {
                self.keep_lights().await;
                self.answer(reply, Ok(()));
            }
            OutputCommand::SetFrameRate {
                frame_rate,
                origin,
                reply,
            } => {
                let before = self.held_for(origin);
                let in_use = self.output.set_frame_rate(frame_rate);
                self.record_user(before, vec![(Slot::FrameRate, Intent::FrameRate(in_use))]);
                self.answer(reply, Ok(in_use));
            }
        }
    }

    /// Records the batch's applied targets as the user's own, each with `mode`.
    fn record_applied(
        &mut self,
        before: Option<Vec<(Slot, Held)>>,
        report: &BatchReport,
        instance: &EffectInstance,
        mode: ApplyMode,
    ) {
        let own = report
            .results
            .iter()
            .filter(|result| result.outcome == TargetOutcome::Applied)
            .map(|result| {
                let applied = Intent::Applied {
                    effect: instance.clone(),
                    mode: mode.clone(),
                };
                (Slot::Effect(result.target.clone()), applied)
            })
            .collect();
        self.record_user(before, own);
    }

    /// The same effect on the same target keeps its start time; anything else starts now.
    async fn apply(&mut self, assignment: Assignment) -> Result<EffectInstance, OutputError> {
        let effect = prismforge_effects::prepare(&assignment.effect)?;
        let complete = effect.instance().clone();
        let started = self.kept_start(&assignment.target, &complete.effect);
        self.run_effect(assignment.target, effect, Layout::Strip, started)
            .await?;
        Ok(complete)
    }

    /// When `target` started running `effect` if it runs that effect now.
    pub(super) fn kept_start(&self, target: &OutputTarget, effect: &str) -> Option<Instant> {
        self.output
            .effect(&target.device)?
            .effects
            .iter()
            .find(|running| running.target == *target && running.effect.instance().effect == effect)
            .map(|running| running.started)
    }

    /// Takes the device over on first use, then starts the effect at `started` or, without one, now; returns the start used.
    pub(super) async fn run_effect(
        &mut self,
        target: OutputTarget,
        effect: Effect,
        layout: Layout,
        started: Option<Instant>,
    ) -> Result<Instant, OutputError> {
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
            let started = started.unwrap_or_else(Instant::now);
            let output =
                EffectOutput::new(previous, shown, running(target, effect, layout, started));
            self.output.remembered.remove(&device);
            self.output.claims.insert(device, Claim::Effect(output));
            return Ok(started);
        }
        let started = started.unwrap_or_else(Instant::now);
        if let Some(output) = self.output.effect_mut(&device) {
            output.assign(running(target, effect, layout, started));
        }
        Ok(started)
    }

    pub(super) async fn stop_effect(
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

fn running(target: OutputTarget, effect: Effect, layout: Layout, started: Instant) -> Running {
    Running {
        target,
        effect,
        layout,
        started,
        written: false,
    }
}
