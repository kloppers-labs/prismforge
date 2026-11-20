use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use prismforge_core::DeviceRef;
use prismforge_core::automation::{
    WAKE_RECONNECT_LIMIT, WAKE_SETTLE_INTERVAL, WAKE_SETTLE_LIMIT, WakeReady, WakeTarget,
    wait_after_wake,
};
use tokio::time::Instant;

/// OpenRGB answers after `live_after`; each device read returns the next list, and the last one repeats.
struct Scripted {
    started: Instant,
    live_after: Duration,
    reads: Mutex<Vec<Option<Vec<&'static str>>>>,
    reconnects: AtomicUsize,
    read_at: Mutex<Vec<Duration>>,
}

impl Scripted {
    fn new(live_after: Duration, reads: Vec<Option<Vec<&'static str>>>) -> Self {
        Self {
            started: Instant::now(),
            live_after,
            reads: Mutex::new(reads),
            reconnects: AtomicUsize::new(0),
            read_at: Mutex::new(Vec::new()),
        }
    }
}

impl WakeTarget for Scripted {
    async fn is_live(&self) -> bool {
        self.started.elapsed() >= self.live_after
    }

    async fn reconnect(&self) {
        self.reconnects.fetch_add(1, Ordering::SeqCst);
    }

    async fn devices(&self) -> Option<Vec<DeviceRef>> {
        self.read_at.lock().unwrap().push(self.started.elapsed());
        let mut reads = self.reads.lock().unwrap();
        let next = if reads.len() > 1 {
            reads.remove(0)
        } else {
            reads[0].clone()
        };
        next.map(|names| names.into_iter().map(DeviceRef::new).collect())
    }
}

fn refs(names: &[&str]) -> Vec<DeviceRef> {
    names.iter().copied().map(DeviceRef::new).collect()
}

#[tokio::test(start_paused = true)]
async fn devices_that_come_back_late_are_waited_for_until_two_reads_agree() {
    let target = Scripted::new(
        Duration::ZERO,
        vec![
            Some(vec!["a"]),
            Some(vec!["a", "b"]),
            Some(vec!["b", "a", "c"]),
            Some(vec!["c", "b", "a"]),
        ],
    );
    let ready = wait_after_wake(&target, &refs(&["a", "b", "c"])).await;
    assert_eq!(
        ready,
        WakeReady::Ready {
            devices: refs(&["a", "b", "c"]),
            missing: Vec::new(),
            settled: true,
        }
    );
    assert_eq!(
        *target.read_at.lock().unwrap(),
        [0, 1, 2, 3].map(|seconds| WAKE_SETTLE_INTERVAL * seconds)
    );
    assert_eq!(target.reconnects.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn devices_still_missing_after_the_wait_are_named() {
    let target = Scripted::new(Duration::ZERO, vec![Some(vec!["a", "b"])]);
    let ready = wait_after_wake(&target, &refs(&["a", "b", "c"])).await;
    assert_eq!(
        ready,
        WakeReady::Ready {
            devices: refs(&["a", "b"]),
            missing: refs(&["c"]),
            settled: true,
        }
    );
}

#[tokio::test(start_paused = true)]
async fn a_device_list_that_never_settles_is_used_after_the_cap() {
    let target = Scripted::new(
        Duration::ZERO,
        (0..40)
            .map(|round| {
                Some(if round % 2 == 0 {
                    vec!["a"]
                } else {
                    vec!["a", "b"]
                })
            })
            .collect(),
    );
    let started = Instant::now();
    let ready = wait_after_wake(&target, &refs(&["a", "b"])).await;
    assert_eq!(started.elapsed(), WAKE_SETTLE_LIMIT);
    let WakeReady::Ready { settled, .. } = ready else {
        panic!("not ready: {ready:?}");
    };
    assert!(!settled);
}

#[tokio::test(start_paused = true)]
async fn a_failed_read_never_counts_as_agreement() {
    let target = Scripted::new(
        Duration::ZERO,
        vec![None, None, Some(vec!["a"]), Some(vec!["a"])],
    );
    let ready = wait_after_wake(&target, &refs(&["a"])).await;
    assert_eq!(
        ready,
        WakeReady::Ready {
            devices: refs(&["a"]),
            missing: Vec::new(),
            settled: true,
        }
    );
    assert_eq!(target.read_at.lock().unwrap().len(), 4);
}

#[tokio::test(start_paused = true)]
async fn openrgb_that_answers_late_is_retried_with_a_growing_wait() {
    let target = Scripted::new(Duration::from_secs(3), vec![Some(vec!["a"])]);
    let ready = wait_after_wake(&target, &refs(&["a"])).await;
    assert!(matches!(ready, WakeReady::Ready { settled: true, .. }));
    assert_eq!(target.reconnects.load(Ordering::SeqCst), 3);
    assert_eq!(
        target.read_at.lock().unwrap().first().copied(),
        Some(Duration::from_millis(3500))
    );
}

#[tokio::test(start_paused = true)]
async fn openrgb_that_never_answers_gives_up_at_the_cap() {
    let target = Scripted::new(Duration::from_secs(3600), vec![Some(vec!["a"])]);
    let started = Instant::now();
    let ready = wait_after_wake(&target, &refs(&["a"])).await;
    assert_eq!(
        ready,
        WakeReady::NeverBack {
            waited: WAKE_RECONNECT_LIMIT
        }
    );
    assert_eq!(started.elapsed(), WAKE_RECONNECT_LIMIT);
    assert!(target.read_at.lock().unwrap().is_empty());
}
