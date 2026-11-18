mod support;

use std::time::Duration;

use prismforge_effects::{
    EffectInstance, InstanceError, PREVIEW_LIMITS, PreviewRequest, ValueError, ValueProblem,
    linear_positions, prepare, preview,
};

use support::{RED, color, instance, number};

fn request(
    instance: EffectInstance,
    led_count: u32,
    frame_count: u32,
    interval_ms: u32,
) -> PreviewRequest {
    PreviewRequest {
        instance,
        led_count,
        start_ms: 2_500,
        frame_count,
        interval_ms,
    }
}

fn rainbow() -> EffectInstance {
    instance("rainbow-wave", &[("speed", number(90))])
}

#[test]
fn frames_equal_the_strip_the_engine_writes_at_the_same_times() {
    let batch = preview(&request(rainbow(), 16, 12, 40)).expect("valid");
    let effect = prepare(&rainbow()).expect("valid");

    assert_eq!(batch.frames.len(), 12);
    assert!(!batch.is_static);
    for (index, frame) in batch.frames.iter().enumerate() {
        let time = Duration::from_millis(2_500 + 40 * index as u64);
        assert_eq!(frame, &effect.render_strip(time, 16), "frame {index}");
    }
}

#[test]
fn the_strip_renderer_is_the_linear_render_with_seed_zero() {
    let effect = prepare(&rainbow()).expect("valid");
    let time = Duration::from_millis(1_234);

    assert_eq!(
        effect.render_strip(time, 9),
        effect.render(time, &linear_positions(9), 0)
    );
}

#[test]
fn led_count_frame_count_and_interval_are_clamped() {
    let limits = PREVIEW_LIMITS;

    let small = preview(&request(rainbow(), 0, 0, 0)).expect("valid");
    assert_eq!(small.led_count, limits.leds.min);
    assert_eq!(small.frames.len() as u32, limits.frames.min);
    assert_eq!(small.interval_ms, limits.interval_ms.min);
    assert!(
        small
            .frames
            .iter()
            .all(|frame| frame.len() as u32 == limits.leds.min)
    );

    let large = preview(&request(rainbow(), 100_000, 100_000, u32::MAX)).expect("valid");
    assert_eq!(large.led_count, limits.leds.max);
    assert_eq!(large.frames.len() as u32, limits.frames.max);
    assert_eq!(large.interval_ms, limits.interval_ms.max);
    assert!(
        large
            .frames
            .iter()
            .all(|frame| frame.len() as u32 == limits.leds.max)
    );
    assert_eq!(large.start_ms, 2_500);
}

#[test]
fn a_static_effect_gets_a_single_frame() {
    let solid = instance("solid", &[("color", color(RED))]);

    let batch = preview(&request(solid, 4, 30, 50)).expect("valid");

    assert!(batch.is_static);
    assert_eq!(batch.frames, [vec![RED; 4]]);
}

#[test]
fn an_invalid_instance_returns_the_structured_validation_error() {
    let invalid = instance("breathing", &[("speed", number(9_999))]);

    assert_eq!(
        preview(&request(invalid, 8, 4, 50)).map(|_| ()),
        Err(InstanceError::InvalidValues {
            errors: vec![ValueError {
                control: "speed".into(),
                problem: ValueProblem::OutOfRange { min: 0, max: 120 },
            }]
        })
    );
}
