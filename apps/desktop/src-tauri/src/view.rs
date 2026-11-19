use prismforge_core::OutputTarget;
use prismforge_core::profile::{ApplyError, ApplyReport, ProfileId};
use prismforge_core::session::{OutputError, SessionError, SessionSnapshot};
use prismforge_core::store::{FileProblem, ImportError, StoreError};
use prismforge_effects::InstanceError;
use serde::Serialize;

use crate::stores::StoreUnavailable;

/// What the window shows; a higher `revision` always replaces a lower one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ShellView {
    pub revision: u64,
    pub session: Option<SessionSnapshot>,
    /// The profile being applied; the shell refuses every other output command until it ends.
    pub applying: Option<ApplyingProfile>,
    /// What the last apply-on-connect of this session did.
    pub connect_apply: Option<ConnectApply>,
    /// Whether this session will still apply a marked profile when it connects.
    pub on_connect: OnConnect,
}

/// The apply-on-connect state of the session, taken from its trigger and the store's mark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum OnConnect {
    /// No session, or its store is not available: nothing is applied on connect.
    NotApplicable,
    /// Not live yet and no profile is marked; a profile marked before the devices are read applies then.
    NotArmed,
    /// Not live yet; this profile is applied as soon as the devices are read.
    Armed { id: ProfileId, name: String },
    /// It came due in this session; nothing more is applied until the user connects again.
    Fired {
        applied: Option<String>,
        next: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ApplyingProfile {
    pub id: ProfileId,
    pub name: String,
    pub automatic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ConnectApply {
    pub id: ProfileId,
    pub name: String,
    pub outcome: ConnectOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ConnectOutcome {
    Applied { report: ApplyReport },
    Failed { error: ApplyError },
}

/// Why an export or import path was refused before any file was read or written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum FileRule {
    NotAbsolute,
    /// The path is inside PrismForge's own folder, where only the store writes.
    InStoreFolder,
    NotAFile,
    TooLarge {
        limit: u64,
    },
    Unreadable {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum CommandError {
    NoSession,
    EmptyHost,
    Session(SessionError),
    InvalidEffect(InstanceError),
    NoEffect { target: OutputTarget },
    ProfileApplying,
    NotConnected,
    Store(StoreError),
    StoreUnavailable(StoreUnavailable),
    Unreadable { problem: FileProblem },
    File(FileRule),
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

impl From<StoreError> for CommandError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<StoreUnavailable> for CommandError {
    fn from(error: StoreUnavailable) -> Self {
        Self::StoreUnavailable(error)
    }
}

impl From<ApplyError> for CommandError {
    fn from(error: ApplyError) -> Self {
        match error {
            ApplyError::NotConnected => Self::NotConnected,
            ApplyError::Store(error) => Self::Store(error),
        }
    }
}

impl From<ImportError> for CommandError {
    fn from(error: ImportError) -> Self {
        match error {
            ImportError::Unreadable { problem } => Self::Unreadable { problem },
            ImportError::Store(error) => Self::Store(error),
        }
    }
}

impl From<FileRule> for CommandError {
    fn from(rule: FileRule) -> Self {
        Self::File(rule)
    }
}
