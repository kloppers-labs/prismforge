mod support;

use std::time::Duration;

use prismforge_effects::{ControlValue, Direction, Flow, Plane, Rgb, ScenePoint};

use support::{BLACK, BLUE, GREEN, RED, SEED, WHITE, color, effect, every_effect, number, palette};

const WIDE: Plane = Plane {
    width: 1.0,
    height: 0.5,
};
const SQUARE: Plane = Plane {
    width: 1.0,
    height: 1.0,
};
const RIGHT: Flow = Flow::Along { angle: 0 };
const DOWN: Flow = Flow::Along { angle: 90 };
const LEFT: Flow = Flow::Along { angle: 180 };

fn at(x: f64, y: f64) -> Option<ScenePoint> {
    Some(ScenePoint { x, y, spacing: 0.0 })
}

fn across(
    effect: &prismforge_effects::Effect,
    flow: Flow,
    plane: Plane,
    points: &[Option<ScenePoint>],
) -> Vec<Rgb> {
    effect.render_across(Duration::ZERO, flow, plane, points)
}

#[test]
fn a_gradient_runs_across_the_plane_in_the_flow_direction() {
    let gradient = effect("gradient", &[("palette", palette(&[RED, GREEN, BLUE]))]);
    let row = [at(-0.5, 0.0), at(0.0, 0.0), at(0.5, 0.0)];
    let column = [at(0.0, -0.25), at(0.0, 0.25)];

    assert_eq!(across(&gradient, RIGHT, WIDE, &row), vec![RED, GREEN, BLUE]);
    assert_eq!(across(&gradient, LEFT, WIDE, &row), vec![BLUE, GREEN, RED]);
    assert_eq!(across(&gradient, DOWN, WIDE, &column), vec![RED, BLUE]);
}

#[test]
fn a_flow_from_a_centre_grows_outwards() {
    let gradient = effect("gradient", &[("palette", palette(&[RED, BLUE]))]);
    let middle = Flow::FromCentre { x: 50, y: 50 };
    let corner = Flow::FromCentre { x: 0, y: 0 };
    let points = [at(0.0, 0.0), at(0.5, 0.5), at(-0.5, -0.5)];

    assert_eq!(
        across(&gradient, middle, SQUARE, &points),
        vec![RED, BLUE, BLUE]
    );
    assert_eq!(
        across(&gradient, corner, SQUARE, &points),
        vec![Rgb::new(128, 0, 128), BLUE, RED]
    );
}

#[test]
fn leds_without_a_place_stay_black() {
    for effect in every_effect(&[]) {
        let colors = across(&effect, RIGHT, WIDE, &[None, at(0.0, 0.0), None]);

        assert_eq!(colors[0], BLACK, "{}", effect.instance().effect);
        assert_eq!(colors[2], BLACK, "{}", effect.instance().effect);
    }
}

#[test]
fn a_flow_samples_the_same_colours_as_a_line_of_leds() {
    let backward = ("direction", ControlValue::Direction(Direction::Backward));
    let effects = [
        effect(
            "gradient",
            &[("palette", palette(&[RED, GREEN, WHITE])), backward.clone()],
        ),
        effect(
            "rainbow-wave",
            &[("speed", number(45)), ("waves", number(3))],
        ),
        effect("breathing", &[("speed", number(30))]),
        effect("color-cycle", &[("speed", number(90))]),
        effect(
            "solid",
            &[
                ("color", color(Rgb::new(9, 8, 7))),
                ("brightness", number(40)),
            ],
        ),
    ];
    let along: Vec<f64> = (0..=8).map(|step| f64::from(step) / 8.0).collect();
    let points: Vec<Option<ScenePoint>> = along.iter().map(|&s| at(s - 0.5, 0.1)).collect();

    for effect in &effects {
        for millis in [0, 700, 12_345] {
            let time = Duration::from_millis(millis);

            assert_eq!(
                effect.render_across(time, RIGHT, WIDE, &points),
                effect.render(time, &along, SEED),
                "{} at {millis} ms",
                effect.instance().effect
            );
        }
    }
}

#[test]
fn a_moving_highlight_travels_along_the_flow_and_is_at_least_one_spacing_wide() {
    let highlight = effect(
        "moving-highlight",
        &[
            ("color", color(WHITE)),
            ("background", color(BLACK)),
            ("speed", number(60)),
            ("width", number(5)),
        ],
    );
    let spaced = |x| {
        Some(ScenePoint {
            x,
            y: 0.0,
            spacing: 0.25,
        })
    };
    let points = [spaced(0.0), spaced(0.125), spaced(0.25), spaced(-0.5)];

    let half_round = highlight.render_across(Duration::from_millis(500), RIGHT, WIDE, &points);
    let start = highlight.render_across(Duration::ZERO, RIGHT, WIDE, &points);

    assert_eq!(
        half_round,
        vec![WHITE, Rgb::new(128, 128, 128), BLACK, BLACK]
    );
    assert_ne!(start[3], BLACK, "the band enters at the first end");
    assert_eq!(&start[..3], &[BLACK; 3]);
}

#[test]
fn the_picture_does_not_depend_on_the_scale_of_the_plane() {
    let gradient = effect("gradient", &[("palette", palette(&[RED, GREEN, BLUE]))]);
    let small = [at(-0.3, 0.1), at(0.2, -0.2)];
    let large = [at(-0.6, 0.2), at(0.4, -0.4)];
    let double = Plane {
        width: 2.0,
        height: 1.0,
    };
    let flow = Flow::Along { angle: 30 };

    assert_eq!(
        across(&gradient, flow, WIDE, &small),
        across(&gradient, flow, double, &large)
    );
}

#[test]
fn only_effects_that_change_along_the_leds_follow_a_position() {
    let following: Vec<String> = every_effect(&[])
        .iter()
        .filter(|effect| effect.follows_position())
        .map(|effect| effect.instance().effect.clone())
        .collect();

    assert_eq!(following, ["gradient", "rainbow-wave", "moving-highlight"]);
}

#[test]
fn a_flow_keeps_its_stored_form() {
    let along = serde_json::to_string(&Flow::Along { angle: 45 }).expect("encode");
    let centre = serde_json::to_string(&Flow::FromCentre { x: 50, y: 20 }).expect("encode");

    assert_eq!(along, r#"{"Along":{"angle":45}}"#);
    assert_eq!(centre, r#"{"FromCentre":{"x":50,"y":20}}"#);
}

#[test]
fn the_two_ends_of_the_plane_never_light_together() {
    let highlight = effect(
        "moving-highlight",
        &[("speed", number(60)), ("width", number(10))],
    );
    let ends = [at(-0.5, 0.0), at(0.5, 0.0)];

    for millis in 0..1_000 {
        let colors = highlight.render_across(Duration::from_millis(millis), RIGHT, WIDE, &ends);

        assert!(
            colors[0] == BLACK || colors[1] == BLACK,
            "both ends lit at {millis} ms: {colors:?}"
        );
    }
}
