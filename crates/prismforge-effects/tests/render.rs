mod support;

use std::time::Duration;

use prismforge_effects::{ControlValue, Direction, Rgb, linear_positions};

use support::{
    BLACK, BLUE, GREEN, RED, SEED, WHITE, color, effect, every_effect, frame, number, palette,
};

const LEDS: usize = 5;

fn direction(value: Direction) -> ControlValue {
    ControlValue::Direction(value)
}

#[test]
fn solid_colour_fills_every_led_and_never_changes() {
    let solid = effect("solid", &[("color", color(Rgb::new(10, 20, 30)))]);

    assert_eq!(frame(&solid, 0, 4), vec![Rgb::new(10, 20, 30); 4]);
    assert_eq!(frame(&solid, 3_600_000, 4), frame(&solid, 0, 4));
}

#[test]
fn gradient_starts_on_the_first_colour_and_ends_on_the_last() {
    let gradient = effect("gradient", &[("palette", palette(&[RED, GREEN, BLUE]))]);

    let colors = frame(&gradient, 0, LEDS);

    assert_eq!(colors[0], RED);
    assert_eq!(colors[1], Rgb::new(128, 128, 0));
    assert_eq!(colors[2], GREEN);
    assert_eq!(colors[4], BLUE);
}

#[test]
fn colour_cycle_wraps_from_the_last_colour_back_to_the_first() {
    let cycle = effect(
        "color-cycle",
        &[
            ("palette", palette(&[RED, GREEN, BLUE, WHITE])),
            ("speed", number(30)),
        ],
    );

    assert_eq!(frame(&cycle, 0, 3), vec![RED; 3]);
    assert_eq!(frame(&cycle, 500, 3), vec![GREEN; 3]);
    assert_eq!(frame(&cycle, 1_750, 3), vec![Rgb::new(255, 128, 128); 3]);
    assert_eq!(frame(&cycle, 2_000, 3), vec![RED; 3]);
}

#[test]
fn breathing_starts_at_full_colour_and_is_dark_at_half_period() {
    let orange = Rgb::new(255, 120, 0);
    let breathing = effect(
        "breathing",
        &[("color", color(orange)), ("speed", number(15))],
    );

    assert_eq!(frame(&breathing, 0, 2), vec![orange; 2]);
    assert_eq!(frame(&breathing, 2_000, 2), vec![BLACK; 2]);
    assert_eq!(frame(&breathing, 4_000, 2), vec![orange; 2]);
}

#[test]
fn rainbow_wave_moves_one_led_forward_per_quarter_cycle() {
    let wave = effect(
        "rainbow-wave",
        &[("speed", number(60)), ("waves", number(1))],
    );

    let start = frame(&wave, 0, LEDS);
    let later = frame(&wave, 250, LEDS);

    assert_eq!(start[0], RED);
    assert_eq!(later[1..], start[..LEDS - 1]);
    assert_eq!(later[0], start[3]);
}

#[test]
fn moving_highlight_lights_the_led_under_its_centre() {
    let highlight = effect(
        "moving-highlight",
        &[
            ("color", color(WHITE)),
            ("background", color(BLUE)),
            ("speed", number(60)),
            ("width", number(20)),
        ],
    );

    assert_eq!(frame(&highlight, 0, LEDS), vec![BLUE; LEDS]);
    assert_eq!(
        frame(&highlight, 500, LEDS),
        vec![BLUE, BLUE, WHITE, BLUE, BLUE]
    );
}

#[test]
fn backward_direction_mirrors_the_forward_pattern() {
    let cases = [
        (
            "gradient",
            vec![("palette", palette(&[RED, GREEN, BLUE, WHITE]))],
        ),
        ("rainbow-wave", vec![("speed", number(45))]),
        (
            "moving-highlight",
            vec![("speed", number(60)), ("width", number(40))],
        ),
    ];
    for (id, values) in cases {
        let mut forward = values.clone();
        forward.push(("direction", direction(Direction::Forward)));
        let mut backward = values;
        backward.push(("direction", direction(Direction::Backward)));

        let mut mirrored = frame(&effect(id, &forward), 350, LEDS);
        mirrored.reverse();

        assert_eq!(frame(&effect(id, &backward), 350, LEDS), mirrored, "{id}");
    }
}

