use std::sync::Arc;
use std::time::Duration;

use prismforge_effects::Rgb;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio::time::{Instant, sleep_until};

use crate::device::{DeviceRef, MatrixSize, ZoneKind};
use crate::inventory::Inventory;
use crate::scene::OutputTarget;
use crate::session::output::Output;

/// Most live output updates a feed delivers each second.
pub const LIVE_OUTPUT_RATE: u32 = 25;

/// The colours PrismForge last wrote, per zone, for the devices it runs effects on and a feed asked for.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct LiveOutput {
    pub devices: Vec<LiveDevice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct LiveDevice {
    pub device: DeviceRef,
    pub zones: Vec<LiveZone>,
}

/// One zone in LED order; LEDs no effect has written yet hold the colours reported at take-over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct LiveZone {
    pub zone: usize,
    pub name: String,
    pub kind: ZoneKind,
    pub matrix: Option<MatrixSize>,
    pub colors: Vec<Rgb>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LiveRequest {
    pub feed: u64,
    pub targets: Vec<OutputTarget>,
}

/// The live output side of a session: what feeds ask for and what the session last published.
#[derive(Clone)]
pub(crate) struct LiveChannels {
    pub requests: Arc<watch::Sender<LiveRequest>>,
    pub output: Arc<watch::Sender<LiveOutput>>,
}

impl LiveChannels {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(watch::channel(LiveRequest::default()).0),
            output: Arc::new(watch::channel(LiveOutput::default()).0),
        }
    }

    pub fn feed(&self, feed: u64, targets: Vec<OutputTarget>) -> LiveFeed {
        self.requests.send_replace(LiveRequest { feed, targets });
        LiveFeed {
            id: feed,
            requests: self.requests.clone(),
            output: self.output.subscribe(),
            last: None,
        }
    }
}

/// Live output for the targets it was opened with; a newer feed replaces it, and dropping it stops the copying.
pub struct LiveFeed {
    id: u64,
    requests: Arc<watch::Sender<LiveRequest>>,
    output: watch::Receiver<LiveOutput>,
    last: Option<Instant>,
}

impl LiveFeed {
    /// The next change, never sooner than 1 / [`LIVE_OUTPUT_RATE`] seconds after the last; `None` once the session ended.
    pub async fn next(&mut self) -> Option<LiveOutput> {
        if let Some(last) = self.last {
            sleep_until(last + Duration::from_secs(1) / LIVE_OUTPUT_RATE).await;
        }
        self.output.changed().await.ok()?;
        self.last = Some(Instant::now());
        Some(self.output.borrow_and_update().clone())
    }
}

impl Drop for LiveFeed {
    fn drop(&mut self) {
        self.requests.send_if_modified(|request| {
            let mine = request.feed == self.id;
            if mine {
                request.targets.clear();
            }
            mine
        });
    }
}

/// Copies what the output state holds; it never calls the backend.
pub(crate) fn collect(
    request: &LiveRequest,
    output: &Output,
    inventory: Option<&Inventory>,
) -> LiveOutput {
    let Some(inventory) = inventory else {
        return LiveOutput::default();
    };
    let mut devices: Vec<LiveDevice> = Vec::new();
    for target in &request.targets {
        let (Some(effect), Some(entry)) = (
            output.effect(&target.device),
            inventory.device(&target.device),
        ) else {
            continue;
        };
        let descriptor = &entry.descriptor;
        let index = match devices.iter().position(|live| live.device == target.device) {
            Some(index) => index,
            None => {
                devices.push(LiveDevice {
                    device: target.device.clone(),
                    zones: Vec::new(),
                });
                devices.len() - 1
            }
        };
        let mut start = 0;
        for (zone, shape) in descriptor.zones.iter().enumerate() {
            let leds = start..start + shape.led_count;
            start = leds.end;
            let wanted = target.zone.is_none_or(|asked| asked == zone);
            let zones = &mut devices[index].zones;
            if !wanted || zones.iter().any(|live| live.zone == zone) {
                continue;
            }
            zones.push(LiveZone {
                zone,
                name: shape.name.clone(),
                kind: shape.kind,
                matrix: shape.matrix,
                colors: effect.shown.get(leds).unwrap_or_default().to_vec(),
            });
        }
        devices[index].zones.sort_by_key(|live| live.zone);
    }
    LiveOutput { devices }
}
