mod support;

use std::sync::Arc;
use std::time::Duration;

use prismforge_core::session::{Session, SessionConfig, SessionSnapshot};
use prismforge_desktop::{Publisher, forward};
use prismforge_testkit::DemoBackend;
use tokio::sync::watch;

use support::Recorder;

const SPACING: Duration = Duration::from_millis(50);

fn marked(marker: u64) -> SessionSnapshot {
    let (handle, _session) = Session::new(DemoBackend::new(), SessionConfig::default());
    let mut snapshot = handle.snapshot();
    snapshot.diagnostics.counters.connect_attempts = marker;
    snapshot
}

fn markers(recorder: &Recorder) -> Vec<u64> {
    recorder
        .views()
        .iter()
        .map(|view| {
            view.session
                .as_ref()
                .expect("session view")
                .diagnostics
                .counters
                .connect_attempts
        })
        .collect()
}

fn start(sender: &watch::Sender<SessionSnapshot>) -> (Recorder, tokio::task::JoinHandle<()>) {
    let recorder = Recorder::default();
    let publisher = Arc::new(Publisher::new(recorder.clone()));
    let task = tokio::spawn(forward(sender.subscribe(), publisher, SPACING));
    (recorder, task)
}

#[tokio::test(start_paused = true)]
async fn a_burst_is_delivered_as_its_first_and_its_latest_snapshot() {
    let (sender, _) = watch::channel(marked(0));
    let (recorder, _task) = start(&sender);

    sender.send_replace(marked(1));
    tokio::time::sleep(Duration::from_millis(1)).await;
    for marker in 2..=100 {
        sender.send_replace(marked(marker));
    }
    tokio::time::sleep(SPACING * 4).await;

    assert_eq!(markers(&recorder), [1, 100]);
    let revisions: Vec<u64> = recorder.views().iter().map(|view| view.revision).collect();
    assert!(
        revisions.windows(2).all(|pair| pair[0] < pair[1]),
        "{revisions:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn a_steady_stream_is_delivered_no_closer_than_the_spacing() {
    let (sender, _) = watch::channel(marked(0));
    let (recorder, _task) = start(&sender);

    for marker in 1..=500 {
        sender.send_replace(marked(marker));
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    tokio::time::sleep(SPACING * 2).await;

    let timed = recorder.timed();
    assert!(timed.len() <= 500 / 50 + 2, "{} deliveries", timed.len());
    assert!(
        timed
            .windows(2)
            .all(|pair| pair[1].0.duration_since(pair[0].0) >= SPACING)
    );
    assert_eq!(markers(&recorder).last(), Some(&500));
}

#[tokio::test(start_paused = true)]
async fn the_first_change_after_a_quiet_period_is_delivered_at_once() {
    let (sender, _) = watch::channel(marked(0));
    let (recorder, _task) = start(&sender);

    sender.send_replace(marked(1));
    tokio::time::sleep(Duration::from_secs(1)).await;
    sender.send_replace(marked(2));
    tokio::time::sleep(Duration::from_millis(1)).await;

    assert_eq!(markers(&recorder), [1, 2]);
}

#[tokio::test(start_paused = true)]
async fn forwarding_ends_with_the_session() {
    let (sender, _) = watch::channel(marked(0));
    let (_recorder, task) = start(&sender);

    drop(sender);

    tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .expect("forwarder ended")
        .expect("forwarder did not panic");
}
