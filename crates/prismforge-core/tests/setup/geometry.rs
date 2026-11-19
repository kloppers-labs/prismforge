use prismforge_core::setup::{
    Corner, FirstLed, Geometry, GridPoint, Line, Location, Matrix, Perimeter, Ring, Runs, Strip,
    Turn,
};
use serde_json::Map;

use crate::support::{assert_locations, placed, point, strip};

const THIRD: f64 = 1.0 / 3.0;
const SIXTH: f64 = 1.0 / 6.0;

fn ring(first_angle: u16, turn: Turn) -> Geometry {
    Geometry::Ring(Ring {
        leds: 4,
        first_angle,
        turn,
        extra: Map::new(),
    })
}

fn matrix(first: Corner, runs: Runs, serpentine: bool) -> Geometry {
    Geometry::Matrix(Matrix {
        columns: 3,
        rows: 2,
        first,
        runs,
        serpentine,
        extra: Map::new(),
    })
}

fn perimeter(turn: Turn) -> Geometry {
    Geometry::Perimeter(Perimeter {
        top: 2,
        right: 1,
        bottom: 2,
        left: 1,
        width: 2,
        height: 1,
        first: Corner::TopLeft,
        turn,
        extra: Map::new(),
    })
}

fn line(first: FirstLed) -> Geometry {
    let point = |x, y| GridPoint {
        x,
        y,
        extra: Map::new(),
    };
    Geometry::Line(Line {
        leds: 4,
        points: vec![point(0, 0), point(10, 0), point(10, 10)],
        first,
        extra: Map::new(),
    })
}

#[test]
fn a_strip_runs_from_its_first_led_to_the_other_end() {
    let forward = strip(4);
    let backward = Geometry::Strip(Strip {
        leds: 4,
        first: FirstLed::AtEnd,
        extra: Map::new(),
    });

    assert_eq!(forward.led_count(), 4);
    assert_locations(
        &forward.leds(),
        &[(-0.375, 0.0), (-0.125, 0.0), (0.125, 0.0), (0.375, 0.0)],
    );
    assert_locations(
        &backward.leds(),
        &[(0.375, 0.0), (0.125, 0.0), (-0.125, 0.0), (-0.375, 0.0)],
    );
}

#[test]
fn a_ring_turns_from_its_first_angle() {
    assert_locations(
        &ring(0, Turn::Clockwise).leds(),
        &[(0.0, -0.5), (0.5, 0.0), (0.0, 0.5), (-0.5, 0.0)],
    );
    assert_locations(
        &ring(0, Turn::CounterClockwise).leds(),
        &[(0.0, -0.5), (-0.5, 0.0), (0.0, 0.5), (0.5, 0.0)],
    );
    assert_locations(
        &ring(90, Turn::Clockwise).leds(),
        &[(0.5, 0.0), (0.0, 0.5), (-0.5, 0.0), (0.0, -0.5)],
    );
}

#[test]
fn a_matrix_fills_its_lines_from_the_first_corner() {
    let along_rows = matrix(Corner::TopLeft, Runs::AlongRows, false);
    let serpentine = matrix(Corner::TopLeft, Runs::AlongRows, true);
    let from_bottom_right = matrix(Corner::BottomRight, Runs::AlongColumns, false);

    assert_eq!(along_rows.led_count(), 6);
    assert_locations(
        &along_rows.leds(),
        &[
            (-THIRD, -SIXTH),
            (0.0, -SIXTH),
            (THIRD, -SIXTH),
            (-THIRD, SIXTH),
            (0.0, SIXTH),
            (THIRD, SIXTH),
        ],
    );
    assert_locations(
        &serpentine.leds(),
        &[
            (-THIRD, -SIXTH),
            (0.0, -SIXTH),
            (THIRD, -SIXTH),
            (THIRD, SIXTH),
            (0.0, SIXTH),
            (-THIRD, SIXTH),
        ],
    );
    assert_locations(
        &from_bottom_right.leds(),
        &[
            (THIRD, SIXTH),
            (THIRD, -SIXTH),
            (0.0, SIXTH),
            (0.0, -SIXTH),
            (-THIRD, SIXTH),
            (-THIRD, -SIXTH),
        ],
    );
}

