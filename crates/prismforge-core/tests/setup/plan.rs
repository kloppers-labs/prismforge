use std::time::{Duration, Instant};

use prismforge_core::session::SessionDevice;
use prismforge_core::setup::{
    AcrossPlan, Corner, Geometry, Matrix, OutputStatus, PlacedTarget, PlanError, Runs, Setup,
    SetupProblem, plan_across,
};
use prismforge_core::store::MemberState;
use prismforge_core::{DeviceIdentity, DeviceRef, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, EffectInstance, Flow, prepare};
use serde_json::Map;

use crate::support::{
    BOARD, HEADER_1, HEADER_2, HEADER_LEDS, LOGO, STRIP, board, chained_fans, component,
    component_id, mirrored_fans, output, output_id, placed, point, ring, strip, view, view_id,
};

const RED: Rgb = Rgb::new(255, 0, 0);
const BLUE: Rgb = Rgb::new(0, 0, 255);
const BLACK: Rgb = Rgb::new(0, 0, 0);
const RIGHT: Flow = Flow::Along { angle: 0 };

fn gradient() -> prismforge_effects::Effect {
    prepare(&EffectInstance {
        effect: "gradient".into(),
        values: [("palette".to_owned(), ControlValue::Palette(vec![RED, BLUE]))].into(),
    })
    .expect("valid")
}

fn render(plan: &AcrossPlan, target: &PlacedTarget) -> Vec<Rgb> {
    gradient().render_across(Duration::ZERO, plan.flow(), plan.plane(), &target.points)
}

fn target(plan: &AcrossPlan, zone: usize) -> &PlacedTarget {
    plan.targets
        .iter()
        .find(|placed| placed.target.zone == Some(zone))
        .expect("planned target")
}

/// An 8-LED strip on the board's fixed strip zone, left, and a 36-LED strip on header 1, right.
fn two_strips() -> Setup {
    Setup {
        components: vec![
            component("c-left", strip(8)),
            component("c-right", strip(36)),
        ],
        outputs: vec![
            output("o-left", STRIP, 0, &[&["c-left"]]),
            output("o-right", HEADER_1, 0, &[&["c-right"]]),
        ],
        views: vec![view(
            "v-pc",
            vec![
                placed("c-left", 50, 20, 80),
                placed("c-right", 200, 20, 180),
            ],
        )],
        extra: Map::new(),
    }
}

fn reds(colors: &[Rgb]) -> Vec<u8> {
    colors.iter().map(|color| color.r).collect()
}

#[test]
fn samples_effects_at_scene_positions() {
    let setup = two_strips();
    setup.validate().expect("valid");
    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");

    let left = render(&plan, target(&plan, STRIP));
    let right = render(&plan, target(&plan, HEADER_1));
    let picture: Vec<u8> = reds(&left).into_iter().chain(reds(&right)).collect();

    assert_eq!(plan.targets.len(), 2);
    assert_eq!(left.len(), 8);
    assert_eq!(right.len(), HEADER_LEDS);
    assert_eq!(left[0], RED);
    assert_eq!(right[HEADER_LEDS - 1], BLUE);
    assert!(
        picture.windows(2).all(|pair| pair[0] > pair[1]),
        "one picture from left to right: {picture:?}"
    );
    assert!(
        left[7].r > right[0].r + 20,
        "the gap between the strips stays in the picture"
    );
}

