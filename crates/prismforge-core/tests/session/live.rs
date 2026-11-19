use std::time::Duration;

use prismforge_core::session::{LIVE_OUTPUT_RATE, LiveOutput, SessionConfig};
use prismforge_core::{DeviceRef, OutputTarget, Rgb, ZoneKind};
use prismforge_testkit::{DemoOutput, FakeServer};
use tokio::time::{Instant, sleep, timeout};

use crate::support::{
    GPU, GPU_LEDS, MEMORY, MOTHERBOARD, config, connected_openrgb, demo, rainbow, reference, solid,
    strip, wait_until, whole,
};

const RED: Rgb = Rgb::new(255, 0, 0);

fn zone(device: &str, zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: Some(zone),
    }
}

fn last_device_frame(outputs: &[DemoOutput], device: &str) -> Vec<Rgb> {
    outputs
        .iter()
        .rev()
        .find_map(|output| match output {
            DemoOutput::DeviceFrame {
                device: target,
                colors,
            } if target.as_str() == device => Some(colors.clone()),
            _ => None,
        })
        .expect("a device frame")
}

fn last_zone_frame(outputs: &[DemoOutput], device: &str, zone: usize) -> Vec<Rgb> {
    outputs
        .iter()
        .rev()
        .find_map(|output| match output {
            DemoOutput::ZoneFrame {
                device: target,
                zone: index,
                colors,
            } if target.as_str() == device && *index == zone => Some(colors.clone()),
            _ => None,
        })
        .expect("a zone frame")
}

fn colors_of(live: &LiveOutput, device: &str, zone: usize) -> Vec<Rgb> {
    live.devices
        .iter()
        .find(|entry| entry.device.as_str() == device)
        .and_then(|entry| entry.zones.iter().find(|each| each.zone == zone))
        .map(|each| each.colors.clone())
        .unwrap_or_else(|| panic!("no live colours for {device} zone {zone}"))
}

#[tokio::test(start_paused = true)]
async fn live_output_equals_written_frames() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("gpu");
    handle
        .apply_effect(zone(MOTHERBOARD, 1), rainbow())
        .await
        .expect("header");
    sleep(Duration::from_millis(100)).await;
    let reported = handle
        .snapshot()
        .devices
        .iter()
        .find(|device| device.identity.reference().map(DeviceRef::as_str) == Some(MOTHERBOARD))
        .expect("motherboard")
        .descriptor
        .colors
        .clone();
    let mut feed = handle.live_output(vec![whole(GPU), whole(MOTHERBOARD)]);

    for _ in 0..12 {
        let live = feed.next().await.expect("live output");
        let outputs = controls.outputs();
        assert_eq!(colors_of(&live, GPU, 0), last_device_frame(&outputs, GPU));
        assert_eq!(
            colors_of(&live, MOTHERBOARD, 1),
            last_zone_frame(&outputs, MOTHERBOARD, 1)
        );
        assert_eq!(colors_of(&live, MOTHERBOARD, 0), reported[..6].to_vec());
        let board = live
            .devices
            .iter()
            .find(|entry| entry.device.as_str() == MOTHERBOARD)
            .expect("board");
        let zones: Vec<(usize, &str, ZoneKind, usize)> = board
            .zones
            .iter()
            .map(|each| (each.zone, each.name.as_str(), each.kind, each.colors.len()))
            .collect();
        assert_eq!(
            zones,
            vec![
                (0, "Onboard", ZoneKind::Linear, 6),
                (1, "Addressable Header 1", ZoneKind::Linear, 30),
                (2, "Addressable Header 2", ZoneKind::Linear, 16),
            ]
        );
    }
}

#[tokio::test(start_paused = true)]
async fn live_output_is_only_for_the_targets_asked_for() {
    let (handle, _) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("gpu");
    handle
        .apply_effect(whole(MOTHERBOARD), rainbow())
        .await
        .expect("board");

    let mut feed = handle.live_output(vec![zone(MOTHERBOARD, 2)]);
    let live = feed.next().await.expect("live output");

    assert_eq!(live.devices.len(), 1);
    assert_eq!(live.devices[0].device.as_str(), MOTHERBOARD);
    let zones: Vec<usize> = live.devices[0].zones.iter().map(|each| each.zone).collect();
    assert_eq!(zones, vec![2]);
}

