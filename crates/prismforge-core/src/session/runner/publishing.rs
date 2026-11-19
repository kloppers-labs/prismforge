use std::sync::atomic::Ordering;

use crate::backend::RgbBackend;
use crate::connection::ConnectionState;
use crate::device::DeviceRef;
use crate::session::diagnostics::Diagnostics;
use crate::session::output::Claim;
use crate::session::runner::{Link, Session};
use crate::session::snapshot::{
    DeviceOutput, IdentifyTarget, OutputCondition, OutputStatus, SessionDevice, SessionSnapshot,
};

impl<B: RgbBackend + Send + 'static> Session<B> {
    /// Wakes subscribers only when the snapshot differs from the one they have.
    pub(super) fn publish(&self) {
        let connection = match &self.link {
            Link::Idle { reason } => ConnectionState::Disconnected {
                reason: reason.clone(),
            },
            Link::Connecting => ConnectionState::Connecting,
            Link::Connected => self.backend.connection_state().clone(),
            Link::Retrying {
                attempt, reason, ..
            } => ConnectionState::Retrying {
                attempt: *attempt,
                reason: reason.clone(),
            },
        };
        let next = SessionSnapshot {
            connection,
            backend: self.backend.info(),
            devices: self.devices(),
            issues: self
                .inventory
                .as_ref()
                .map(|inventory| inventory.issues.clone())
                .unwrap_or_default(),
            identify: self.identify.as_ref().map(|active| IdentifyTarget {
                device: active.plan.device.clone(),
                zone: active.plan.zone,
            }),
            output: self.output_status(),
            diagnostics: Diagnostics {
                last_error: self.last_error.clone(),
                counters: self.counters,
                events: self.events.to_vec(),
            },
        };
        let refused = self.refused_busy.load(Ordering::Relaxed);
        self.snapshot.send_if_modified(|snapshot| {
            let mut next = next;
            let published = snapshot.diagnostics.counters.commands_refused_busy;
            next.diagnostics.counters.commands_refused_busy = published.max(refused);
            let changed = *snapshot != next;
            if changed {
                *snapshot = next;
            }
            changed
        });
    }

    fn devices(&self) -> Vec<SessionDevice> {
        let Some(inventory) = &self.inventory else {
            return Vec::new();
        };
        let available = matches!(self.link, Link::Connected);
        let current = inventory.devices.iter().map(|device| (device, available));
        let missing = inventory.missing.iter().filter_map(|reference| {
            self.seen
                .iter()
                .find(|seen| seen.identity.reference() == Some(reference))
                .map(|device| (device, false))
        });
        current
            .chain(missing)
            .map(|(device, available)| SessionDevice {
                identity: device.identity.clone(),
                descriptor: device.descriptor.clone(),
                available,
            })
            .collect()
    }

    fn output_status(&self) -> OutputStatus {
        OutputStatus {
            frame_rate: self.output.frame_rate,
            scene: self.output.scene(),
            devices: self
                .output
                .claims
                .iter()
                .map(|(device, claim)| DeviceOutput {
                    device: device.clone(),
                    owner: claim.owner(),
                    condition: self.condition(device, claim),
                })
                .collect(),
        }
    }

    fn condition(&self, device: &DeviceRef, claim: &Claim) -> OutputCondition {
        let Claim::Effect(output) = claim else {
            return OutputCondition::Idle;
        };
        if let Some(reason) = self
            .output_pause()
            .or_else(|| self.device_pause(device, output))
        {
            return OutputCondition::Paused { reason };
        }
        match &output.refusal {
            Some(refusal) => OutputCondition::Refused {
                error: refusal.error.clone(),
                attempts: refusal.attempts,
            },
            None => OutputCondition::Running,
        }
    }
}
