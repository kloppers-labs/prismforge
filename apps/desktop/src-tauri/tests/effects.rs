mod support;

use std::time::Duration;

use prismforge_core::session::{OutputCondition, RestoreOutcome, SessionConfig, SessionError};
use prismforge_core::{DeviceRef, OutputOwner, OutputTarget};
use prismforge_desktop::{CommandError, Shell, ShellConfig, preview_effect};
use prismforge_effects::{
    ControlValue, EffectInstance, InstanceError, PREVIEW_LIMITS, PreviewRequest, ValueError,
    ValueProblem,
};
use prismforge_testkit::{DemoBackend, DemoControls};

use support::temp::TempDir;
use support::{Recorder, is_connected, stores};

const GPU: &str = "demo-graphics-card";

fn instance(effect: &str, values: &[(&str, ControlValue)]) -> EffectInstance {
    EffectInstance {
        effect: effect.into(),
        values: values
            .iter()
            .map(|(id, value)| ((*id).to_owned(), value.clone()))
            .collect(),
    }
}

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

async fn demo_shell(config: ShellConfig) -> (Shell<Recorder>, Recorder, DemoControls, TempDir) {
    let recorder = Recorder::default();
    let (stores, dir) = stores("effects");
    let shell = Shell::new(recorder.clone(), config, stores);
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_connected).await;
    (shell, recorder, controls, dir)
}

async fn demo() -> (Shell<Recorder>, Recorder, DemoControls, TempDir) {
    demo_shell(ShellConfig::default()).await
}

/// Waits past the view spacing, so the view holds every change made so far.
async fn owner(shell: &Shell<Recorder>, device: &str) -> Option<OutputOwner> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    shell
        .view()
        .session
        .expect("session")
        .output
        .devices
        .into_iter()
        .find(|output| output.device.as_str() == device)
        .map(|output| output.owner)
}

#[tokio::test(start_paused = true)]
async fn the_catalog_lists_every_effect_with_the_frame_rate_and_preview_limits() {
    let recorder = Recorder::default();
    let (stores, _idle_dir) = stores("idle");
    let idle = Shell::new(recorder, ShellConfig::default(), stores);
    assert_eq!(idle.catalog().await, Err(CommandError::NoSession));

    let (shell, _, _, _dir) = demo().await;
    let catalog = shell.catalog().await.expect("catalog");

    let ids: Vec<&str> = catalog
        .effects
        .iter()
        .map(|effect| effect.id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "solid",
            "breathing",
            "gradient",
            "rainbow-wave",
            "color-cycle",
            "moving-highlight"
        ]
    );
    assert_eq!((catalog.frame_rate.min, catalog.frame_rate.max), (1, 60));
    assert_eq!(catalog.preview, PREVIEW_LIMITS);
}

#[tokio::test(start_paused = true)]
async fn apply_and_stop_reach_the_session() {
    let (shell, recorder, _, _dir) = demo().await;

    shell
        .apply_effect(whole(GPU), instance("rainbow-wave", &[]))
        .await
        .expect("apply");
    let running = recorder
        .wait_for(|snapshot| !snapshot.output.scene.assignments.is_empty())
        .await;
    let stopped = shell.stop_effect(whole(GPU)).await;

    let session = running.session.expect("session");
    assert_eq!(session.output.scene.assignments[0].target, whole(GPU));
    assert_eq!(
        session.output.devices[0].condition,
        OutputCondition::Running
    );
    assert_eq!(stopped, Ok(Some(RestoreOutcome::Restored)));
}

#[tokio::test(start_paused = true)]
async fn invalid_values_come_back_as_structured_errors() {
    let (shell, _, _, _dir) = demo().await;

    let applied = shell
        .apply_effect(
            whole(GPU),
            instance("breathing", &[("speed", ControlValue::Number(500))]),
        )
        .await;

    assert_eq!(
        applied,
        Err(CommandError::InvalidEffect(InstanceError::InvalidValues {
            errors: vec![ValueError {
                control: "speed".into(),
                problem: ValueProblem::OutOfRange { min: 0, max: 120 },
            }]
        }))
    );
}

