use std::future::Future;
use std::time::Duration;

use tokio::time::{Instant, sleep};

use crate::device::DeviceRef;

/// Waits between tries to reach OpenRGB after a wake; the last wait repeats.
pub const WAKE_BACKOFF: [Duration; 4] = [
    Duration::from_millis(500),
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
];
/// PrismForge stops waiting for OpenRGB and for the devices this long after a wake.
pub const WAKE_RECONNECT_LIMIT: Duration = Duration::from_secs(30);
/// Time between two reads of the device list while the devices come back.
pub const WAKE_SETTLE_INTERVAL: Duration = Duration::from_secs(1);
/// The device list is used as it is once this much time has passed, or at the wake's deadline, even if it still changes.
pub const WAKE_SETTLE_LIMIT: Duration = Duration::from_secs(15);

/// The session after a wake, as the wake sequence sees it.
pub trait WakeTarget {
    fn is_live(&self) -> impl Future<Output = bool> + Send;

    /// Asks for a connect now; returns when that try ended.
    fn reconnect(&self) -> impl Future<Output = ()> + Send;

    /// Reads the device list again; `None` when the read failed.
    fn devices(&self) -> impl Future<Output = Option<Vec<DeviceRef>>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WakeReady {
    Ready {
        devices: Vec<DeviceRef>,
        /// Devices that were there before the sleep and are not back.
        missing: Vec<DeviceRef>,
        /// False when the list still changed at the limit.
        settled: bool,
    },
    NeverBack {
        waited: Duration,
    },
}

/// Waits until OpenRGB answers and two reads of the device list a second apart agree; every wait is capped.
pub async fn wait_after_wake<T: WakeTarget>(target: &T, before: &[DeviceRef]) -> WakeReady {
    let started = Instant::now();
    let mut tries = 0;
    loop {
        if !reach(target, started, &mut tries).await {
            return WakeReady::NeverBack {
                waited: started.elapsed(),
            };
        }
        let deadline = started + WAKE_RECONNECT_LIMIT;
        let Some((devices, settled)) = settle(target, deadline).await else {
            continue;
        };
        let missing = before
            .iter()
            .filter(|device| !devices.contains(device))
            .cloned()
            .collect();
        return WakeReady::Ready {
            devices,
            missing,
            settled,
        };
    }
}

/// Retries the connection until OpenRGB answers; false once [`WAKE_RECONNECT_LIMIT`] passed since the wake.
async fn reach<T: WakeTarget>(target: &T, started: Instant, tries: &mut usize) -> bool {
    while !target.is_live().await {
        if started.elapsed() >= WAKE_RECONNECT_LIMIT {
            return false;
        }
        target.reconnect().await;
        if target.is_live().await {
            break;
        }
        let pause = WAKE_BACKOFF[(*tries).min(WAKE_BACKOFF.len() - 1)];
        sleep(pause.min(WAKE_RECONNECT_LIMIT.saturating_sub(started.elapsed()))).await;
        *tries += 1;
    }
    true
}

/// `None` when the connection was lost again while reading.
async fn settle<T: WakeTarget>(target: &T, deadline: Instant) -> Option<(Vec<DeviceRef>, bool)> {
    let stop = deadline.min(Instant::now() + WAKE_SETTLE_LIMIT);
    let mut last = sorted(target.devices().await);
    let mut known = last.clone();
    loop {
        if last.is_none() && !target.is_live().await {
            return None;
        }
        let now = Instant::now();
        if now >= stop {
            return Some((known.unwrap_or_default(), false));
        }
        sleep(WAKE_SETTLE_INTERVAL.min(stop - now)).await;
        let next = sorted(target.devices().await);
        if next.is_some() && next == last {
            return Some((next.unwrap_or_default(), true));
        }
        if next.is_some() {
            known.clone_from(&next);
        }
        last = next;
    }
}

fn sorted(devices: Option<Vec<DeviceRef>>) -> Option<Vec<DeviceRef>> {
    devices.map(|mut devices| {
        devices.sort();
        devices.dedup();
        devices
    })
}
