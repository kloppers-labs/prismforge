use prismforge_core::automation::{EventProfile, EventProfiles};
use serde_json::{Map, Value};

use super::packet::{PayloadError, decode_string, decode_u32, encode_string};

pub const SERVER_FLAG_SUPPORTS_SETTINGS_MANAGER: u32 = 1 << 4;

pub const PROFILE_MANAGER_KEY: &str = "ProfileManager";

pub fn encode_settings_key(key: &str) -> Vec<u8> {
    encode_string(key)
}

pub fn decode_server_flags(payload: &[u8]) -> Result<u32, PayloadError> {
    decode_u32(payload)
}

/// The `ProfileManager` settings: six event profiles, each `{"enabled": bool, "name": string}`; a missing one is off.
pub fn decode_event_profiles(payload: &[u8]) -> Result<EventProfiles, PayloadError> {
    let text = decode_string(payload).ok_or_else(|| invalid("not a NUL-terminated string"))?;
    let value: Value = serde_json::from_str(&text).map_err(invalid)?;
    let settings = value
        .as_object()
        .ok_or_else(|| invalid("settings are not a JSON object"))?;
    Ok(EventProfiles {
        open: profile(settings, "open_profile")?,
        exit: profile(settings, "exit_profile")?,
        resume: profile(settings, "resume_profile")?,
        suspend: profile(settings, "suspend_profile")?,
        service_startup: profile(settings, "service_startup_profile")?,
        service_shutdown: profile(settings, "service_shutdown_profile")?,
    })
}

fn profile(settings: &Map<String, Value>, key: &str) -> Result<EventProfile, PayloadError> {
    let Some(entry) = settings.get(key) else {
        return Ok(EventProfile::default());
    };
    let enabled = entry.get("enabled").and_then(Value::as_bool);
    let name = entry.get("name").and_then(Value::as_str);
    match (enabled, name) {
        (Some(enabled), Some(name)) => Ok(EventProfile {
            enabled,
            name: name.to_owned(),
        }),
        _ => Err(invalid(format!("{key} is not an enabled flag and a name"))),
    }
}

fn invalid(detail: impl ToString) -> PayloadError {
    PayloadError::Invalid {
        detail: detail.to_string(),
    }
}