#[test]
fn brightness_zero_gives_black_for_every_effect() {
    for effect in every_effect(&[("brightness", number(0))]) {
        assert_eq!(frame(&effect, 1_234, LEDS), vec![BLACK; LEDS]);
        assert!(effect.is_static());
    }
}

#[test]
fn full_brightness_leaves_colours_unchanged_and_half_brightness_halves_them() {
    let base = Rgb::new(200, 100, 51);
    let at = |brightness| {
        frame(
            &effect(
                "solid",
                &[("color", color(base)), ("brightness", number(brightness))],
            ),
            0,
            1,
        )[0]
    };

    assert_eq!(at(100), base);
    assert_eq!(at(50), Rgb::new(100, 50, 26));
    assert_eq!(at(1), Rgb::new(2, 1, 1));
}

#[test]
fn brightness_is_applied_after_the_effect_colour() {
    let full = frame(&effect("rainbow-wave", &[]), 900, LEDS);
    let dimmed = frame(
        &effect("rainbow-wave", &[("brightness", number(50))]),
        900,
        LEDS,
    );

    let halved: Vec<Rgb> = full
        .iter()
        .map(|c| {
            let half = |channel: u8| ((u32::from(channel) * 50 + 50) / 100) as u8;
            Rgb::new(half(c.r), half(c.g), half(c.b))
        })
        .collect();
    assert_eq!(dimmed, halved);
}

#[test]
fn an_empty_target_gets_an_empty_frame() {
    for effect in every_effect(&[]) {
        assert!(effect.render(Duration::ZERO, &[], SEED).is_empty());
    }
}

#[test]
fn a_single_led_sits_at_the_start_of_the_target() {
    let gradient = effect("gradient", &[("palette", palette(&[RED, BLUE]))]);

    assert_eq!(linear_positions(1), vec![0.0]);
    assert_eq!(frame(&gradient, 0, 1), vec![RED]);
    for effect in every_effect(&[]) {
        assert_eq!(frame(&effect, 777, 1).len(), 1);
    }
}

#[test]
fn linear_positions_run_from_zero_to_one() {
    assert!(linear_positions(0).is_empty());
    assert_eq!(linear_positions(5), vec![0.0, 0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn same_inputs_give_identical_frames() {
    for (first, second) in every_effect(&[]).iter().zip(every_effect(&[])) {
        assert_eq!(frame(first, 4_321, 16), frame(&second, 4_321, 16));
    }
}

#[test]
fn positions_outside_the_target_are_clamped() {
    let gradient = effect("gradient", &[("palette", palette(&[RED, BLUE]))]);

    let colors = gradient.render(Duration::ZERO, &[-3.0, f64::NAN, 9.0], SEED);

    assert_eq!(colors, vec![RED, RED, BLUE]);
}

#[test]
fn animated_effects_change_over_time_and_static_ones_do_not() {
    for effect in every_effect(&[]) {
        let id = effect.instance().effect.clone();
        let changed = frame(&effect, 0, 16) != frame(&effect, 1_100, 16);
        assert_eq!(changed, !effect.is_static(), "{id}");
    }
}

#[test]
fn static_detection_follows_speed_and_effect() {
    let is_static = |id: &str, values: &[(&str, ControlValue)]| effect(id, values).is_static();

    assert!(is_static("solid", &[("color", color(RED))]));
    assert!(is_static("gradient", &[]));
    assert!(!is_static("breathing", &[]));
    assert!(is_static("breathing", &[("speed", number(0))]));
    assert!(!is_static("rainbow-wave", &[]));
    assert!(is_static("rainbow-wave", &[("speed", number(0))]));
    assert!(is_static("color-cycle", &[("speed", number(0))]));
    assert!(is_static("moving-highlight", &[("speed", number(0))]));
}
