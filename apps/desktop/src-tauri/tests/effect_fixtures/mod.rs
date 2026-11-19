use prismforge_core::session::{OutputCondition, OutputError, SessionHandle, SessionSnapshot};
use prismforge_core::{ConnectionState, DeviceRef, LossCause, OutputTarget, Rgb};
use prismforge_desktop::{CommandError, Shell, ShellConfig, ShellView, ViewSink, preview_effect};
use prismforge_effects::{ControlValue, Direction, EffectInstance, PreviewRequest};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice, wire};

use crate::{BLINK, Module, breathing_strip, demo_connected, openrgb, reference, wait_until};

const GPU: &str = "demo-graphics-card";
const MOTHERBOARD: &str = "demo-motherboard";
const MEMORY_1: &str = "demo-memory-1";
const MEMORY_2: &str = "demo-memory-2";
const LIGHT_BAR: &str = "demo-light-bar";
const PLUGIN: &str = "OpenRGB Effects plugin";

struct Silent;

impl ViewSink for Silent {
    fn deliver(&self, _: &ShellView) {}
}

fn effect(id: &str, values: &[(&str, ControlValue)]) -> EffectInstance {
    EffectInstance {
        effect: id.into(),
        values: values
            .iter()
            .map(|(control, value)| ((*control).to_owned(), value.clone()))
            .collect(),
    }
}

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

fn zone(device: &str, zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: Some(zone),
    }
}

fn rainbow() -> EffectInstance {
    effect("rainbow-wave", &[])
}

fn condition_of(snapshot: &SessionSnapshot, device: &str) -> Option<OutputCondition> {
    snapshot
        .output
        .devices
        .iter()
        .find(|output| output.device.as_str() == device)
        .map(|output| output.condition.clone())
}

async fn applied(handle: &SessionHandle, target: OutputTarget, instance: EffectInstance) {
    handle
        .apply_effect(target, instance)
        .await
        .expect("apply effect");
}

pub async fn sessions(sessions: &mut Module) {
    let (handle, controls) = demo_connected().await;
    controls.refuse_writes(&DeviceRef::new(MEMORY_1));
    applied(&handle, whole(GPU), rainbow()).await;
    let highlight = effect(
        "moving-highlight",
        &[
            ("color", ControlValue::Color(Rgb::new(255, 64, 0))),
            ("direction", ControlValue::Direction(Direction::Backward)),
        ],
    );
    applied(&handle, zone(MOTHERBOARD, 1), highlight).await;
    applied(
        &handle,
        whole(MEMORY_1),
        effect(
            "solid",
            &[("color", ControlValue::Color(Rgb::new(0, 128, 255)))],
        ),
    )
    .await;
    handle
        .declare_external(DeviceRef::new(MEMORY_2), PLUGIN.into())
        .await
        .expect("declare");
    handle
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("select mode");
    let running = wait_until(&handle, |snapshot| {
        matches!(
            condition_of(snapshot, MEMORY_1),
            Some(OutputCondition::Refused { .. })
        ) && condition_of(snapshot, GPU) == Some(OutputCondition::Running)
    })
    .await;
    sessions.add("demoEffectsRunning", &running);

    let identify = tokio::spawn(handle.identify(DeviceRef::new(GPU), None, BLINK));
    let identifying = wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    sessions.add("demoEffectIdentifying", &identifying);
    identify.await.expect("task").expect("identify");

    controls.fail_connect(true);
    controls.lose_connection(LossCause::Timeout);
    let lost = wait_until(&handle, |snapshot| {
        matches!(snapshot.connection, ConnectionState::Retrying { .. })
    })
    .await;
    sessions.add("demoEffectsNotConnected", &lost);
}

pub async fn catalog(catalog: &mut Module) {
    let (stores, _dir) = crate::support::stores("catalog");
    let shell = Shell::new(Silent, ShellConfig::default(), stores);
    shell.start(DemoBackend::new()).await.expect("start");
    catalog.add("catalog", &shell.catalog().await.expect("catalog"));
    shell.stop().await;
}

pub async fn stops(stops: &mut Module) {
    let (handle, _) = demo_connected().await;
    applied(&handle, whole(GPU), rainbow()).await;
    let restored = handle.stop_effect(whole(GPU)).await;
    stops.add("stopRestored", &restore(restored));

    let (handle, _) = demo_connected().await;
    applied(&handle, zone(MOTHERBOARD, 0), rainbow()).await;
    applied(&handle, zone(MOTHERBOARD, 1), rainbow()).await;
    let kept = handle.stop_effect(zone(MOTHERBOARD, 1)).await;
    stops.add("stopOtherZonesStillRun", &restore(kept));

    let (handle, controls) = demo_connected().await;
    controls.hide_active_mode(&DeviceRef::new(GPU));
    handle.refresh().await.expect("refresh");
    applied(&handle, whole(GPU), rainbow()).await;
    let unknown = handle.stop_effect(whole(GPU)).await;
    stops.add("stopPreviousStateUnknown", &restore(unknown));

    let (handle, controls) = demo_connected().await;
    applied(&handle, whole(GPU), rainbow()).await;
    controls.fail_connect(true);
    controls.lose_connection(LossCause::Timeout);
    wait_until(&handle, |snapshot| {
        matches!(snapshot.connection, ConnectionState::Retrying { .. })
    })
    .await;
    let lost = handle.stop_effect(whole(GPU)).await;
    stops.add("stopNotRestoredConnectionLost", &restore(lost));

    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha", "Fixture Strip"));
    let handle = openrgb(&server);
    handle.connect().await.expect("connect");
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    let target = OutputTarget {
        device: alpha,
        zone: None,
    };
    applied(&handle, target.clone(), rainbow()).await;
    server.reject(wire::UPDATE_MODE);
    let refused = handle.stop_effect(target).await;
    stops.add("stopNotRestoredRefused", &restore(refused));
}

