use prismforge_core::session::RestoreOutcome;
use prismforge_core::{DeviceRef, OutputTarget};
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

pub(crate) async fn stop_each<S: ViewSink>(shell: &Shell<S>) -> Vec<DeviceStop> {
    let Some(session) = shell.snapshot().await else {
        return Vec::new();
    };
    let mut devices: Vec<DeviceRef> = Vec::new();
    for assignment in &session.output.scene.assignments {
        if !devices.contains(&assignment.target.device) {
            devices.push(assignment.target.device.clone());
        }
    }
    let mut stops = Vec::with_capacity(devices.len());
    for device in devices {
        let label = session
            .devices
            .iter()
            .find(|known| known.identity.reference() == Some(&device))
            .map_or_else(|| device.to_string(), |known| known.descriptor.name.clone());
        let target = OutputTarget {
            device: device.clone(),
            zone: None,
        };
        let result = match shell.stop_effect(target).await {
            Ok(restore) => StopResult::Stopped { restore },
            Err(error) => StopResult::Failed { error },
        };
        stops.push(DeviceStop {
            device,
            label,
            result,
        });
    }
    stops
}