#[tokio::test(start_paused = true)]
async fn stopping_a_target_without_an_effect_names_the_target() {
    let (shell, _, _, _dir) = demo().await;

    assert_eq!(
        shell.stop_effect(whole(GPU)).await,
        Err(CommandError::NoEffect { target: whole(GPU) })
    );
}

#[tokio::test(start_paused = true)]
async fn a_declared_device_refuses_effects_until_it_is_handed_back() {
    let (shell, _, controls, _dir) = demo().await;
    let gpu = DeviceRef::new(GPU);

    shell
        .declare_external(gpu.clone(), "  Effects plugin  ".into())
        .await
        .expect("declare");
    let refused = shell
        .apply_effect(whole(GPU), instance("rainbow-wave", &[]))
        .await;
    let declared = owner(&shell, GPU).await;
    shell.clear_external(gpu.clone()).await.expect("hand back");
    let applied = shell
        .apply_effect(whole(GPU), instance("rainbow-wave", &[]))
        .await;

    let external = OutputOwner::External {
        label: "Effects plugin".into(),
    };
    assert_eq!(declared, Some(external.clone()));
    assert_eq!(
        refused,
        Err(CommandError::Session(SessionError::OutputConflict {
            device: gpu,
            owner: external,
        }))
    );
    assert_eq!(applied, Ok(()));
    assert!(!controls.outputs().is_empty());
}

#[tokio::test(start_paused = true)]
async fn the_program_name_is_trimmed_bounded_and_never_empty() {
    let (shell, _, _, _dir) = demo().await;
    let gpu = DeviceRef::new(GPU);
    let memory = DeviceRef::new("demo-memory-1");

    shell
        .declare_external(gpu, "   ".into())
        .await
        .expect("declare");
    shell
        .declare_external(memory, "x".repeat(500))
        .await
        .expect("declare");

    assert_eq!(
        owner(&shell, GPU).await,
        Some(OutputOwner::External {
            label: "another program".into()
        })
    );
    assert_eq!(
        owner(&shell, "demo-memory-1").await,
        Some(OutputOwner::External {
            label: "x".repeat(60)
        })
    );
}

#[tokio::test(start_paused = true)]
async fn hardware_mode_and_frame_rate_reach_the_session() {
    let (shell, _, _, _dir) = demo().await;

    shell
        .select_mode(DeviceRef::new(GPU), 1)
        .await
        .expect("select mode");
    let rate = shell.set_frame_rate(500).await;

    assert_eq!(
        owner(&shell, GPU).await,
        Some(OutputOwner::HardwareMode { mode: 1 })
    );
    assert_eq!(rate, Ok(60));
}

#[tokio::test(start_paused = true)]
async fn a_full_queue_answers_busy_for_effect_commands() {
    let (shell, _, controls, _dir) = demo_shell(ShellConfig {
        session: SessionConfig {
            command_capacity: 1,
            ..SessionConfig::default()
        },
        ..ShellConfig::default()
    })
    .await;
    controls.set_call_delay(Duration::from_secs(10));

    let (_, second) = tokio::join!(shell.refresh(), async {
        shell
            .apply_effect(whole(GPU), instance("rainbow-wave", &[]))
            .await
    });

    assert_eq!(second, Err(CommandError::Session(SessionError::Busy)));
}

#[tokio::test(start_paused = true)]
async fn preview_needs_no_session_and_sends_nothing_to_a_backend() {
    let (shell, _, controls, _dir) = demo().await;
    let before = shell.view().session.expect("session");
    let request = PreviewRequest {
        instance: instance("moving-highlight", &[]),
        led_count: 12,
        start_ms: 0,
        frame_count: 30,
        interval_ms: 33,
    };

    let batch = preview_effect(request.clone()).expect("preview");
    let invalid = preview_effect(PreviewRequest {
        instance: instance("sparkle", &[]),
        ..request
    });
    tokio::time::sleep(Duration::from_secs(1)).await;

    assert_eq!(batch.frames.len(), 30);
    assert_eq!(
        invalid,
        Err(CommandError::InvalidEffect(InstanceError::UnknownEffect {
            effect: "sparkle".into()
        }))
    );
    assert!(controls.outputs().is_empty());
    let after = shell.view().session.expect("session");
    assert_eq!(after.output, before.output);
    assert_eq!(after.diagnostics.counters, before.diagnostics.counters);
}
