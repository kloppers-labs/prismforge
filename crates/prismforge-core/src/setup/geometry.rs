use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::profile::Extra;

/// Most LEDs one component has.
pub const COMPONENT_LED_LIMIT: u32 = 4096;
/// Most points one free line has.
pub const LINE_POINT_LIMIT: usize = 256;

/// The shape of a lit component and the order of its LEDs; the user declares it, PrismForge never guesses it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Geometry {
    Strip(Strip),
    Ring(Ring),
    Matrix(Matrix),
    Perimeter(Perimeter),
    Point(Single),
    Line(Line),
}

/// LEDs in a straight line; LED 1 is at the `first` end and the LEDs run to the other end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strip {
    pub leds: u32,
    pub first: FirstLed,
    #[serde(flatten)]
    pub extra: Extra,
}

/// LEDs round a circle; LED 1 sits `first_angle` degrees clockwise from the top.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ring {
    pub leds: u32,
    pub first_angle: u16,
    pub turn: Turn,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A grid with LED 1 in the `first` corner; the LEDs fill one line after another, and turn back on each line when `serpentine`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matrix {
    pub columns: u32,
    pub rows: u32,
    pub first: Corner,
    pub runs: Runs,
    pub serpentine: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

/// LEDs round the edge of a `width` by `height` rectangle, counted per side, from the `first` corner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Perimeter {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
    pub first: Corner,
    pub turn: Turn,
    #[serde(flatten)]
    pub extra: Extra,
}

/// One LED, such as a logo.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Single {
    #[serde(flatten)]
    pub extra: Extra,
}

/// LEDs spread evenly along the path through `points`, LED 1 at the `first` end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub leds: u32,
    pub points: Vec<GridPoint>,
    pub first: FirstLed,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPoint {
    pub x: i32,
    pub y: i32,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirstLed {
    AtStart,
    AtEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Turn {
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

/// Whether the LEDs of a grid fill a row or a column first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runs {
    AlongRows,
    AlongColumns,
}

/// A point in the component's own space: the longer side runs from -0.5 to 0.5 around 0, and y grows downwards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub x: f64,
    pub y: f64,
}

impl Location {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    fn towards(self, other: Self, amount: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * amount,
            self.y + (other.y - self.y) * amount,
        )
    }
}

impl Geometry {
    pub fn led_count(&self) -> u32 {
        match self {
            Self::Strip(Strip { leds, .. })
            | Self::Ring(Ring { leds, .. })
            | Self::Line(Line { leds, .. }) => *leds,
            Self::Matrix(matrix) => matrix.columns.saturating_mul(matrix.rows),
            Self::Perimeter(perimeter) => perimeter
                .sides()
                .iter()
                .fold(0, |sum, side| sum.saturating_add(*side)),
            Self::Point(_) => 1,
        }
    }

    /// Each LED's place in the component's own space, in LED order; none for a shape that is not valid.
    pub fn leds(&self) -> Vec<Location> {
        if !self.is_valid() {
            return Vec::new();
        }
        match self {
            Self::Strip(strip) => strip.leds(),
            Self::Ring(ring) => ring.leds(),
            Self::Matrix(matrix) => matrix.leds(),
            Self::Perimeter(perimeter) => perimeter.leds(),
            Self::Point(_) => vec![Location::new(0.0, 0.0)],
            Self::Line(line) => line.leds(),
        }
    }

    /// The gap between neighbouring LEDs in the component's own space, from the shape and not from the wiring order; 0 for one LED.
    pub fn spacing(&self) -> f64 {
        match self {
            Self::Strip(strip) => 1.0 / f64::from(strip.leds.max(1)),
            Self::Ring(ring) if ring.leds > 1 => (PI / f64::from(ring.leds)).sin(),
            Self::Matrix(matrix) => 1.0 / f64::from(matrix.columns.max(matrix.rows).max(1)),
            Self::Perimeter(perimeter) => perimeter.spacing(),
            Self::Line(line) => line.length() / f64::from(line.leds.max(1)),
            Self::Ring(_) | Self::Point(_) => 0.0,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        let count = self.led_count();
        let shape = match self {
            Self::Ring(ring) => ring.first_angle < 360,
            Self::Perimeter(perimeter) => perimeter.width > 0 && perimeter.height > 0,
            Self::Line(line) => (2..=LINE_POINT_LIMIT).contains(&line.points.len()),
            Self::Strip(_) | Self::Matrix(_) | Self::Point(_) => true,
        };
        shape && (1..=COMPONENT_LED_LIMIT).contains(&count)
    }
}

impl Strip {
    fn leds(&self) -> Vec<Location> {
        let start = Location::new(-0.5, 0.0);
        let end = Location::new(0.5, 0.0);
        match self.first {
            FirstLed::AtStart => spread(start, end, self.leds),
            FirstLed::AtEnd => spread(end, start, self.leds),
        }
    }
}

impl Ring {
    fn leds(&self) -> Vec<Location> {
        let sign = match self.turn {
            Turn::Clockwise => 1.0,
            Turn::CounterClockwise => -1.0,
        };
        let first = f64::from(self.first_angle) / 360.0;
        let count = f64::from(self.leds);
        (0..self.leds)
            .map(|index| {
                let turns = first + sign * f64::from(index) / count;
                let (sin, cos) = (turns * TAU).sin_cos();
                Location::new(0.5 * sin, -0.5 * cos)
            })
            .collect()
    }
}

impl Matrix {
    fn leds(&self) -> Vec<Location> {
        let (columns, rows) = (self.columns, self.rows);
        let pitch = 1.0 / f64::from(columns.max(rows));
        let (lines, per_line) = match self.runs {
            Runs::AlongRows => (rows, columns),
            Runs::AlongColumns => (columns, rows),
        };
        let from_right = matches!(self.first, Corner::TopRight | Corner::BottomRight);
        let from_bottom = matches!(self.first, Corner::BottomLeft | Corner::BottomRight);
        let mut leds = Vec::with_capacity(self.columns.saturating_mul(self.rows) as usize);
        for line in 0..lines {
            for step in 0..per_line {
                let step = if self.serpentine && line % 2 == 1 {
                    per_line - 1 - step
                } else {
                    step
                };
                let (column, row) = match self.runs {
                    Runs::AlongRows => (step, line),
                    Runs::AlongColumns => (line, step),
                };
                let column = if from_right {
                    columns - 1 - column
                } else {
                    column
                };
                let row = if from_bottom { rows - 1 - row } else { row };
                leds.push(Location::new(
                    (f64::from(column) + 0.5 - f64::from(columns) / 2.0) * pitch,
                    (f64::from(row) + 0.5 - f64::from(rows) / 2.0) * pitch,
                ));
            }
        }
        leds
    }
}

impl Perimeter {
    fn sides(&self) -> [u32; 4] {
        [self.top, self.right, self.bottom, self.left]
    }

