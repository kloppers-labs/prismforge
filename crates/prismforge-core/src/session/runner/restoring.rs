use crate::backend::RgbBackend;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::inventory::Inventory;
use crate::session::diagnostics::Operation;
use crate::session::identify::{RestoreFailure, RestoreOutcome};
use crate::session::previous::{PreviousState, RestoreError};
use crate::session::runner::{Link, Session};

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// The reported state may be PrismForge's own output when an earlier restore did not finish.
    pub(super) fn state_suspect(&self, device: &DeviceRef, inventory: &Inventory) -> bool {
        self.tainted.as_ref() == Some(device)
            || (self.unrestored.contains(device)
                && inventory
                    .device(device)
                    .is_some_and(|entry| entry.descriptor.capabilities.in_direct_mode()))
    }

    pub(super) fn note_restore(&mut self, device: &DeviceRef, outcome: &RestoreOutcome) {
        match outcome {
            RestoreOutcome::Restored => self.unrestored.retain(|known| known != device),
            RestoreOutcome::Unchanged => {}
            _ if self.unrestored.contains(device) => {}
            _ => self.unrestored.push(device.clone()),
        }
    }

    /// Re-reads the inventory after a change, so the snapshot and the next take-over never keep our own output.
    pub(super) async fn settle(&mut self, device: &DeviceRef, changed: bool) {
        self.lose_if_disconnected();
        if changed && matches!(self.link, Link::Connected) {
            self.tainted = Some(device.clone());
            let _ = self.refresh_or_lose(1).await;
        }
    }

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
