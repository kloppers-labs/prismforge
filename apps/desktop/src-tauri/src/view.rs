use prismforge_core::OutputTarget;
use prismforge_core::session::{OutputError, SessionError, SessionSnapshot};
use prismforge_effects::InstanceError;
use serde::Serialize;

/// What the window shows; a higher `revision` always replaces a lower one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ShellView {
    pub revision: u64,
    pub session: Option<SessionSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum CommandError {
    NoSession,
    EmptyHost,
    Session(SessionError),
    InvalidEffect(InstanceError),
    NoEffect { target: OutputTarget },
}

impl From<SessionError> for CommandError {
    fn from(error: SessionError) -> Self {
        Self::Session(error)
    }
}

impl From<OutputError> for CommandError {
    fn from(error: OutputError) -> Self {
        match error {
            OutputError::Session(error) => Self::Session(error),
            OutputError::InvalidEffect(error) => Self::InvalidEffect(error),
            OutputError::NoEffect { target } => Self::NoEffect { target },
        }
    }
}

impl From<InstanceError> for CommandError {
    fn from(error: InstanceError) -> Self {
        Self::InvalidEffect(error)
    }
}
