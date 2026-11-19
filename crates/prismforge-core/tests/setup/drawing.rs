use prismforge_core::setup::{
    Frame, Outline, OutlineId, OutlineKind, Setup, SetupProblem, ViewId, plan_across,
};
use prismforge_effects::Flow;
use serde_json::Map;

use crate::support::{board, component_id, mirrored_fans, placed, view, view_id};

const RIGHT: Flow = Flow::Along { angle: 0 };

fn outline(id: &str, kind: OutlineKind) -> Outline {
    Outline {
        id: OutlineId::parse(id).expect("outline id"),
        name: "Case".into(),
        kind,
        x: 300,
        y: 300,
        width: 400,
        height: 560,
        rotation: 0,
        extra: Map::new(),
    }
}

fn placed_fans() -> Setup {
    let mut setup = mirrored_fans();
    setup.views = vec![view(
        "v-pc",
        vec![
            placed("c-fan-a", 100, 100, 80),
            placed("c-fan-b", 100, 200, 80),
            placed("c-fan-c", 300, 100, 80),
        ],
    )];
    setup
}

fn pc() -> ViewId {
    view_id("v-pc")
}

#[test]
fn outlines_and_frames_change_no_led_and_no_output() {
    let plain = placed_fans();
    let mut drawn = placed_fans();
    drawn.components[0].frame = Some(Frame {
        width: 160,
        height: 60,
        extra: Map::new(),
    });
    drawn.views[0].outlines = vec![
        outline("u-case", OutlineKind::Case),
        outline("u-desk", OutlineKind::Desk),
    ];

    let before = plan_across(&plain, &pc(), RIGHT, &[board()]).expect("plan");
    let after = plan_across(&drawn, &pc(), RIGHT, &[board()]).expect("plan");

    assert_eq!(drawn.validate(), Ok(()));
    assert_eq!(after, before);
    assert_eq!(
        drawn.parts(&drawn.outputs[0]),
        plain.parts(&plain.outputs[0])
    );
}

#[test]
fn an_outline_needs_a_size_and_a_turn_below_360_degrees() {
    for broken in [
        Outline {
            width: 0,
            ..outline("u-case", OutlineKind::Case)
        },
        Outline {
            rotation: 360,
            ..outline("u-case", OutlineKind::Monitor)
        },
    ] {
        let mut setup = placed_fans();
        setup.views[0].outlines = vec![broken];

        assert_eq!(
            setup.validate(),
            Err(SetupProblem::InvalidOutline {
                view: pc(),
                outline: OutlineId::parse("u-case").expect("id"),
            })
        );
    }
}

#[test]
fn a_frame_is_10_to_1000_percent_of_the_leds() {
    for (width, height, valid) in [
        (10, 10, true),
        (1000, 1000, true),
        (9, 60, false),
        (160, 1001, false),
    ] {
        let mut setup = placed_fans();
        setup.components[1].frame = Some(Frame {
            width,
            height,
            extra: Map::new(),
        });

        let expected = if valid {
            Ok(())
        } else {
            Err(SetupProblem::InvalidGeometry {
                component: component_id("c-fan-b"),
            })
        };
        assert_eq!(setup.validate(), expected, "{width} x {height}");
    }
}

#[test]
fn an_outline_id_is_unique_across_the_whole_setup() {
    let mut setup = placed_fans();
    let mut desk = view("v-desk", Vec::new());
    desk.outlines = vec![outline("u-case", OutlineKind::Pc)];
    setup.views[0].outlines = vec![outline("u-case", OutlineKind::Case)];
    setup.views.push(desk);

    assert_eq!(
        setup.validate(),
        Err(SetupProblem::DuplicateId {
            id: "u-case".into()
        })
    );
}