pub async fn errors(errors: &mut Module) {
    let (handle, _) = demo_connected().await;
    let invalid = handle
        .apply_effect(
            whole(GPU),
            effect(
                "breathing",
                &[
                    ("speed", ControlValue::Number(500)),
                    ("brightness", ControlValue::Number(150)),
                ],
            ),
        )
        .await;
    errors.add("invalidBreathing", &command_error(invalid));

    let palette = handle
        .apply_effect(
            whole(GPU),
            effect(
                "gradient",
                &[("palette", ControlValue::Palette(vec![Rgb::new(255, 0, 0)]))],
            ),
        )
        .await;
    errors.add("invalidGradientPalette", &command_error(palette));

    handle
        .declare_external(DeviceRef::new(GPU), PLUGIN.into())
        .await
        .expect("declare");
    let conflict = handle.apply_effect(whole(GPU), rainbow()).await;
    errors.add("outputConflict", &command_error(conflict));

    let none = handle.stop_effect(whole(MOTHERBOARD)).await;
    errors.add("noEffect", &command_error(none));
}

pub fn previews(previews: &mut Module) {
    let request = |instance: EffectInstance, frame_count: u32| PreviewRequest {
        instance,
        led_count: 6,
        start_ms: 0,
        frame_count,
        interval_ms: 20,
    };
    let batch = |request| preview_effect(request).expect("preview");
    previews.add("rainbowPreview", &batch(request(rainbow(), 3)));
    previews.add(
        "solidPreview",
        &batch(request(
            effect(
                "solid",
                &[("color", ControlValue::Color(Rgb::new(0, 200, 120)))],
            ),
            30,
        )),
    );
}

fn restore<T>(result: Result<T, OutputError>) -> T {
    result.expect("stop outcome")
}

fn command_error<T: std::fmt::Debug>(result: Result<T, OutputError>) -> CommandError {
    CommandError::from(result.expect_err("an error"))
}

/// A demo batch with every kind of result, and the stop and stop-all reports that follow it.
pub async fn batches(batches: &mut Module, sessions: &mut Module) {
    let (handle, _) = demo_connected().await;
    handle
        .declare_external(DeviceRef::new(MEMORY_1), PLUGIN.into())
        .await
        .expect("declare");
    let report = handle
        .apply_batch(
            vec![
                whole(GPU),
                whole(MEMORY_1),
                whole(MEMORY_2),
                zone(MOTHERBOARD, 1),
                whole(MOTHERBOARD),
                whole(LIGHT_BAR),
            ],
            rainbow(),
        )
        .await
        .expect("batch");
    batches.add("batchAppliedWithSkips", &report);
    wait_until(&handle, |snapshot| {
        condition_of(snapshot, GPU) == Some(OutputCondition::Running)
    })
    .await;
    sessions.add("demoBatchRunning", &handle.snapshot());
    let stopped = handle
        .stop_batch(vec![whole(GPU), whole(MEMORY_1)])
        .await
        .expect("stop");
    batches.add("batchStopped", &stopped);
    let all = handle.stop_all_effects().await.expect("stop all");
    batches.add("stopAllReport", &all);
}

/// Live output of a real OpenRGB session with a keyboard matrix and a strip, and the session beside it.
pub async fn live(live: &mut Module, sessions: &mut Module) {
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("keys", "Fixture Keyboard")
            .serial("SN-keys")
            .location("usb:keys")
            .matrix_zone("Keys", 4, 12),
    );
    server.add_device(1, breathing_strip("desk", "Fixture Desk Strip"));
    let handle = openrgb(&server);
    handle.connect().await.expect("connect");
    let snapshot = handle.snapshot();
    let keys = reference(&snapshot, "SN-keys");
    let desk = reference(&snapshot, "SN-desk");
    let targets: Vec<OutputTarget> = [keys, desk]
        .into_iter()
        .map(|device| OutputTarget { device, zone: None })
        .collect();
    handle
        .apply_batch(targets.clone(), rainbow())
        .await
        .expect("batch");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let mut feed = handle.live_output(targets);
    live.add(
        "liveKeyboardAndStrip",
        &feed.next().await.expect("live output"),
    );
    sessions.add("openRgbKeyboardRunning", &handle.snapshot());
}
