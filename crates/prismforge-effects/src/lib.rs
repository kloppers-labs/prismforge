//! Original built-in lighting effects as pure functions of parameters, time, LED position and seed.

mod builtin;
mod color;
mod control;
mod definition;
mod effect;
mod instance;
mod plane;
mod position;
mod preview;
mod registry;

pub use color::Rgb;
pub use control::{
    Bounds, ControlDefinition, ControlKind, ControlValue, Direction, NumberUnit, SpeedUnit,
    ValueProblem, ValueType,
};
pub use definition::EffectDefinition;
pub use effect::Effect;
pub use instance::{EffectInstance, InstanceError, ValueError};
pub use plane::{Flow, Plane, ScenePoint};
pub use position::linear_positions;
pub use preview::{PREVIEW_LIMITS, PreviewBatch, PreviewLimits, PreviewRequest, preview};
pub use registry::{definitions, prepare};
