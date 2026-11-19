mod control;
mod geometry;
mod ids;
mod model;
mod plan;
mod problem;
mod resolve;
mod validate;

pub use control::{Control, Part};
pub use geometry::{
    COMPONENT_LED_LIMIT, Corner, FirstLed, Geometry, GridPoint, LINE_POINT_LIMIT, Line, Location,
    Matrix, Perimeter, Ring, Runs, Single, Strip, Turn,
};
pub use ids::{ComponentId, OutlineId, OutputId, ViewId};
pub use model::{Component, Frame, Outline, OutlineKind, Placement, Setup, SetupOutput, View};
pub use plan::{AcrossPlan, PlacedTarget, PlanError, plan_across};
pub use problem::{SetupPart, SetupProblem};
pub use resolve::{OutputStatus, resolve_outputs};
pub use validate::{
    BRANCH_LIMIT, COMPONENT_LIMIT, FRAME_LIMITS, OUTLINE_SIZE_LIMIT, OUTPUT_LIMIT, VIEW_LIMIT,
};
