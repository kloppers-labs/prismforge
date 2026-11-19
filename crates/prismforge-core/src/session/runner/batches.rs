use prismforge_effects::EffectInstance;

use crate::backend::RgbBackend;
use crate::scene::OutputTarget;
use crate::session::OutputError;
use crate::session::batch::{
    BatchReport, MAX_BATCH_TARGETS, TargetOutcome, TargetResult, coverage,
};
use crate::session::runner::Session;

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// Targets that run this effect keep running and the others join at their earliest start; without one, the clock starts with the first target.
    pub(super) async fn apply_batch(
        &mut self,
        targets: Vec<OutputTarget>,
        instance: &EffectInstance,
    ) -> Result<(BatchReport, EffectInstance), OutputError> {
        bounded(&targets)?;
        let effect = prismforge_effects::prepare(instance)?;
        let complete = effect.instance().clone();
        let covered = coverage(&targets);
        let mut started = targets
            .iter()
            .zip(&covered)
            .filter(|(_, cover)| cover.is_none())
            .filter_map(|(target, _)| self.kept_start(target, &complete.effect))
            .min();
        let mut results = Vec::with_capacity(targets.len());
        for (target, cover) in targets.into_iter().zip(covered) {
            let outcome = match cover {
                Some(by) => TargetOutcome::Covered { by },
                None => match self
                    .run_effect(target.clone(), effect.clone(), started)
                    .await
                {
                    Ok(start) => {
                        started = Some(start);
                        TargetOutcome::Applied
                    }
                    Err(error) => TargetOutcome::Skipped { error },
                },
            };
            results.push(TargetResult { target, outcome });
        }
        Ok((BatchReport { results }, complete))
    }

    /// `None` stops every device PrismForge runs effects on, as whole devices.
    pub(super) async fn stop_batch(
        &mut self,
        targets: Option<Vec<OutputTarget>>,
    ) -> Result<BatchReport, OutputError> {
        let targets = match targets {
            Some(targets) => targets,
            None => self
                .output
                .effects()
                .map(|(device, _)| OutputTarget {
                    device: device.clone(),
                    zone: None,
                })
                .collect(),
        };
        bounded(&targets)?;
        let covered = coverage(&targets);
        let mut results = Vec::with_capacity(targets.len());
        for (target, cover) in targets.into_iter().zip(covered) {
            let outcome = match cover {
                Some(by) => TargetOutcome::Covered { by },
                None => match self.stop_effect(target.clone()).await {
                    Ok(restore) => TargetOutcome::Stopped { restore },
                    Err(error) => TargetOutcome::Skipped { error },
                },
            };
            results.push(TargetResult { target, outcome });
        }
        Ok(BatchReport { results })
    }
}

fn bounded(targets: &[OutputTarget]) -> Result<(), OutputError> {
    if targets.len() > MAX_BATCH_TARGETS {
        return Err(OutputError::TooManyTargets {
            limit: MAX_BATCH_TARGETS,
        });
    }
    Ok(())
}