#[test]
fn a_perimeter_goes_round_from_the_first_corner() {
    let clockwise = perimeter(Turn::Clockwise);

    assert_eq!(clockwise.led_count(), 6);
    assert_locations(
        &clockwise.leds(),
        &[
            (-0.25, -0.25),
            (0.25, -0.25),
            (0.5, 0.0),
            (0.25, 0.25),
            (-0.25, 0.25),
            (-0.5, 0.0),
        ],
    );
    assert_locations(
        &perimeter(Turn::CounterClockwise).leds(),
        &[
            (-0.5, 0.0),
            (-0.25, 0.25),
            (0.25, 0.25),
            (0.5, 0.0),
            (0.25, -0.25),
            (-0.25, -0.25),
        ],
    );
}

#[test]
fn a_point_is_one_led_in_the_middle() {
    assert_eq!(point().led_count(), 1);
    assert_locations(&point().leds(), &[(0.0, 0.0)]);
}

#[test]
fn a_free_line_spreads_its_leds_evenly_along_the_path() {
    assert_locations(
        &line(FirstLed::AtStart).leds(),
        &[(-0.25, -0.5), (0.25, -0.5), (0.5, -0.25), (0.5, 0.25)],
    );
    assert_locations(
        &line(FirstLed::AtEnd).leds(),
        &[(0.5, 0.25), (0.5, -0.25), (0.25, -0.5), (-0.25, -0.5)],
    );
}

#[test]
fn a_placement_moves_and_scales_the_component() {
    let placement = placed("c-strip", 100, 50, 40);

    let leds: Vec<Location> = strip(4)
        .leds()
        .into_iter()
        .map(|led| placement.locate(led))
        .collect();

    assert_locations(
        &leds,
        &[(85.0, 50.0), (95.0, 50.0), (105.0, 50.0), (115.0, 50.0)],
    );
}

#[test]
fn a_flipped_placement_mirrors_the_component_left_to_right() {
    let mut placement = placed("c-strip", 100, 50, 40);
    placement.flipped = true;

    let leds: Vec<Location> = strip(4)
        .leds()
        .into_iter()
        .map(|led| placement.locate(led))
        .collect();

    assert_locations(
        &leds,
        &[(115.0, 50.0), (105.0, 50.0), (95.0, 50.0), (85.0, 50.0)],
    );
}

#[test]
fn a_rotation_turns_the_component_clockwise_after_the_flip() {
    let mut placement = placed("c-strip", 100, 50, 40);
    placement.rotation = 90;
    let turned: Vec<Location> = strip(4)
        .leds()
        .into_iter()
        .map(|led| placement.locate(led))
        .collect();
    placement.flipped = true;
    let flipped: Vec<Location> = strip(4)
        .leds()
        .into_iter()
        .map(|led| placement.locate(led))
        .collect();

    assert_locations(
        &turned,
        &[(100.0, 35.0), (100.0, 45.0), (100.0, 55.0), (100.0, 65.0)],
    );
    assert_locations(
        &flipped,
        &[(100.0, 65.0), (100.0, 55.0), (100.0, 45.0), (100.0, 35.0)],
    );
}

#[test]
fn the_spacing_of_leds_comes_from_the_shape_not_the_wire_order() {
    let close = |left: f64, right: f64| (left - right).abs() < 1e-9;

    assert!(close(strip(4).spacing(), 0.25));
    assert!(close(ring(0, Turn::Clockwise).spacing(), 0.5_f64.sqrt()));
    assert!(close(
        matrix(Corner::TopLeft, Runs::AlongRows, false).spacing(),
        THIRD
    ));
    assert!(close(perimeter(Turn::Clockwise).spacing(), 0.5));
    assert!(close(point().spacing(), 0.0));
    assert!(close(line(FirstLed::AtStart).spacing(), 0.5));
}

#[test]
fn a_shape_that_is_not_valid_has_no_leds() {
    let mut flat = perimeter(Turn::Clockwise);
    if let Geometry::Perimeter(perimeter) = &mut flat {
        perimeter.width = 0;
        perimeter.height = 0;
    }
    let mut long = line(FirstLed::AtStart);
    if let Geometry::Line(line) = &mut long {
        line.leds = 5_000;
    }
    let mut wide = matrix(Corner::TopLeft, Runs::AlongRows, false);
    if let Geometry::Matrix(matrix) = &mut wide {
        matrix.columns = 100;
        matrix.rows = 100;
    }

    for shape in [flat, long, wide] {
        assert!(shape.leds().is_empty(), "{shape:?}");
        assert!(shape.spacing().is_finite(), "{shape:?}");
    }
}
