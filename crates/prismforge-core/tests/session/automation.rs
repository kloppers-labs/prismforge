use std::time::Duration;

use prismforge_core::automation::{
    DeviceLightsOff, EventProfile, EventProfiles, LightsOffOutcome, LightsOffSkip, plan_lights_off,
};
use prismforge_core::session::{OutputError, SessionError};
use prismforge_core::{DeviceRef, OutputOwner, Rgb};
use prismforge_testkit::{DemoOutput, FakeServer, FixtureMode, wire};
use tokio::time::sleep;

use crate::support::{
    GPU, config, connected_openrgb, demo, device_frames, direct_mode_calls, output_of,
    output_packets, rainbow, reference, strip, whole,
};

const BLACK: [u8; 3] = [0, 0, 0];

/// A strip with Direct Mode, a card with a harmless "Off" mode, and a keyboard whose modes all save.
fn three_devices() -> FakeServer {
    let server = FakeServer::new();
    server.add_device(
        0,
        strip("strip", 6)
            .modes(vec![FixtureMode::direct(), FixtureMode::breathing()])
            .active_mode(1),
    );
    server.add_device(
        1,
        strip("card", 4).modes(vec![FixtureMode::static_color(), FixtureMode::off()]),
    );
    server.add_device(
        2,
        strip("keys", 8).modes(vec![
            FixtureMode::static_color().saving_automatically(),
            FixtureMode::off().saving_automatically(),
        ]),
    );
    server
}

#[tokio::test(start_paused = true)]
async fn lights_off_writes_black_selects_a_safe_off_mode_and_never_touches_a_saving_keyboard() {
    let server = three_devices();
    let handle = connected_openrgb(&server).await;
    let snapshot = handle.snapshot();
    let (strip, card, keys) = (
        reference(&snapshot, "SN-strip"),
        reference(&snapshot, "SN-card"),
        reference(&snapshot, "SN-keys"),
    );
    let before = output_packets(&server).len();

    let report = handle
        .lights_off(plan_lights_off(&snapshot.devices))
        .await
        .expect("lights off");

    let outcome = |device: &DeviceRef| {
        report
            .devices
            .iter()
            .find(|entry| entry.device.as_ref() == Some(device))
            .map(|entry| entry.outcome.clone())
    };
    assert_eq!(outcome(&strip), Some(LightsOffOutcome::Black));
    assert_eq!(
        outcome(&card),
        Some(LightsOffOutcome::OffMode {
            mode: 1,
            name: "Off".into()
        })
    );
    assert_eq!(
        outcome(&keys),
        Some(LightsOffOutcome::Skipped {
            reason: LightsOffSkip::OffModeSavesToDevice
        })
    );
    let sent = &output_packets(&server)[before..];
    let targets: Vec<(u32, String)> = sent
        .iter()
        .map(|packet| (packet.packet_id, packet.target.clone().unwrap_or_default()))
        .collect();
    assert_eq!(
        targets,
        [
            (wire::SET_CUSTOM_MODE, "strip".into()),
            (wire::UPDATE_LEDS, "strip".into()),
            (wire::UPDATE_MODE, "card".into()),
        ]
    );
    assert_eq!(sent[1].color_write().expect("colours").colors, [BLACK; 6]);
    assert_eq!(sent[2].mode_update().expect("mode").mode_index, 1);
    assert!(server.received_with_id(wire::SAVE_MODE).is_empty());
}

#[tokio::test(start_paused = true)]
async fn lights_off_ends_effects_without_a_restore_and_a_disconnect_keeps_them_dark() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(100)).await;
    let plan = plan_lights_off(&handle.snapshot().devices);
    let report = handle.lights_off(plan).await.expect("lights off");
    assert!(report.devices.iter().any(|entry| {
        entry.device == Some(DeviceRef::new(GPU)) && entry.outcome == LightsOffOutcome::Black
    }));
    assert!(handle.snapshot().output.scene.assignments.is_empty());
    let after_off = controls.outputs().len();
    sleep(Duration::from_millis(200)).await;
    handle.disconnect().await.expect("disconnect");

    let later = &controls.outputs()[after_off..];
    assert!(
        later
            .iter()
            .all(|output| !matches!(output, DemoOutput::ModeReapplied { .. })),
        "{later:?}"
    );
    let frames = device_frames(&controls.outputs()[..after_off], GPU);
    assert_eq!(
        frames.last().expect("black frame"),
        &vec![Rgb::new(0, 0, 0); frames.last().expect("frame").len()]
    );
}

