use serde::{Deserialize, Serialize};

const FLAT: f64 = 1e-9;

/// An LED's place across a setup: (0, 0) is the middle of the plane, whose longer side is 1 long.
/// `spacing` is the gap between neighbouring LEDs of the LED's own component, on the same scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenePoint {
    pub x: f64,
    pub y: f64,
    pub spacing: f64,
}

/// The area an effect covers, on the scale of its points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub width: f64,
    pub height: f64,
}

/// How an effect crosses the plane: along a direction (degrees clockwise from left to right), or out from a centre (percent of the plane).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Flow {
    Along { angle: u16 },
    FromCentre { x: u8, y: u8 },
}

/// Maps a point to a position from 0.0 to 1.0 across the whole plane, so the same effect code serves a line and a plane.
pub(crate) struct Projection {
    origin: Offset,
    direction: Option<Offset>,
    length: f64,
}

#[derive(Clone, Copy)]
struct Offset {
    x: f64,
    y: f64,
}

impl Projection {
    pub fn new(flow: Flow, plane: Plane) -> Self {
        let (half_width, half_height) = (plane.width / 2.0, plane.height / 2.0);
        match flow {
            Flow::Along { angle } => {
                let (sin, cos) = f64::from(angle % 360).to_radians().sin_cos();
                Self {
                    origin: Offset { x: 0.0, y: 0.0 },
                    direction: Some(Offset { x: cos, y: sin }),
                    length: plane.width * cos.abs() + plane.height * sin.abs(),
                }
            }
            Flow::FromCentre { x, y } => {
                let share = |percent: u8| f64::from(percent.min(100)) / 100.0;
                let centre = Offset {
                    x: (share(x) - 0.5) * plane.width,
                    y: (share(y) - 0.5) * plane.height,
                };
                let farthest = (centre.x.abs() + half_width).hypot(centre.y.abs() + half_height);
                Self {
                    origin: centre,
                    direction: None,
                    length: farthest,
                }
            }
        }
    }

    pub fn position(&self, point: ScenePoint) -> f64 {
        if self.length < FLAT {
            return 0.0;
        }
        let (x, y) = (point.x - self.origin.x, point.y - self.origin.y);
        let position = match self.direction {
            Some(direction) => 0.5 + (x * direction.x + y * direction.y) / self.length,
            None => x.hypot(y) / self.length,
        };
        if position.is_nan() {
            0.0
        } else {
            position.clamp(0.0, 1.0)
        }
    }

    /// A distance on the plane as a share of the full position range.
    pub fn share(&self, distance: f64) -> f64 {
        if self.length < FLAT {
            0.0
        } else {
            distance / self.length
        }
    }
}