#[test]
fn a_flipped_component_shows_the_picture_in_its_own_led_order() {
    let mut setup = two_strips();
    setup.views[0].placements[1].flipped = true;
    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");

    let right = render(&plan, target(&plan, HEADER_1));

    assert_eq!(right[0], BLUE);
    assert!(reds(&right).windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn leds_no_component_covers_are_not_placed_and_stay_black() {
    let mut setup = mirrored_fans();
    setup
        .views
        .push(view("v-pc", vec![placed("c-fan-a", 0, 0, 40)]));

    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");
    let hub = target(&plan, HEADER_1);
    let colors = render(&plan, hub);

    assert_eq!(hub.points.len(), HEADER_LEDS);
    assert_eq!(hub.not_placed(), (12..HEADER_LEDS).collect::<Vec<_>>());
    assert!(colors[12..].iter().all(|color| *color == BLACK));
    assert!(colors[..12].iter().any(|color| *color != BLACK));
}

#[test]
fn a_mirrored_output_takes_its_places_from_the_first_placed_branch() {
    let mut setup = mirrored_fans();
    setup.views.push(view(
        "v-pc",
        vec![placed("c-fan-b", 0, 0, 40), placed("c-fan-c", 100, 0, 40)],
    ));
    let mut only_b = setup.clone();
    only_b.views[0].placements.pop();

    let both = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");
    let one = plan_across(&only_b, &view_id("v-pc"), RIGHT, &[board()]).expect("view");

    assert_eq!(both.targets.len(), 1, "one output, one write");
    let hub = target(&both, HEADER_1);
    let leftmost = hub.points[9].expect("placed");
    assert!(
        leftmost.x < 0.0,
        "fan b, the first placed branch, gives the places"
    );
    assert_eq!(
        target(&one, HEADER_1).points[..12].iter().flatten().count(),
        12
    );
}

#[test]
fn a_daisy_chain_places_each_part_from_its_own_component() {
    let mut setup = chained_fans();
    setup.views.push(view(
        "v-pc",
        vec![placed("c-fan-1", 0, 0, 40), placed("c-fan-3", 200, 0, 40)],
    ));

    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");
    let chain = target(&plan, HEADER_2);

    assert_eq!(chain.not_placed(), (12..24).collect::<Vec<_>>());
    assert!(chain.points[0].expect("fan 1").x < chain.points[24].expect("fan 3").x);
}

#[test]
fn outputs_whose_zone_changed_or_no_longer_fits_get_nothing() {
    let mut setup = two_strips();
    setup.components.push(component("c-logo", point()));
    setup
        .outputs
        .push(output("o-logo", LOGO, 0, &[&["c-logo"]]));
    setup.views[0].placements.push(placed("c-logo", 120, 60, 4));
    let mut changed = board();
    changed.descriptor.zones[LOGO].name = "Renamed".into();
    changed.descriptor.zones[HEADER_1].led_count = 20;

    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[changed]).expect("view");

    let planned: Vec<&OutputTarget> = plan.targets.iter().map(|placed| &placed.target).collect();
    let excluded: Vec<(_, _, _)> = plan
        .excluded
        .iter()
        .map(|status: &OutputStatus| (status.output.clone(), status.state, status.problem.clone()))
        .collect();
    assert_eq!(planned, [&output("o-left", STRIP, 0, &[]).target()]);
    assert_eq!(
        excluded,
        [
            (
                output_id("o-right"),
                MemberState::Resolved,
                Some(SetupProblem::ExceedsOutput {
                    output: output_id("o-right"),
                    needed: 36,
                    available: 20,
                })
            ),
            (output_id("o-logo"), MemberState::Changed, None),
        ]
    );
}

#[test]
fn outputs_of_devices_not_found_stay_in_the_plan_for_the_session_to_refuse() {
    let setup = two_strips();
    let mut ambiguous = board();
    ambiguous.identity = DeviceIdentity::Ambiguous {
        candidates: vec![DeviceRef::new(BOARD)],
    };

    let missing = plan_across(&setup, &view_id("v-pc"), RIGHT, &[]).expect("view");
    let unclear = plan_across(&setup, &view_id("v-pc"), RIGHT, &[ambiguous]).expect("view");

    assert_eq!(missing.targets.len(), 2);
    assert_eq!(unclear.targets.len(), 2);
    assert!(missing.excluded.is_empty() && unclear.excluded.is_empty());
}

#[test]
fn the_picture_does_not_change_with_the_scale_of_the_view() {
    let setup = two_strips();
    let mut larger = setup.clone();
    for placement in &mut larger.views[0].placements {
        placement.x *= 3;
        placement.y *= 3;
        placement.size *= 3;
    }

    let small = plan_across(
        &setup,
        &view_id("v-pc"),
        Flow::Along { angle: 20 },
        &[board()],
    );
    let large = plan_across(
        &larger,
        &view_id("v-pc"),
        Flow::Along { angle: 20 },
        &[board()],
    );

    let colors = |plan: &AcrossPlan| -> Vec<Rgb> {
        plan.targets
            .iter()
            .flat_map(|placed| render(plan, placed))
            .collect()
    };
    assert_eq!(colors(&small.expect("view")), colors(&large.expect("view")));
}

#[test]
fn a_view_that_does_not_exist_has_no_plan() {
    assert_eq!(
        plan_across(&two_strips(), &view_id("v-desk"), RIGHT, &[board()]),
        Err(PlanError::UnknownView {
            view: view_id("v-desk"),
        })
    );
}

/// 600 LEDs over 8 outputs: four headers of 75 LEDs on two boards, each a chain of three 25-LED fans.
fn large_setup() -> (Setup, Vec<SessionDevice>) {
    let mut devices = Vec::new();
    let mut setup = Setup::default();
    let mut placements = Vec::new();
    for board_index in 0..2 {
        let mut device = board();
        let reference = DeviceRef::new(format!("fixture-board-{board_index}"));
        device.identity = DeviceIdentity::Matched(reference.clone());
        for zone in [HEADER_1, HEADER_2] {
            device.descriptor.zones[zone].led_count = 75;
        }
        device
            .descriptor
            .zones
            .push(device.descriptor.zones[HEADER_1].clone());
        device
            .descriptor
            .zones
            .push(device.descriptor.zones[HEADER_1].clone());
        for (slot, zone) in [HEADER_1, HEADER_2, 4, 5].into_iter().enumerate() {
            let mut branch = Vec::new();
            for fan in 0..3 {
                let id = format!("c-fan-{board_index}-{slot}-{fan}");
                setup.components.push(component(&id, ring(25)));
                let row = 50 * i32::try_from(slot).expect("small");
                placements.push(placed(&id, fan * 50 + board_index * 160, row, 40));
                branch.push(id);
            }
            let mut wired = output(&format!("o-{board_index}-{slot}"), HEADER_1, 0, &[]);
            wired.device = reference.clone();
            wired.zone = Some(zone);
            wired.zone_name = Some(device.descriptor.zones[zone].name.clone());
            wired.branches = vec![branch.iter().map(|id| component_id(id)).collect()];
            setup.outputs.push(wired);
        }
        device.descriptor.led_count = device
            .descriptor
            .zones
            .iter()
            .map(|zone| zone.led_count)
            .sum();
        devices.push(device);
    }
    setup.views.push(view("v-pc", placements));
    (setup, devices)
}

#[test]
fn sampling_one_frame_of_a_large_setup_fits_the_frame_budget() {
    let (setup, devices) = large_setup();
    setup.validate().expect("valid");
    setup.check_devices(&devices).expect("fits");
    let plan = plan_across(
        &setup,
        &view_id("v-pc"),
        Flow::FromCentre { x: 50, y: 50 },
        &devices,
    )
    .expect("view");
    let highlight = prepare(&EffectInstance {
        effect: "moving-highlight".into(),
        values: Default::default(),
    })
    .expect("valid");
    let leds: usize = plan.targets.iter().map(|placed| placed.points.len()).sum();
    assert_eq!((plan.targets.len(), leds), (8, 600));

    const FRAMES: u32 = 200;
    let started = Instant::now();
    for frame in 0..FRAMES {
        let time = Duration::from_millis(u64::from(frame) * 16);
        for placed in &plan.targets {
            let colors = highlight.render_across(time, plan.flow(), plan.plane(), &placed.points);
            assert_eq!(colors.len(), placed.points.len());
        }
    }
    let per_frame = started.elapsed() / FRAMES;

    let budget = Duration::from_secs(1) / 60;
    assert!(
        per_frame < budget / 8,
        "one frame of 600 LEDs took {per_frame:?}; the frame budget at 60 frames per second is {budget:?}"
    );
}

#[test]
fn every_output_reports_its_state_and_any_problem_with_its_parts() {
    let setup = two_strips();
    let mut smaller = board();
    smaller.descriptor.zones[HEADER_1].led_count = 30;

    let statuses = prismforge_core::setup::resolve_outputs(&setup, &[smaller]);

    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[0].state, MemberState::Resolved);
    assert_eq!(statuses[0].problem, None);
    assert_eq!(
        statuses[1].problem,
        Some(SetupProblem::ExceedsOutput {
            output: output_id("o-right"),
            needed: 36,
            available: 30,
        })
    );
}

