use std::sync::Arc;

use prismforge_effects::{Flow, Plane, ScenePoint};

use crate::scene::OutputTarget;
use crate::session::SessionDevice;
use crate::setup::geometry::Location;
use crate::setup::ids::ViewId;
use crate::setup::model::{Setup, SetupOutput, View};
use crate::setup::problem::SetupProblem;
use crate::setup::resolve::{OutputStatus, resolve_outputs};
use crate::setup::validate::capacity;

/// One effect across a view: a place for each LED of each target, on one plane for all of them.
/// Only `plan_across` makes one, so its view, flow and places come from a valid setup.
#[derive(Debug, Clone, PartialEq)]
pub struct AcrossPlan {
    view: ViewId,
    flow: Flow,
    plane: Plane,
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

/// Why there is no plan; nothing was placed or allocated for it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    #[error("the setup has no view {view}")]
    UnknownView { view: ViewId },
    #[error("the setup is not valid: {problem}")]
    InvalidSetup { problem: SetupProblem },
    #[error("the flow {flow:?} is outside its ranges")]
    InvalidFlow { flow: Flow },
}

impl AcrossPlan {
    pub fn view(&self) -> &ViewId {
        &self.view
    }

    pub fn flow(&self) -> Flow {
        self.flow
    }

    pub fn plane(&self) -> Plane {
        self.plane
    }

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

/// Places every LED of `view` that is wired and placed, after checking the setup and the flow.
/// The plane is the box round all those LEDs, so a missing device never moves the picture.
pub fn plan_across(
    setup: &Setup,
    view: &ViewId,
    flow: Flow,
    devices: &[SessionDevice],
) -> Result<AcrossPlan, PlanError> {
    if !flow.is_valid() {
        return Err(PlanError::InvalidFlow { flow });
    }
    setup
        .validate()
        .map_err(|problem| PlanError::InvalidSetup { problem })?;
    let view = setup
        .view(view)
        .ok_or_else(|| PlanError::UnknownView { view: view.clone() })?;
    let placed: Vec<Vec<Option<Located>>> = setup
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
            for (part, located) in parts.iter().zip(&placed[index]) {
                let Some(located) = located else {
                    continue;
                };
                for (offset, location) in located.leds.iter().enumerate() {
                    if let Some(slot) = points.get_mut(part.leds.start + offset)
                        && slot.is_none()
                    {
                        *slot = Some(domain.point(*location, located.spacing));
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
    Ok(plan)
}

/// One placed component: each LED's view location, and the gap between neighbouring LEDs in view units.
struct Located {
    leds: Vec<Location>,
    spacing: f64,
}

/// Each part of `output` in part order as placed in the view; `None` for a part not placed in it.
fn located(setup: &Setup, view: &View, output: &SetupOutput) -> Vec<Option<Located>> {
    setup
        .parts(output)
        .iter()
        .map(|part| {
            let placement = view.placement(&part.component)?;
            let geometry = &setup.component(&part.component)?.geometry;
            Some(Located {
                leds: geometry
                    .leds()
                    .into_iter()
                    .map(|led| placement.locate(led))
                    .collect(),
                spacing: geometry.spacing() * f64::from(placement.size),
            })
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
}

impl Domain {
    fn new<'a>(components: impl Iterator<Item = &'a Located>) -> Self {
        let (mut low, mut high) = (
            Location::new(f64::INFINITY, f64::INFINITY),
            Location::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        );
        for led in components.flat_map(|located| &located.leds) {
            low = Location::new(low.x.min(led.x), low.y.min(led.y));
            high = Location::new(high.x.max(led.x), high.y.max(led.y));
        }
        if low.x > high.x {
            return Self {
                centre: Location::new(0.0, 0.0),
                longer: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }
        let (width, height) = (high.x - low.x, high.y - low.y);
        Self {
            centre: Location::new((low.x + high.x) / 2.0, (low.y + high.y) / 2.0),
            longer: width.max(height),
            width,
            height,
        }
    }

    fn scale(&self, length: f64) -> f64 {
        if self.longer > 0.0 {
            length / self.longer
        } else {
            0.0
        }
    }

    fn plane(&self) -> Plane {
        Plane {
            width: self.scale(self.width),
            height: self.scale(self.height),
        }
    }

    fn point(&self, location: Location, spacing: f64) -> ScenePoint {
        ScenePoint {
            x: self.scale(location.x - self.centre.x),
            y: self.scale(location.y - self.centre.y),
            spacing: self.scale(spacing),
        }
    }
}
