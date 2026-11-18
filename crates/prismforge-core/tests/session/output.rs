use std::collections::BTreeMap;
use std::time::Duration;

use prismforge_core::session::{OutputCondition, SessionConfig, SessionHandle};
use prismforge_core::{BackendError, DeviceRef, LossCause, OutputTarget, Rgb};
use prismforge_effects::{linear_positions, prepare};
use prismforge_testkit::DemoOutput;
use tokio::time::{Instant, sleep, sleep_until};

use crate::support::{
    GPU, GPU_LEDS, MEMORY, MOTHERBOARD, config, demo, device_frames, direct_mode_calls,
    is_connected, is_retrying, output_of, rainbow, solid, wait_until, whole,
};

const RED: Rgb = Rgb::new(255, 0, 0);
const BLUE: Rgb = Rgb::new(0, 0, 255);
const MINUTE: Duration = Duration::from_secs(60);

#[tokio::test(start_paused = true)]
async fn coalesces_frames_when_output_is_slow() {
    const SLOW_WRITE: Duration = Duration::from_millis(70);
    const WINDOW: Duration = Duration::from_millis(1_050);
    const TICKS_IN_WINDOW: u64 = 21; // one every 50 ms at 20 FPS
    const WRITES_IN_WINDOW: u64 = 10; // each 70 ms write uses two 50 ms tick slots
    let (handle, controls) = demo(SessionConfig {
        frame_rate: 20,
        ..config()
    })
    .await;
    controls.set_call_delay(SLOW_WRITE);

    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    let started = Instant::now();
    sleep_until(started + WINDOW).await;

    let counters = handle.snapshot().diagnostics.counters;
    let frames: Vec<(Instant, Vec<Rgb>)> = controls
        .timed_outputs()
        .into_iter()
        .filter_map(|(at, output)| match output {
            DemoOutput::DeviceFrame { colors, .. } => Some((at, colors)),
            _ => None,
        })
        .collect();
    assert_eq!(frames.len() as u64, WRITES_IN_WINDOW);
    assert_eq!(counters.frames_written, WRITES_IN_WINDOW);
    assert_eq!(counters.ticks_skipped, WRITES_IN_WINDOW);
    assert!(counters.frames_written < TICKS_IN_WINDOW);
    let effect = prepare(&rainbow()).expect("valid");
    for (written_at, colors) in &frames {
        let began = *written_at - SLOW_WRITE - started;
        assert_eq!(
            colors,
            &effect.render(began, &linear_positions(GPU_LEDS), 0),
            "frame written at {began:?}"
        );
    }
    let (last, _) = frames.last().expect("frames");
    assert!(started + WINDOW - *last < SLOW_WRITE + Duration::from_millis(50));
}

#[tokio::test(start_paused = true)]
async fn bounds_retry_work() {
    const ATTEMPTS_IN_A_MINUTE: u64 = 7; // at 0, 1, 3, 7, 15, 31 and 47 s
    let (handle, controls) = demo(config()).await;
    controls.refuse_writes(&DeviceRef::new(GPU));

    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply gpu");
    handle
        .apply_effect(whole(MEMORY), rainbow())
        .await
        .expect("apply memory");
    sleep(MINUTE).await;

    let snapshot = handle.snapshot();
    let outputs = controls.outputs();
    assert_eq!(
        snapshot.diagnostics.counters.output_writes_refused,
        ATTEMPTS_IN_A_MINUTE
    );
    assert!(device_frames(&outputs, GPU).is_empty());
    let memory_frames = device_frames(&outputs, MEMORY).len();
    assert!(
        (1_799..=1_801).contains(&memory_frames),
        "{memory_frames} frames in a minute at 30 FPS"
    );
    assert!(matches!(
        output_of(&snapshot, GPU).expect("gpu output").condition,
        OutputCondition::Refused {
            error: BackendError::Rejected { .. },
            attempts: ATTEMPTS_IN_A_MINUTE
        }
    ));
    assert_eq!(
        output_of(&snapshot, MEMORY)
            .expect("memory output")
            .condition,
        OutputCondition::Running
    );
}

