use tokio::time::Instant;

use crate::backend::RgbBackend;
use crate::error::BackendError;
use crate::session::diagnostics::{Operation, SessionEventKind};
use crate::session::identify::{IdentifyOutcome, IdentifyPlan, RestoreOutcome};
use crate::session::previous::PreviousState;
use crate::session::runner::{ActiveIdentify, Link, Session, not_restored_after_loss};
use crate::session::{IdentifyRequest, Reply, SessionError};

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(super) async fn start_identify(
        &mut self,
        request: IdentifyRequest,
        reply: Reply<IdentifyOutcome>,
    ) {
        if self.tainted.is_some() && self.identify.is_none() && matches!(self.link, Link::Connected)
        {
            let _ = self.refresh_or_lose(1).await;
        }
        let plan = match self.plan_identify(&request) {
            Ok(plan) => plan,
            Err(error) => {
                let _ = reply.send(Err(error));
                return;
            }
        };
        self.push(SessionEventKind::IdentifyStarted {
            device: plan.device.clone(),
            zone: plan.zone,
        });
        let Err(failed) = plan.light(&mut self.backend).await else {
            self.identify = Some(ActiveIdentify {
                plan,
                until: Instant::now() + request.duration,
                reply,
            });
            return;
        };
        let lost = self.lost_reason();
        self.record_error(Operation::Identify, &failed.error, lost.clone());
        let restore = match lost {
            Some(reason) => not_restored_after_loss(&reason),
            None if !failed.device_changed => RestoreOutcome::Unchanged,
            None => self.restore(&plan.previous).await,
        };
        self.end_identify(plan, reply, Some(failed.error), restore)
            .await;
    }

    fn plan_identify(&self, request: &IdentifyRequest) -> Result<IdentifyPlan, SessionError> {
        if let Some(active) = &self.identify {
            return Err(SessionError::IdentifyInProgress {
                device: active.plan.device.clone(),
            });
        }
        let max = self.config.max_identify_duration;
        if request.duration.is_zero() || request.duration > max {
            return Err(SessionError::InvalidDuration {
                max_ms: max.as_millis() as u64,
            });
        }
        let device = &request.device;
        self.ensure_not_external(device)?;
        let inventory = self.live_inventory()?;
        let suspect = self.state_suspect(device, inventory);
        let mut plan = IdentifyPlan::new(inventory, device, request.zone, suspect)?;
        if let Some(output) = self.output.effect(device) {
            plan.previous = PreviousState::shown(device, output.shown.clone());
        }
        Ok(plan)
    }

    pub(super) async fn finish_identify(&mut self, active: ActiveIdentify) {
        let restore = self.restore(&active.plan.previous).await;
        self.end_identify(active.plan, active.reply, None, restore)
            .await;
    }

    async fn end_identify(
        &mut self,
        plan: IdentifyPlan,
        reply: Reply<IdentifyOutcome>,
        light_failure: Option<BackendError>,
        restore: RestoreOutcome,
    ) {
        let changed = restore != RestoreOutcome::Unchanged;
        let device = plan.device.clone();
        self.reply_identify(plan, reply, light_failure, restore);
        self.output.touch(&device);
        self.settle(&device, changed).await;
    }

    pub(super) fn reply_identify(
        &mut self,
        plan: IdentifyPlan,
        reply: Reply<IdentifyOutcome>,
        light_failure: Option<BackendError>,
        restore: RestoreOutcome,
    ) {
        self.note_restore(&plan.device, &restore);
        let outcome = IdentifyOutcome {
            device: plan.device,
            zone: plan.zone,
            light_failure,
            restore,
        };
        self.push(SessionEventKind::IdentifyFinished {
            outcome: outcome.clone(),
        });
        let _ = reply.send(Ok(outcome));
    }
}
