use std::time::Duration;

use prismforge_core::session::SessionError;
use prismforge_core::{ConnectionState, Rgb};
use prismforge_effects::ControlValue;
use prismforge_testkit::DemoBackend;
use tokio::time::Instant;

use crate::support::{self, config, start};

const SLOW: Duration = Duration::from_secs(10);

#[tokio::test(start_paused = true)]
async fn slow_backend_call_does_not_block_snapshot_reads() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");
    controls.set_call_delay(SLOW);

    let started = Instant::now();
    let slow = tokio::spawn(handle.refresh());
    tokio::time::sleep(Duration::from_millis(1)).await;
    let during = handle.snapshot();
    let read_at = Instant::now();

    assert_eq!(read_at - started, Duration::from_millis(1));
    assert_eq!(during.connection, ConnectionState::Connected);
    assert_eq!(during.devices.len(), 5);
    slow.await.expect("task").expect("slow refresh");
    assert!(started.elapsed() >= SLOW);
}

#[tokio::test(start_paused = true)]
async fn commands_beyond_the_queue_bound_are_refused_as_busy() {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend, config());
    handle.connect().await.expect("connect");
    controls.set_call_delay(SLOW);
    let capacity = config().command_capacity;

    let in_flight = tokio::spawn(handle.refresh());
    tokio::time::sleep(Duration::from_millis(1)).await;
    let queued: Vec<_> = (0..capacity).map(|_| handle.refresh()).collect();
    let refused_at = Instant::now();
    let refused = handle.refresh().await;

    assert_eq!(refused, Err(SessionError::Busy));
    assert_eq!(Instant::now(), refused_at);
    assert_eq!(
        handle.snapshot().diagnostics.counters.commands_refused_busy,
        1
    );
    in_flight.await.expect("task").expect("in flight refresh");
    for command in queued {
        command.await.expect("queued refresh");
    }
    assert_eq!(
        handle.snapshot().diagnostics.counters.commands_refused_busy,
        1
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_reply_comes_after_the_snapshot_that_shows_it() {
    let (handle, _) = support::demo(support::config()).await;
    let gpu = support::whole(support::GPU);
    for round in 0..300u32 {
        let color = Rgb::new((round % 251) as u8, 7, 9);
        handle
            .apply_effect(gpu.clone(), support::solid(color))
            .await
            .expect("apply");
        let scene = handle.snapshot().output.scene;
        assert_eq!(scene.assignments.len(), 1, "round {round}");
        assert_eq!(
            scene.assignments[0].effect.values.get("color"),
            Some(&ControlValue::Color(color)),
            "round {round}"
        );
        handle.stop_effect(gpu.clone()).await.expect("stop");
        assert!(
            handle.snapshot().output.scene.assignments.is_empty(),
            "round {round}"
        );
    }
}