#[tokio::test(start_paused = true)]
async fn does_not_reanimate_static_scene() {
    let (handle, controls) = demo(config()).await;

    handle
        .apply_effect(whole(GPU), solid(RED))
        .await
        .expect("apply red");
    sleep(MINUTE).await;
    let after_first = controls.outputs();
    let counters = handle.snapshot().diagnostics.counters;

    assert_eq!(device_frames(&after_first, GPU), [vec![RED; GPU_LEDS]]);
    assert_eq!(counters.frames_written, 1);
    assert_eq!(counters.ticks_skipped, 0);

    handle
        .apply_effect(whole(GPU), solid(BLUE))
        .await
        .expect("apply blue");
    sleep(MINUTE).await;
    assert_eq!(device_frames(&controls.outputs(), GPU).len(), 2);

    controls.lose_connection(LossCause::Timeout);
    wait_until(&handle, is_retrying).await;
    wait_until(&handle, is_connected).await;
    sleep(MINUTE).await;
    let after_reconnect = controls.outputs();
    assert_eq!(
        device_frames(&after_reconnect, GPU),
        [
            vec![RED; GPU_LEDS],
            vec![BLUE; GPU_LEDS],
            vec![BLUE; GPU_LEDS]
        ]
    );
    assert_eq!(direct_mode_calls(&after_reconnect, GPU), 2);

    controls.change_inventory();
    sleep(MINUTE).await;
    assert_eq!(device_frames(&controls.outputs(), GPU).len(), 4);
    assert_eq!(handle.snapshot().diagnostics.counters.frames_written, 4);
}

#[tokio::test(start_paused = true)]
async fn disconnect_stays_prompt_while_effects_run_at_thirty_frames_per_second() {
    let (handle, controls) = demo(config()).await;
    controls.set_call_delay(Duration::from_millis(2));
    for device in [GPU, MEMORY, MOTHERBOARD] {
        handle
            .apply_effect(whole(device), rainbow())
            .await
            .expect("apply");
    }
    sleep(Duration::from_secs(1)).await;
    let running = handle.snapshot().diagnostics.counters.frames_written;

    let asked = Instant::now();
    handle.disconnect().await.expect("disconnect");
    let took = asked.elapsed();

    assert!(running >= 85, "{running} frames in one second");
    assert!(
        took <= Duration::from_millis(30),
        "disconnect took {took:?}"
    );
    assert!(handle.snapshot().output.devices.is_empty());
    let outputs = controls.outputs();
    assert!(outputs.contains(&DemoOutput::ModeReapplied {
        device: DeviceRef::new(GPU),
        mode: 2
    }));
    let after = outputs.len();
    sleep(MINUTE).await;
    assert_eq!(controls.outputs().len(), after);
}

#[tokio::test(start_paused = true)]
async fn frame_rate_is_clamped_to_a_safe_range() {
    let (handle, controls) = demo(SessionConfig {
        frame_rate: 0,
        ..config()
    })
    .await;
    assert_eq!(handle.snapshot().output.frame_rate, 1);

    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(10_500)).await;
    assert_eq!(device_frames(&controls.outputs(), GPU).len(), 11);

    assert_eq!(handle.set_frame_rate(10_000).await, Ok(60));
    assert_eq!(handle.set_frame_rate(0).await, Ok(1));
    assert_eq!(handle.set_frame_rate(24).await, Ok(24));
    assert_eq!(handle.snapshot().output.frame_rate, 24);
}

#[tokio::test(start_paused = true)]
async fn each_tick_sends_one_full_frame_per_device() {
    let (handle, controls) = demo(config()).await;
    for device in [GPU, MEMORY] {
        handle
            .apply_effect(whole(device), rainbow())
            .await
            .expect("apply");
    }
    sleep(Duration::from_millis(1_010)).await;

    let mut ticks: BTreeMap<Instant, Vec<(String, usize)>> = BTreeMap::new();
    for (at, output) in controls.timed_outputs() {
        match output {
            DemoOutput::DeviceFrame { device, colors } => ticks
                .entry(at)
                .or_default()
                .push((device.as_str().to_owned(), colors.len())),
            DemoOutput::DirectMode { .. } => {}
            other => panic!("unexpected output {other:?}"),
        }
    }
    let both = vec![(GPU.to_owned(), GPU_LEDS), (MEMORY.to_owned(), 8)];
    let first = ticks.pop_first().expect("first tick");
    assert_eq!(first.1, [(GPU.to_owned(), GPU_LEDS)]);
    assert!(ticks.values().all(|frames| frames == &both), "{ticks:?}");
    assert_eq!(ticks.len(), 30);
}

