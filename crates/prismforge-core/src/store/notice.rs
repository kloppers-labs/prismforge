use serde::{Deserialize, Serialize};

use crate::store::durable::WriteFailure;
use crate::store::envelope::FileProblem;

/// Something the store did to a file without being asked; `file` is relative to the store directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct StoreNotice {
    pub file: String,
    pub event: NoticeEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum NoticeEvent {
    /// The file was missing (`problem` is `None`) or damaged; its backup was used and copied back if `rewritten`.
    RestoredFromBackup {
        problem: Option<FileProblem>,
        set_aside: Option<String>,
        rewritten: bool,
    },
    /// Neither the file nor its backup could be used; the damaged ones were renamed, never deleted.
    StartedEmpty {
        main: Option<FileProblem>,
        backup: Option<FileProblem>,
        set_aside: Vec<String>,
    },
    /// The file needs a newer PrismForge; it is left as it is and this version never writes it.
    LeftUntouched {
        problem: FileProblem,
    },
    WriteFailed {
        failure: WriteFailure,
    },
}
