use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::profile::Extra;
use crate::store::StoreError;
use crate::store::durable::{self, Loaded, StoreFile};
use crate::store::envelope::{self, FileProblem};
use crate::store::files::FileOps;
use crate::store::notice::{NoticeEvent, StoreNotice};

pub const APP_SETTINGS_FORMAT: &str = "prismforge.app-settings";

const APP_SETTINGS_FILE: &str = "app-settings.json";
const MAX_HOST_CHARS: usize = 253;

/// What closing the window does once the user has chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum CloseAction {
    /// The window hides and PrismForge keeps running in the tray.
    Background,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct OpenRgbAddress {
    pub host: String,
    pub port: u16,
}

/// The user's choices for the app itself; every switch is off and no close action is chosen by default.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub on_close: Option<CloseAction>,
    pub start_hidden: bool,
    pub connect_at_start: bool,
    pub last_address: Option<OpenRgbAddress>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl AppSettings {
    fn validate(&self) -> Result<(), FileProblem> {
        let Some(address) = &self.last_address else {
            return Ok(());
        };
        let host = address.host.trim();
        if host.is_empty()
            || host.chars().count() > MAX_HOST_CHARS
            || host.chars().any(char::is_control)
        {
            return Err(FileProblem::invalid("the OpenRGB host is not valid"));
        }
        if address.port == 0 {
            return Err(FileProblem::invalid("the OpenRGB port is 0"));
        }
        Ok(())
    }
}

/// The app settings file in the per-user folder, written durably like every store file.
pub struct AppSettingsFile {
    file: StoreFile,
    ops: Box<dyn FileOps>,
    settings: AppSettings,
    newer: Option<FileProblem>,
    notices: Vec<StoreNotice>,
}

impl AppSettingsFile {
    /// Reads the file, recovering from its backup; never fails, a damaged file is renamed and never deleted.
    pub fn open(root: &Path, ops: impl FileOps + 'static) -> Self {
        let file = StoreFile::new(root, None, APP_SETTINGS_FILE);
        let mut notices = Vec::new();
        let mut note = |event| {
            notices.push(StoreNotice {
                file: file.name.clone(),
                event,
            })
        };
        let (settings, newer) = match durable::load(&ops, &file, &decode) {
            Loaded::Absent => (AppSettings::default(), None),
            Loaded::Ready(settings) => (settings, None),
            Loaded::Recovered(settings, event) => {
                note(event);
                (settings, None)
            }
            Loaded::Empty(event) => {
                note(event);
                (AppSettings::default(), None)
            }
            Loaded::Untouched(problem) => {
                note(NoticeEvent::LeftUntouched {
                    problem: problem.clone(),
                });
                (AppSettings::default(), Some(problem))
            }
        };
        Self {
            file,
            ops: Box::new(ops),
            settings,
            newer,
            notices,
        }
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn notices(&self) -> &[StoreNotice] {
        &self.notices
    }

    /// Writes only when `edit` changed something; the value in memory changes only after the write.
    pub fn change(&mut self, edit: impl FnOnce(&mut AppSettings)) -> Result<(), StoreError> {
        let mut settings = self.settings.clone();
        edit(&mut settings);
        if settings == self.settings {
            return Ok(());
        }
        if let Some(problem) = &self.newer {
            return Err(StoreError::NewerFile {
                file: self.file.name.clone(),
                problem: problem.clone(),
            });
        }
        settings
            .validate()
            .map_err(|problem| StoreError::InvalidSettings { problem })?;
        if let Some(folder) = self.file.path.parent() {
            self.ops
                .create_dir_all(folder)
                .map_err(|error| StoreError::Folder {
                    detail: error.to_string(),
                })?;
        }
        let bytes = envelope::seal(APP_SETTINGS_FORMAT, Map::new(), &settings);
        durable::write(self.ops.as_ref(), &self.file, &bytes, &|bytes| {
            decode(bytes).is_ok()
        })?;
        self.settings = settings;
        Ok(())
    }
}

fn decode(bytes: &[u8]) -> Result<AppSettings, FileProblem> {
    let settings: AppSettings = envelope::decode(envelope::open(bytes, APP_SETTINGS_FORMAT)?)?;
    settings.validate()?;
    Ok(settings)
}