#[tokio::test(start_paused = true)]
async fn a_zone_assignment_writes_only_that_zone() {
    let (handle, controls) = demo(config()).await;
    let header = OutputTarget {
        device: DeviceRef::new(MOTHERBOARD),
        zone: Some(1),
    };

    handle
        .apply_effect(header.clone(), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(500)).await;

    let outputs = controls.outputs();
    assert_eq!(
        outputs[0],
        DemoOutput::DirectMode {
            device: DeviceRef::new(MOTHERBOARD)
        }
    );
    let frames = &outputs[1..];
    assert!(frames.len() >= 15);
    assert!(frames.iter().all(|output| matches!(
        output,
        DemoOutput::ZoneFrame { zone: 1, colors, .. } if colors.len() == 30
    )));
    assert_eq!(handle.snapshot().output.scene.assignments[0].target, header);
}

#[tokio::test(start_paused = true)]
async fn connection_loss_pauses_output_until_direct_mode_is_enabled_again() {
    let (handle, controls) = demo(config()).await;
    controls.fail_connect(true);
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;

    controls.lose_connection(LossCause::Timeout);
    let lost = wait_until(&handle, is_retrying).await;
    let before = controls.outputs().len();
    sleep(Duration::from_secs(5)).await;
    assert_eq!(controls.outputs().len(), before);
    assert!(matches!(
        output_of(&lost, GPU).expect("gpu output").condition,
        OutputCondition::Paused { .. }
    ));

    controls.fail_connect(false);
    wait_until(&handle, is_connected).await;
    sleep(Duration::from_millis(100)).await;
    let resumed = &controls.outputs()[before..];
    assert_eq!(
        resumed[0],
        DemoOutput::DirectMode {
            device: DeviceRef::new(GPU)
        }
    );
    assert!(resumed.len() > 2);
    assert_eq!(
        output_of(&handle.snapshot(), GPU)
            .expect("gpu output")
            .condition,
        OutputCondition::Running
    );
}

#[tokio::test(start_paused = true)]
async fn a_command_waits_for_at_most_one_backend_call_while_effects_run() {
    let (handle, controls) = demo(config()).await;
    for device in [GPU, MEMORY, MOTHERBOARD] {
        handle
            .apply_effect(whole(device), rainbow())
            .await
            .expect("apply");
    }
    controls.set_call_delay(SLOW_CALL);

    let slowest = slowest_reply(&handle).await;
    assert!(slowest <= SLOW_CALL, "a command waited {slowest:?}");

    let before = handle.snapshot().diagnostics.counters.frames_written;
    let senders: Vec<_> = (0..3)
        .map(|_| {
            let handle = handle.clone();
            tokio::spawn(async move {
                for _ in 0..20 {
                    handle.refresh().await.expect("refresh");
                }
            })
        })
        .collect();
    for sender in senders {
        sender.await.expect("sender");
    }
    let during = handle.snapshot().diagnostics.counters.frames_written - before;
    assert!(during >= 30, "{during} frames while commands kept coming");
}

const SLOW_CALL: Duration = Duration::from_millis(25);

async fn slowest_reply(handle: &SessionHandle) -> Duration {
    let mut slowest = Duration::ZERO;
    for _ in 0..40 {
        sleep(Duration::from_millis(7)).await;
        let asked = Instant::now();
        handle.set_frame_rate(30).await.expect("set frame rate");
        slowest = slowest.max(asked.elapsed());
    }
    slowest
}

#[tokio::test(start_paused = true)]
async fn a_command_waits_for_at_most_one_backend_call_while_zone_effects_run() {
    let (handle, controls) = demo(config()).await;
    for zone in 0..3 {
        let target = OutputTarget {
            device: DeviceRef::new(MOTHERBOARD),
            zone: Some(zone),
        };
        handle.apply_effect(target, rainbow()).await.expect("apply");
    }
    controls.set_call_delay(SLOW_CALL);

    let slowest = slowest_reply(&handle).await;

    assert!(slowest <= SLOW_CALL, "a command waited {slowest:?}");
}

#[tokio::test(start_paused = true)]
async fn a_command_waits_for_at_most_one_backend_call_while_output_resumes() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;
    controls.set_call_delay(SLOW_CALL);

    controls.lose_connection(LossCause::Timeout);
    wait_until(&handle, is_retrying).await;
    wait_until(&handle, is_connected).await;
    let asked = Instant::now();
    handle.set_frame_rate(30).await.expect("set frame rate");
    let waited = asked.elapsed();
    sleep(Duration::from_millis(200)).await;

    assert!(waited <= SLOW_CALL, "a command waited {waited:?}");
    assert_eq!(direct_mode_calls(&controls.outputs(), GPU), 2);
}

