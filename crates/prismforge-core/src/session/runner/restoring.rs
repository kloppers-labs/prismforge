use crate::backend::RgbBackend;
use crate::error::BackendError;
use crate::session::diagnostics::Operation;
use crate::session::identify::{RestoreFailure, RestoreOutcome};
use crate::session::previous::{PreviousState, RestoreError};
use crate::session::runner::Session;

impl<B: RgbBackend + Send + 'static> Session<B> {
    pub(super) async fn restore(&mut self, previous: &PreviousState) -> RestoreOutcome {
        let mut attempt = previous.restore(&mut self.backend, 0).await;
        if let Err(failed) = &attempt
            && failed.error == BackendError::StaleInventory
            && self.lost_reason().is_none()
        {
            let steps_done = failed.steps_done;
            attempt = match self.refresh_or_lose(1).await {
                Err(error) => Err(RestoreError { steps_done, error }),
                Ok(()) => {
                    let inventory = self
                        .inventory
                        .as_ref()
                        .expect("refresh stored an inventory");
                    if let Err(failure) = previous.still_matches(inventory) {
                        return partial_or_none(steps_done, failure);
                    }
                    previous.restore(&mut self.backend, steps_done).await
                }
            };
        }
        let failed = match attempt {
            Ok(()) if previous.known => return RestoreOutcome::Restored,
            Ok(()) => {
                return RestoreOutcome::PartlyRestored {
                    failure: RestoreFailure::PreviousStateUnknown,
                };
            }
            Err(failed) => failed,
        };
        let lost = self.lost_reason();
        self.record_error(Operation::Restore, &failed.error, lost.clone());
        let failure = match lost {
            Some(reason) => RestoreFailure::ConnectionLost { reason },
            None => RestoreFailure::Refused {
                error: failed.error,
            },
        };
        partial_or_none(failed.steps_done, failure)
    }
}

fn partial_or_none(steps_done: usize, failure: RestoreFailure) -> RestoreOutcome {
    if steps_done == 0 {
        RestoreOutcome::NotRestored { failure }
    } else {
        RestoreOutcome::PartlyRestored { failure }
    }
}
