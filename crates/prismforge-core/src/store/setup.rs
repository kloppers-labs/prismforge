use std::path::Path;

use serde_json::Map;

use crate::profile::clean_name;
use crate::session::SessionDevice;
use crate::setup::Setup;
use crate::store::StoreError;
use crate::store::durable::{self, Loaded, StoreFile};
use crate::store::envelope::{self, FileProblem};
use crate::store::files::FileOps;
use crate::store::notice::{NoticeEvent, StoreNotice};

pub const SETUP_FORMAT: &str = "prismforge.setup";

const SETUP_FILE: &str = "setup.json";

/// The setup of one store in one file, written durably like every store file; never part of an export.
pub struct SetupFile {
    file: StoreFile,
    setup: Setup,
    newer: Option<FileProblem>,
    notices: Vec<StoreNotice>,
}

impl SetupFile {
    /// Reads the file, recovering from its backup; never fails, a damaged file is renamed and never deleted.
    pub fn open(root: &Path, ops: &dyn FileOps) -> Self {
        let file = StoreFile::new(root, None, SETUP_FILE);
        let mut notices = Vec::new();
        let mut note = |event| {
            notices.push(StoreNotice {
                file: file.name.clone(),
                event,
            })
        };
        let (setup, newer) = match durable::load(ops, &file, &decode) {
            Loaded::Absent => (Setup::default(), None),
            Loaded::Ready(setup) => (setup, None),
            Loaded::Recovered(setup, event) => {
                note(event);
                (setup, None)
            }
            Loaded::Empty(event) => {
                note(event);
                (Setup::default(), None)
            }
            Loaded::Untouched(problem) => {
                note(NoticeEvent::LeftUntouched {
                    problem: problem.clone(),
                });
                (Setup::default(), Some(problem))
            }
        };
        Self {
            file,
            setup,
            newer,
            notices,
        }
    }

    /// Refused while the file needs a newer PrismForge.
    pub fn setup(&self) -> Result<&Setup, StoreError> {
        match &self.newer {
            Some(problem) => Err(StoreError::NewerFile {
                file: self.file.name.clone(),
                problem: problem.clone(),
            }),
            None => Ok(&self.setup),
        }
    }

    pub fn notices(&self) -> &[StoreNotice] {
        &self.notices
    }

    /// Writes `setup` in place of the stored one; outputs that are new or changed are checked against `devices`.
    pub fn save(
        &mut self,
        ops: &dyn FileOps,
        mut setup: Setup,
        devices: &[SessionDevice],
    ) -> Result<(), StoreError> {
        if let Some(problem) = &self.newer {
            return Err(StoreError::NewerFile {
                file: self.file.name.clone(),
                problem: problem.clone(),
            });
        }
        for component in &mut setup.components {
            trim(&mut component.name);
        }
        for view in &mut setup.views {
            trim(&mut view.name);
        }
        setup
            .validate()
            .and_then(|()| setup.check_changed_outputs(&self.setup, devices))
            .map_err(|problem| StoreError::InvalidSetup { problem })?;
        if setup == self.setup {
            return Ok(());
        }
        if let Some(folder) = self.file.path.parent() {
            ops.create_dir_all(folder)
                .map_err(|error| StoreError::Folder {
                    detail: error.to_string(),
                })?;
        }
        let bytes = envelope::seal(SETUP_FORMAT, Map::new(), &setup);
        durable::write(ops, &self.file, &bytes, &|bytes| decode(bytes).is_ok())?;
        self.setup = setup;
        Ok(())
    }
}

fn trim(name: &mut String) {
    if let Ok(clean) = clean_name(name) {
        *name = clean;
    }
}

fn decode(bytes: &[u8]) -> Result<Setup, FileProblem> {
    let setup: Setup = envelope::decode(envelope::open(bytes, SETUP_FORMAT)?)?;
    setup.validate().map_err(FileProblem::invalid)?;
    Ok(setup)
}