    fn spacing(&self) -> f64 {
        let longer = f64::from(self.width.max(self.height).max(1));
        let (width, height) = (
            f64::from(self.width) / longer,
            f64::from(self.height) / longer,
        );
        self.sides()
            .into_iter()
            .zip([width, height, width, height])
            .filter(|(count, _)| *count > 0)
            .map(|(count, length)| length / f64::from(count))
            .fold(0.0, f64::max)
    }

    fn leds(&self) -> Vec<Location> {
        let longer = f64::from(self.width.max(self.height));
        let (half_width, half_height) = (
            f64::from(self.width) / longer / 2.0,
            f64::from(self.height) / longer / 2.0,
        );
        let corners = [
            Location::new(-half_width, -half_height),
            Location::new(half_width, -half_height),
            Location::new(half_width, half_height),
            Location::new(-half_width, half_height),
        ];
        let start = match self.first {
            Corner::TopLeft => 0,
            Corner::TopRight => 1,
            Corner::BottomRight => 2,
            Corner::BottomLeft => 3,
        };
        let sides = self.sides();
        let mut leds = Vec::new();
        for turn in 0..4 {
            let (side, from, to) = match self.turn {
                Turn::Clockwise => {
                    let side = (start + turn) % 4;
                    (side, side, (side + 1) % 4)
                }
                Turn::CounterClockwise => {
                    let side = (start + 3 - turn) % 4;
                    (side, (side + 1) % 4, side)
                }
            };
            leds.extend(spread(corners[from], corners[to], sides[side]));
        }
        leds
    }
}

impl Line {
    fn leds(&self) -> Vec<Location> {
        let points = self.normalised();
        let path: Vec<Location> = match self.first {
            FirstLed::AtStart => points,
            FirstLed::AtEnd => points.into_iter().rev().collect(),
        };
        let lengths: Vec<f64> = path
            .windows(2)
            .map(|pair| pair[0].distance(pair[1]))
            .collect();
        let total: f64 = lengths.iter().sum();
        let count = f64::from(self.leds);
        (0..self.leds)
            .map(|index| {
                let mut along = (f64::from(index) + 0.5) / count * total;
                for (segment, length) in lengths.iter().enumerate() {
                    if along <= *length || segment == lengths.len() - 1 {
                        let amount = if *length > 0.0 { along / length } else { 0.0 };
                        return path[segment].towards(path[segment + 1], amount.min(1.0));
                    }
                    along -= length;
                }
                path[0]
            })
            .collect()
    }

    fn length(&self) -> f64 {
        self.normalised()
            .windows(2)
            .map(|pair| pair[0].distance(pair[1]))
            .sum()
    }

    fn normalised(&self) -> Vec<Location> {
        let xs = self.points.iter().map(|point| f64::from(point.x));
        let ys = self.points.iter().map(|point| f64::from(point.y));
        let (low_x, high_x) = bounds(xs);
        let (low_y, high_y) = bounds(ys);
        let longer = (high_x - low_x).max(high_y - low_y);
        let centre = Location::new((low_x + high_x) / 2.0, (low_y + high_y) / 2.0);
        self.points
            .iter()
            .map(|point| {
                if longer > 0.0 {
                    Location::new(
                        (f64::from(point.x) - centre.x) / longer,
                        (f64::from(point.y) - centre.y) / longer,
                    )
                } else {
                    Location::new(0.0, 0.0)
                }
            })
            .collect()
    }
}

/// `count` LEDs from `from` to `to`, each in the middle of an equal share, so neighbours keep one pitch across a join.
fn spread(from: Location, to: Location, count: u32) -> Vec<Location> {
    let total = f64::from(count);
    (0..count)
        .map(|index| from.towards(to, (f64::from(index) + 0.5) / total))
        .collect()
}

fn bounds(values: impl Iterator<Item = f64>) -> (f64, f64) {
    values.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
        (low.min(value), high.max(value))
    })
}
