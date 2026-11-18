//! Original built-in lighting effects as pure functions of parameters, time, LED position and seed.

mod builtin;
mod color;
mod control;
mod definition;
mod effect;
mod instance;
mod position;
mod registry;

pub use color::Rgb;
pub use control::{
    Bounds, ControlDefinition, ControlKind, ControlValue, Direction, NumberUnit, SpeedUnit,
    ValueProblem, ValueType,
};
pub use definition::EffectDefinition;
pub use effect::Effect;
pub use instance::{EffectInstance, InstanceError, ValueError};
pub use position::linear_positions;
pub use registry::{definitions, prepare};