fn highlight() -> prismforge_effects::Effect {
    prepare(&EffectInstance {
        effect: "moving-highlight".into(),
        values: [
            ("speed".to_owned(), ControlValue::Number(60)),
            ("width".to_owned(), ControlValue::Number(5)),
            ("background".to_owned(), ControlValue::Color(BLACK)),
        ]
        .into(),
    })
    .expect("valid")
}

/// The most and the fewest LEDs of `placed` the highlight lights in any frame over one round.
fn lit_range(plan: &AcrossPlan, placed: &PlacedTarget) -> (usize, usize) {
    let effect = highlight();
    let counts: Vec<usize> = (0..100)
        .map(|step| {
            let time = Duration::from_millis(step * 10);
            effect
                .render_across(time, plan.flow(), plan.plane(), &placed.points)
                .iter()
                .zip(placed.points.iter())
                .filter(|(color, point)| point.is_some() && **color != BLACK)
                .count()
        })
        .collect();
    (
        counts.iter().copied().max().unwrap_or(0),
        counts.iter().copied().min().unwrap_or(0),
    )
}

#[test]
fn a_progressive_matrix_keeps_the_highlight_a_band() {
    let setup = Setup {
        components: vec![component(
            "c-panel",
            Geometry::Matrix(Matrix {
                columns: 6,
                rows: 4,
                first: Corner::TopLeft,
                runs: Runs::AlongRows,
                serpentine: false,
                extra: Map::new(),
            }),
        )],
        outputs: vec![output("o-panel", HEADER_1, 0, &[&["c-panel"]])],
        views: vec![view("v-pc", vec![placed("c-panel", 0, 0, 120)])],
        extra: Map::new(),
    };
    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");

    let (most, _) = lit_range(&plan, target(&plan, HEADER_1));

    assert!(
        most <= 12,
        "the band lights at most two columns, not {most} of 24 LEDs"
    );
    assert!(most >= 4, "the band lights at least one column");
}