#[tokio::test(start_paused = true)]
async fn lights_off_skips_a_device_the_user_gave_to_another_program() {
    let (handle, controls) = demo(config()).await;
    handle
        .declare_external(DeviceRef::new(GPU), "Plugin".into())
        .await
        .expect("declare");
    let plan = plan_lights_off(&handle.snapshot().devices);
    let report = handle.lights_off(plan).await.expect("lights off");
    let gpu = report
        .devices
        .iter()
        .find(|entry| entry.device == Some(DeviceRef::new(GPU)))
        .cloned();
    assert_eq!(
        gpu,
        Some(DeviceLightsOff {
            device: Some(DeviceRef::new(GPU)),
            name: "Demo Graphics Card".into(),
            outcome: LightsOffOutcome::Skipped {
                reason: LightsOffSkip::DeclaredExternal {
                    label: "Plugin".into()
                }
            },
        })
    );
    assert!(device_frames(&controls.outputs(), GPU).is_empty());
    assert_eq!(
        output_of(&handle.snapshot(), GPU).map(|output| output.owner),
        Some(OutputOwner::External {
            label: "Plugin".into()
        })
    );
}

#[tokio::test(start_paused = true)]
async fn keep_lights_leaves_the_last_frame_and_a_disconnect_restores_nothing() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    handle.keep_lights().await.expect("keep");
    assert!(!device_frames(&controls.outputs(), GPU).is_empty());
    assert!(handle.snapshot().output.scene.assignments.is_empty());
    let kept = controls.outputs().len();
    handle.disconnect().await.expect("disconnect");
    assert_eq!(controls.outputs().len(), kept);
}

#[tokio::test(start_paused = true)]
async fn resync_forgets_direct_mode_so_the_next_frame_enables_it_again() {
    let (handle, controls) = demo(config()).await;
    handle
        .apply_effect(whole(GPU), rainbow())
        .await
        .expect("apply");
    sleep(Duration::from_millis(200)).await;
    assert_eq!(direct_mode_calls(&controls.outputs(), GPU), 1);
    handle.resync().await.expect("resync");
    sleep(Duration::from_millis(200)).await;
    assert_eq!(direct_mode_calls(&controls.outputs(), GPU), 2);
}

#[tokio::test(start_paused = true)]
async fn openrgb_event_profiles_are_read_when_the_server_has_a_settings_manager() {
    let server = FakeServer::new();
    server.add_device(0, strip("strip", 6));
    server.set_settings(
        "ProfileManager",
        r#"{"suspend_profile": {"enabled": true, "name": "Dark"}, "resume_profile": {"enabled": false, "name": ""}}"#,
    );
    let handle = connected_openrgb(&server).await;
    let read = handle.event_profiles().await.expect("read");
    assert_eq!(
        read,
        Some(EventProfiles {
            suspend: EventProfile {
                enabled: true,
                name: "Dark".into()
            },
            ..EventProfiles::default()
        })
    );

    server.set_server_flags(0);
    assert_eq!(handle.event_profiles().await, Ok(None));
}

#[tokio::test(start_paused = true)]
async fn demo_mode_cannot_tell_what_openrgb_does_and_a_stopped_session_says_so() {
    let (handle, _) = demo(config()).await;
    assert_eq!(handle.event_profiles().await, Ok(None));
    handle.disconnect().await.expect("disconnect");
    assert_eq!(
        handle.event_profiles().await,
        Err(SessionError::Backend(
            prismforge_core::BackendError::NotConnected
        ))
    );
    assert_eq!(
        handle.keep_lights().await,
        Ok(()),
        "nothing runs, so nothing is kept"
    );
    let refused: Result<_, OutputError> = handle.lights_off(Default::default()).await;
    assert!(refused.is_ok());
}

#[tokio::test(start_paused = true)]
async fn lights_off_that_arrives_after_its_time_writes_nothing() {
    let server = three_devices();
    let handle = connected_openrgb(&server).await;
    let plan = plan_lights_off(&handle.snapshot().devices);
    let before = output_packets(&server).len();
    let past = std::time::SystemTime::now() - Duration::from_secs(1);

    let report = handle
        .lights_off_until(plan, past)
        .await
        .expect("lights off");

    assert!(report.devices.iter().all(|device| matches!(
        device.outcome,
        LightsOffOutcome::Skipped {
            reason: LightsOffSkip::TooLate
        } | LightsOffOutcome::Skipped { .. }
    )));
    assert!(report.devices.iter().any(|device| device.outcome
        == LightsOffOutcome::Skipped {
            reason: LightsOffSkip::TooLate
        }));
    assert_eq!(output_packets(&server).len(), before);
}