#[tokio::test(start_paused = true)]
async fn live_output_has_nothing_for_a_device_prismforge_does_not_own() {
    let (handle, _) = demo(config()).await;
    handle
        .select_mode(DeviceRef::new(MEMORY), 2)
        .await
        .expect("mode");
    handle
        .apply_effect(whole(GPU), solid(RED))
        .await
        .expect("gpu");
    let mut lonely = handle.live_output(vec![whole(MEMORY), whole("demo-memory-2")]);
    let nothing = timeout(Duration::from_secs(2), lonely.next()).await;
    assert!(
        nothing.is_err()
            || nothing
                .as_ref()
                .is_ok_and(|live| live.as_ref().is_some_and(|live| live.devices.is_empty())),
        "{nothing:?}"
    );
    drop(lonely);

    let mut feed = handle.live_output(vec![whole(MEMORY), whole(GPU)]);
    let live = feed.next().await.expect("live output");

    let devices: Vec<&str> = live
        .devices
        .iter()
        .map(|entry| entry.device.as_str())
        .collect();
    assert_eq!(devices, vec![GPU]);
    assert_eq!(colors_of(&live, GPU, 0), vec![RED; GPU_LEDS]);
}

#[tokio::test(start_paused = true)]
async fn live_output_rate_is_bounded() {
    const WINDOW: Duration = Duration::from_secs(2);
    let (handle, _) = demo(SessionConfig {
        frame_rate: 60,
        ..config()
    })
    .await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("gpu");
    let mut feed = handle.live_output(vec![whole(GPU)]);

    let started = Instant::now();
    let mut updates = 0u64;
    while let Ok(Some(_)) = timeout(started + WINDOW - Instant::now(), feed.next()).await {
        updates += 1;
    }

    let most = u64::from(LIVE_OUTPUT_RATE) * 2 + 1;
    let frames = handle.snapshot().diagnostics.counters.frames_written;
    assert!(frames >= 110, "the effect itself runs at 60 FPS: {frames}");
    assert!(
        (most - 4..=most).contains(&updates),
        "{updates} live updates in 2 s, at most {most}"
    );
}

/// The frames a fresh demo session writes in two seconds, by time since the apply.
async fn cadence(watch: bool) -> (Vec<(Duration, DemoOutput)>, u64, u64) {
    let (handle, controls) = demo(config()).await;
    let start = Instant::now();
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("gpu");
    if watch {
        let mut feed = handle.live_output(vec![whole(GPU)]);
        for _ in 0..5 {
            feed.next().await.expect("live output");
        }
        drop(feed);
        let mut again = handle.live_output(vec![whole(GPU), whole(MEMORY)]);
        again.next().await.expect("live output");
        sleep(Duration::from_millis(300)).await;
        drop(again);
    }
    sleep_until_after(start, Duration::from_secs(2)).await;
    let counters = handle.snapshot().diagnostics.counters;
    let outputs = controls
        .timed_outputs()
        .into_iter()
        .filter(|(at, _)| *at <= start + Duration::from_secs(2))
        .map(|(at, output)| (at - start, output))
        .collect();
    (outputs, counters.frames_written, counters.ticks_skipped)
}

async fn sleep_until_after(start: Instant, duration: Duration) {
    tokio::time::sleep_until(start + duration + Duration::from_millis(1)).await;
}

#[tokio::test(start_paused = true)]
async fn watching_the_live_output_does_not_change_the_output() {
    let quiet = cadence(false).await;
    let watched = cadence(true).await;

    assert_eq!(quiet.0.len(), watched.0.len());
    assert_eq!(quiet, watched);
}

#[tokio::test(start_paused = true)]
async fn live_output_works_with_openrgb() {
    let server = FakeServer::new();
    server.add_device(0, strip("desk", 5));
    let handle = connected_openrgb(&server).await;
    let snapshot = wait_until(&handle, |snapshot| !snapshot.devices.is_empty()).await;
    let desk = reference(&snapshot, "SN-desk");
    let target = OutputTarget {
        device: desk.clone(),
        zone: None,
    };
    handle
        .apply_effect(target.clone(), solid(RED))
        .await
        .expect("apply");

    let mut feed = handle.live_output(vec![target]);
    let live = feed.next().await.expect("live output");

    assert_eq!(live.devices.len(), 1);
    assert_eq!(live.devices[0].device, desk);
    assert_eq!(live.devices[0].zones[0].name, "Strip");
    assert_eq!(live.devices[0].zones[0].colors, vec![RED; 5]);
}
