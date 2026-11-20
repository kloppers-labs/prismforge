use std::sync::{Mutex, PoisonError};

use prismforge_core::automation::EventProfiles;
use prismforge_core::session::{SessionError, SessionHandle};
use serde::Serialize;

/// What PrismForge knows about OpenRGB's own profiles for the same events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum OpenRgbCheck {
    /// No session, or the settings were not read yet.
    NotChecked,
    Read {
        profiles: EventProfiles,
    },
    /// The server has no settings manager to ask, or this is demo mode.
    CannotTell,
    Failed {
        error: SessionError,
    },
}

/// The last check of each session; never held across an await.
#[derive(Default)]
pub(crate) struct OpenRgbChecks(Mutex<Option<(u64, OpenRgbCheck)>>);

impl OpenRgbChecks {
    pub fn get(&self, session: u64) -> OpenRgbCheck {
        match &*self.lock() {
            Some((checked, check)) if *checked == session => check.clone(),
            _ => OpenRgbCheck::NotChecked,
        }
    }

    /// Reads OpenRGB's settings again; PrismForge never changes them.
    pub async fn refresh(&self, handle: &SessionHandle) -> OpenRgbCheck {
        let check = match handle.event_profiles().await {
            Ok(Some(profiles)) => OpenRgbCheck::Read { profiles },
            Ok(None) => OpenRgbCheck::CannotTell,
            Err(error) => OpenRgbCheck::Failed { error },
        };
        *self.lock() = Some((handle.session_id(), check.clone()));
        check
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<(u64, OpenRgbCheck)>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
