use std::sync::Arc;

use prismforge_effects::{Flow, Plane, ScenePoint};

use crate::scene::OutputTarget;
use crate::session::SessionDevice;
use crate::setup::geometry::Location;
use crate::setup::ids::ViewId;
use crate::setup::model::{Setup, SetupOutput, View};
use crate::setup::resolve::{OutputStatus, resolve_outputs};
use crate::setup::validate::capacity;

/// One effect across a view: a place for each LED of each target, on one plane for all of them.
#[derive(Debug, Clone, PartialEq)]
pub struct AcrossPlan {
    pub view: ViewId,
    pub flow: Flow,
    pub plane: Plane,
    pub targets: Vec<PlacedTarget>,
    /// Outputs that get nothing: their zone changed, or their parts no longer fit it.
    pub excluded: Vec<OutputStatus>,
}

/// A device or zone with the place of each LED by LED index; `None` is an LED no placed component covers.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedTarget {
    pub target: OutputTarget,
    pub points: Arc<[Option<ScenePoint>]>,
}

impl PlacedTarget {
    pub fn not_placed(&self) -> Vec<usize> {
        self.points
            .iter()
            .enumerate()
            .filter(|(_, point)| point.is_none())
            .map(|(index, _)| index)
            .collect()
    }
}

impl AcrossPlan {
    /// The plan for these targets only; the plane stays the plane of the whole view.
    pub fn only(&self, targets: &[OutputTarget]) -> Self {
        Self {
            targets: self
                .targets
                .iter()
                .filter(|placed| targets.contains(&placed.target))
                .cloned()
                .collect(),
            excluded: Vec::new(),
            ..self.clone()
        }
    }
}

/// Places every LED of `view` that is wired and placed; `None` when the setup has no such view.
/// The plane is the box round all those LEDs, so a missing device never moves the picture.
pub fn plan_across(
    setup: &Setup,
    view: &ViewId,
    flow: Flow,
    devices: &[SessionDevice],
) -> Option<AcrossPlan> {
    let view = setup.view(view)?;
    let placed: Vec<Vec<Option<Vec<Location>>>> = setup
        .outputs
        .iter()
        .map(|output| located(setup, view, output))
        .collect();
    let domain = Domain::new(placed.iter().flatten().flatten());
    let statuses = resolve_outputs(setup, devices);
    let mut targets: Vec<OutputTarget> = Vec::new();
    for output in &setup.outputs {
        if !targets.contains(&output.target()) {
            targets.push(output.target());
        }
    }
    let mut plan = AcrossPlan {
        view: view.id.clone(),
        flow,
        plane: domain.plane(),
        targets: Vec::new(),
        excluded: Vec::new(),
    };
    for target in targets {
        let members: Vec<usize> = (0..setup.outputs.len())
            .filter(|&index| setup.outputs[index].target() == target)
            .collect();
        if members
            .iter()
            .any(|&index| !statuses[index].matches_setup())
        {
            plan.excluded
                .extend(members.iter().map(|&index| statuses[index].clone()));
            continue;
        }
        let length = members
            .iter()
            .map(|&index| leds_of(setup, &setup.outputs[index], devices))
            .max()
            .unwrap_or(0);
        let mut points: Vec<Option<ScenePoint>> = vec![None; length];
        for &index in &members {
            let parts = setup.parts(&setup.outputs[index]);
            for (part, leds) in parts.iter().zip(&placed[index]) {
                let Some(leds) = leds else {
                    continue;
                };
                for (offset, location) in leds.iter().enumerate() {
                    if let Some(slot) = points.get_mut(part.leds.start + offset)
                        && slot.is_none()
                    {
                        *slot = Some(domain.point(*location));
                    }
                }
            }
        }
        if points.iter().any(Option::is_some) {
            plan.targets.push(PlacedTarget {
                target,
                points: points.into(),
            });
        }
    }
    Some(plan)
}

/// The view location of each LED of each part of `output`, in part order; `None` for a part not placed in the view.
fn located(setup: &Setup, view: &View, output: &SetupOutput) -> Vec<Option<Vec<Location>>> {
    setup
        .parts(output)
        .iter()
        .map(|part| {
            let placement = view.placement(&part.component)?;
            let component = setup.component(&part.component)?;
            Some(
                component
                    .geometry
                    .leds()
                    .into_iter()
                    .map(|led| placement.locate(led))
                    .collect(),
            )
        })
        .collect()
}

/// The LEDs the target has now, or the LEDs the output uses when the session does not know the device.
fn leds_of(setup: &Setup, output: &SetupOutput, devices: &[SessionDevice]) -> usize {
    match capacity(output, devices) {
        Some((_, available)) => available as usize,
        None => (u64::from(output.first_led) + setup.span(output)) as usize,
    }
}

struct Domain {
    centre: Location,
    longer: f64,
    width: f64,
    height: f64,
    pitch: f64,
}

impl Domain {
    fn new<'a>(components: impl Iterator<Item = &'a Vec<Location>>) -> Self {
        let (mut low, mut high) = (
            Location::new(f64::INFINITY, f64::INFINITY),
            Location::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        );
        let mut pitch: f64 = 0.0;
        for leds in components {
            for led in leds {
                low = Location::new(low.x.min(led.x), low.y.min(led.y));
                high = Location::new(high.x.max(led.x), high.y.max(led.y));
            }
            for pair in leds.windows(2) {
                pitch = pitch.max(pair[0].distance(pair[1]));
            }
        }
        if low.x > high.x {
            return Self {
                centre: Location::new(0.0, 0.0),
                longer: 0.0,
                width: 0.0,
                height: 0.0,
                pitch: 0.0,
            };
        }
        let (width, height) = (high.x - low.x, high.y - low.y);
        Self {
            centre: Location::new((low.x + high.x) / 2.0, (low.y + high.y) / 2.0),
            longer: width.max(height),
            width,
            height,
            pitch,
        }
    }

    fn plane(&self) -> Plane {
        let scale = |length: f64| {
            if self.longer > 0.0 {
                length / self.longer
            } else {
                0.0
            }
        };
        Plane {
            width: scale(self.width),
            height: scale(self.height),
            pitch: scale(self.pitch),
        }
    }

    fn point(&self, location: Location) -> ScenePoint {
        if self.longer > 0.0 {
            ScenePoint {
                x: (location.x - self.centre.x) / self.longer,
                y: (location.y - self.centre.y) / self.longer,
            }
        } else {
            ScenePoint { x: 0.0, y: 0.0 }
        }
    }
}
