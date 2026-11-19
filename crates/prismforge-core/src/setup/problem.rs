use serde::{Deserialize, Serialize};

use crate::profile::NameProblem;
use crate::setup::ids::{ComponentId, OutputId, ViewId};

/// Why a setup was not stored; the stored setup stays as it was.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SetupProblem {
    #[error("{id} is not a valid id")]
    InvalidId { id: String },
    #[error("the id {id} is used twice")]
    DuplicateId { id: String },
    #[error("a name is not valid: {problem}")]
    InvalidName { problem: NameProblem },
    #[error("a setup holds at most {limit} {part:?}")]
    TooMany { part: SetupPart, limit: usize },
    #[error("component {component} has a shape that is not valid")]
    InvalidGeometry { component: ComponentId },
    #[error("component {component} is not in the setup")]
    UnknownComponent { component: ComponentId },
    #[error("component {component} is wired to more than one place")]
    WiredTwice { component: ComponentId },
    #[error("output {output} has no component")]
    EmptyOutput { output: OutputId },
    #[error("outputs {first} and {second} share LEDs")]
    OverlappingParts { first: OutputId, second: OutputId },
    #[error("component {component} is placed twice in view {view}")]
    PlacedTwice {
        view: ViewId,
        component: ComponentId,
    },
    #[error("component {component} has no size in view {view}, or a turn of 360 degrees or more")]
    InvalidPlacement {
        view: ViewId,
        component: ComponentId,
    },
    #[error("component {component} has {leds} LEDs; its output takes {min} to {max}")]
    LedCountOutsideBounds {
        component: ComponentId,
        leds: u32,
        min: u32,
        max: u64,
    },
    #[error("output {output} needs {needed} LEDs; it has {available} now")]
    ExceedsOutput {
        output: OutputId,
        needed: u64,
        available: u64,
    },
    /// An output that is new or whose parts changed must name a device or zone the session reports now.
    #[error("output {output} names a device or zone that is not reported now")]
    UnknownTarget { output: OutputId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SetupPart {
    Components,
    Outputs,
    Views,
    Placements,
    Branches,
}
