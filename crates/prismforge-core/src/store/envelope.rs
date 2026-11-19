use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const PROFILE_FORMAT: &str = "prismforge.profile";
pub const EXPORT_FORMAT: &str = "prismforge.profile-export";
pub const REGISTRY_FORMAT: &str = "prismforge.devices";
pub const SETTINGS_FORMAT: &str = "prismforge.settings";

/// The schema this build writes for every file but profiles and exports; it reads files with the same major version.
pub const SCHEMA_VERSION: SchemaVersion = SchemaVersion { major: 1, minor: 0 };
/// The schema this build writes for profiles and exports; it reads version 1 as well and migrates it.
pub const PROFILE_SCHEMA_VERSION: SchemaVersion = SchemaVersion { major: 2, minor: 0 };
const OLDEST_MAJOR: u32 = 1;

const FORMAT_KEY: &str = "format";
const VERSION_KEY: &str = "schema_version";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
}

/// Why a stored or imported file was not used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum FileProblem {
    #[error("the file could not be read: {detail}")]
    Unreadable { detail: String },
    #[error("the file is larger than {limit} bytes")]
    TooLarge { limit: u64 },
    #[error("the file is not valid JSON: {detail}")]
    NotJson { detail: String },
    #[error("the file is not a {expected} file")]
    WrongFormat {
        expected: String,
        found: Option<String>,
    },
    #[error("the file has no readable schema version")]
    NoVersion,
    #[error("schema version {found} needs a newer PrismForge; this one reads version {supported}")]
    NewerVersion { found: u32, supported: u32 },
    #[error("schema version {found} is not a PrismForge version")]
    UnknownOldVersion { found: u32 },
    #[error("the file content is not valid: {detail}")]
    Invalid { detail: String },
}

impl FileProblem {
    pub(crate) fn invalid(detail: impl ToString) -> Self {
        Self::Invalid {
            detail: detail.to_string(),
        }
    }
}

/// Checks the format marker and that the schema version is the current one, then returns the remaining fields.
pub(crate) fn open(bytes: &[u8], format: &str) -> Result<Map<String, Value>, FileProblem> {
    let (fields, major) = open_versioned(bytes, format)?;
    if major != current(format).major {
        return Err(FileProblem::UnknownOldVersion { found: major });
    }
    Ok(fields)
}

/// Checks the format marker and the schema version, then returns the remaining fields and the major version found.
pub(crate) fn open_versioned(
    bytes: &[u8],
    format: &str,
) -> Result<(Map<String, Value>, u32), FileProblem> {
    let value: Value = serde_json::from_slice(bytes).map_err(|error| FileProblem::NotJson {
        detail: error.to_string(),
    })?;
    let Value::Object(mut fields) = value else {
        return Err(wrong_format(format, None));
    };
    match fields.remove(FORMAT_KEY) {
        Some(Value::String(found)) if found == format => {}
        Some(Value::String(found)) => return Err(wrong_format(format, Some(found))),
        _ => return Err(wrong_format(format, None)),
    }
    let version: SchemaVersion = fields
        .remove(VERSION_KEY)
        .and_then(|version| serde_json::from_value(version).ok())
        .ok_or(FileProblem::NoVersion)?;
    let supported = current(format).major;
    if version.major > supported {
        return Err(FileProblem::NewerVersion {
            found: version.major,
            supported,
        });
    }
    if version.major < OLDEST_MAJOR {
        return Err(FileProblem::UnknownOldVersion {
            found: version.major,
        });
    }
    Ok((fields, version.major))
}

pub(crate) fn decode<T: DeserializeOwned>(fields: Map<String, Value>) -> Result<T, FileProblem> {
    serde_json::from_value(Value::Object(fields)).map_err(FileProblem::invalid)
}

/// Writes the format marker, the current schema version, `leading` and then the fields of `body`.
pub(crate) fn seal(format: &str, leading: Map<String, Value>, body: &impl Serialize) -> Vec<u8> {
    let mut fields = Map::new();
    fields.insert(FORMAT_KEY.into(), Value::String(format.into()));
    fields.insert(
        VERSION_KEY.into(),
        serde_json::to_value(current(format)).expect("a schema version is plain data"),
    );
    fields.extend(leading);
    if let Value::Object(body) = serde_json::to_value(body).expect("stored types are plain data") {
        fields.extend(body);
    }
    let mut bytes = serde_json::to_vec_pretty(&Value::Object(fields)).expect("JSON values encode");
    bytes.push(b'\n');
    bytes
}

fn current(format: &str) -> SchemaVersion {
    if format == PROFILE_FORMAT || format == EXPORT_FORMAT {
        PROFILE_SCHEMA_VERSION
    } else {
        SCHEMA_VERSION
    }
}

fn wrong_format(expected: &str, found: Option<String>) -> FileProblem {
    FileProblem::WrongFormat {
        expected: expected.into(),
        found,
    }
}