fn wakeups(handle: &SessionHandle) -> u64 {
    handle.snapshot().diagnostics.counters.output_wakeups
}

/// Moves the clock on in one-second steps and lets the session run; a spinning loop shows up as many wake-ups.
async fn pass_time(seconds: u64) {
    for _ in 0..seconds {
        tokio::time::advance(Duration::from_secs(1)).await;
        for _ in 0..200 {
            tokio::task::yield_now().await;
        }
    }
}

async fn wakeups_over(handle: &SessionHandle, seconds: u64) -> u64 {
    let before = wakeups(handle);
    pass_time(seconds).await;
    wakeups(handle) - before
}

fn motherboard_zone(zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(MOTHERBOARD),
        zone: Some(zone),
    }
}

#[tokio::test(start_paused = true)]
async fn a_refused_device_with_nothing_left_to_write_does_not_wake_the_loop() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(motherboard_zone(1), solid(RED))
        .await
        .expect("static zone");
    sleep(Duration::from_millis(100)).await;
    controls.refuse_writes(&DeviceRef::new(MOTHERBOARD));
    handle
        .apply_effect(motherboard_zone(0), rainbow())
        .await
        .expect("animated zone");
    sleep(Duration::from_millis(100)).await;
    assert!(matches!(
        output_of(&handle.snapshot(), MOTHERBOARD)
            .expect("motherboard output")
            .condition,
        OutputCondition::Refused { .. }
    ));

    assert_eq!(handle.stop_effect(motherboard_zone(0)).await, Ok(None));
    let wakeups = wakeups_over(&handle, 5).await;

    assert_eq!(wakeups, 0);
    assert_eq!(
        output_of(&handle.snapshot(), MOTHERBOARD)
            .expect("motherboard output")
            .condition,
        OutputCondition::Running
    );
}

#[tokio::test(start_paused = true)]
async fn every_output_condition_wakes_the_loop_only_for_real_work() {
    const ATTEMPTS_IN_A_MINUTE: u64 = 7; // at 0, 1, 3, 7, 15, 31 and 47 s
    let (handle, controls) = demo(config()).await;
    let gpu = DeviceRef::new(GPU);
    let memory = DeviceRef::new(MEMORY);

    assert_eq!(wakeups_over(&handle, 60).await, 0, "empty scene");

    let before = wakeups(&handle);
    handle
        .apply_effect(whole(GPU), solid(RED))
        .await
        .expect("static");
    pass_time(60).await;
    assert_eq!(wakeups(&handle) - before, 1, "static, written once");

    controls.refuse_writes(&memory);
    let before = wakeups(&handle);
    handle
        .apply_effect(whole(MEMORY), rainbow())
        .await
        .expect("refused");
    pass_time(60).await;
    assert_eq!(
        wakeups(&handle) - before,
        ATTEMPTS_IN_A_MINUTE,
        "refused and animated"
    );

    let before = wakeups(&handle);
    handle
        .apply_effect(whole(MEMORY), solid(BLUE))
        .await
        .expect("replace while refused");
    pass_time(60).await;
    assert_eq!(
        wakeups(&handle) - before,
        ATTEMPTS_IN_A_MINUTE,
        "replaced while refused, still refused"
    );

    handle
        .stop_effect(whole(MEMORY))
        .await
        .expect("stop while refused");
    assert_eq!(wakeups_over(&handle, 60).await, 0, "stopped while refused");

    let identify = tokio::spawn(handle.identify(gpu.clone(), None, Duration::from_secs(10)));
    let during_identify = wakeups_over(&handle, 5).await;
    let after_identify = wakeups_over(&handle, 55).await;
    identify.await.expect("task").expect("identify");
    assert_eq!(during_identify, 0, "paused by identify");
    assert_eq!(after_identify, 1, "rewritten once after identify");

    controls.fail_connect(true);
    controls.lose_connection(LossCause::Timeout);
    assert_eq!(wakeups_over(&handle, 60).await, 0, "paused, not connected");

    controls.fail_connect(false);
    let reconnected = wakeups_over(&handle, 60).await;
    assert!(is_connected(&handle.snapshot()));
    assert_eq!(reconnected, 1, "direct mode pending, then one frame");
    assert_eq!(direct_mode_calls(&controls.outputs(), GPU), 3);

    handle.stop_effect(whole(GPU)).await.expect("stop");
    assert_eq!(wakeups_over(&handle, 60).await, 0, "empty scene again");
}
