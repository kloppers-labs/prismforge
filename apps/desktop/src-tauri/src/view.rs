use prismforge_core::session::{SessionError, SessionSnapshot};
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
}

impl From<SessionError> for CommandError {
    fn from(error: SessionError) -> Self {
        Self::Session(error)
    }
}
