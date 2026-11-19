use prismforge_core::DeviceRef;
use prismforge_core::session::{RestoreOutcome, TargetOutcome};
use serde::Serialize;

use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::view::CommandError;

/// What "Stop all PrismForge effects" did, one result per device, kept until the next stop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct StopReport {
    pub sequence: u64,
    pub stops: Vec<DeviceStop>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceStop {
    pub device: DeviceRef,
    /// The name OpenRGB reported when the stop ran.
    pub label: String,
    pub result: StopResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StopResult {
    Stopped { restore: Option<RestoreOutcome> },
    Failed { error: CommandError },
}

/// One stop-all command in the session; a refused command fails every device it would have stopped.
pub(crate) async fn stop_each<S: ViewSink>(shell: &Shell<S>) -> Vec<DeviceStop> {
    let Some(session) = shell.snapshot().await else {
        return Vec::new();
    };
    let label = |device: &DeviceRef| {
        session
            .devices
            .iter()
            .find(|known| known.identity.reference() == Some(device))
            .map_or_else(|| device.to_string(), |known| known.descriptor.name.clone())
    };
    let stop = |device: DeviceRef, result: StopResult| DeviceStop {
        label: label(&device),
        device,
        result,
    };
    match shell.stop_all_effects().await {
        Ok(report) => report
            .results
            .into_iter()
            .filter_map(|each| {
                let result = match each.outcome {
                    TargetOutcome::Stopped { restore } => StopResult::Stopped { restore },
                    TargetOutcome::Skipped { error } => StopResult::Failed {
                        error: error.into(),
                    },
                    TargetOutcome::Applied | TargetOutcome::Covered { .. } => return None,
                };
                Some(stop(each.target.device, result))
            })
            .collect(),
        Err(error) => {
            let mut devices: Vec<DeviceRef> = Vec::new();
            for assignment in &session.output.scene.assignments {
                if !devices.contains(&assignment.target.device) {
                    devices.push(assignment.target.device.clone());
                }
            }
            devices
                .into_iter()
                .map(|device| {
                    let error = error.clone();
                    stop(device, StopResult::Failed { error })
                })
                .collect()
        }
    }
}