#[test]
fn a_sparse_component_does_not_widen_the_band_of_a_dense_one() {
    let setup = Setup {
        components: vec![
            component("c-dense", strip(36)),
            component("c-sparse", strip(2)),
        ],
        outputs: vec![
            output("o-dense", HEADER_1, 0, &[&["c-dense"]]),
            output("o-sparse", STRIP, 0, &[&["c-sparse"]]),
        ],
        views: vec![view(
            "v-pc",
            vec![
                placed("c-dense", 50, 0, 100),
                placed("c-sparse", 160, 0, 100),
            ],
        )],
        extra: Map::new(),
    };
    let plan = plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]).expect("view");

    let (dense, _) = lit_range(&plan, target(&plan, HEADER_1));
    let (sparse, _) = lit_range(&plan, target(&plan, STRIP));

    assert!(
        dense <= 6,
        "the dense strip shows a narrow band, not {dense} of 36 LEDs"
    );
    assert!(
        sparse >= 1,
        "the sparse strip still lights as the band passes"
    );
}

#[test]
fn a_plan_refuses_a_setup_that_is_not_valid() {
    let mut setup = two_strips();
    setup.components[1].geometry = Geometry::Line(prismforge_core::setup::Line {
        leds: u32::MAX,
        points: Vec::new(),
        first: prismforge_core::setup::FirstLed::AtStart,
        extra: Map::new(),
    });

    assert_eq!(
        plan_across(&setup, &view_id("v-pc"), RIGHT, &[board()]),
        Err(PlanError::InvalidSetup {
            problem: SetupProblem::InvalidGeometry {
                component: component_id("c-right"),
            },
        })
    );
}

#[test]
fn a_plan_refuses_a_flow_out_of_range() {
    for flow in [
        Flow::Along { angle: 360 },
        Flow::FromCentre { x: 101, y: 50 },
        Flow::FromCentre { x: 0, y: 255 },
    ] {
        assert_eq!(
            plan_across(&two_strips(), &view_id("v-pc"), flow, &[board()]),
            Err(PlanError::InvalidFlow { flow })
        );
    }
}
