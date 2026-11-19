use serde::Serialize;

use crate::view::SettingProblem;

/// The operating system's per-user start entry for PrismForge.
pub trait StartEntry: Send + Sync + 'static {
    fn is_registered(&self) -> Result<bool, String>;
    fn register(&self) -> Result<(), String>;
    fn unregister(&self) -> Result<(), String>;
}

/// What the system reports about the start entry, read back after every change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StartEntryState {
    Registered,
    NotRegistered,
    Unknown { detail: String },
}

pub fn read_entry(entry: &dyn StartEntry) -> StartEntryState {
    match entry.is_registered() {
        Ok(true) => StartEntryState::Registered,
        Ok(false) => StartEntryState::NotRegistered,
        Err(detail) => StartEntryState::Unknown { detail },
    }
}

/// Succeeds only when the system reads back the state the user asked for.
pub fn switch_entry(entry: &dyn StartEntry, on: bool) -> Result<StartEntryState, SettingProblem> {
    let failed = |detail| SettingProblem::StartEntry { wanted: on, detail };
    if on {
        entry.register()
    } else {
        entry.unregister()
    }
    .map_err(|detail| failed(Some(detail)))?;
    match entry.is_registered() {
        Ok(true) if on => Ok(StartEntryState::Registered),
        Ok(false) if !on => Ok(StartEntryState::NotRegistered),
        Ok(_) => Err(failed(None)),
        Err(detail) => Err(failed(Some(detail))),
    }
}
