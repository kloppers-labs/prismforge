use serde_json::{Map, Value};

use crate::profile::model::{Profile, ProfileId};
use crate::scene::ApplyMode;
use crate::store::envelope::{self, EXPORT_FORMAT, FileProblem, PROFILE_FORMAT};

const ID_KEY: &str = "id";
const ASSIGNMENTS_KEY: &str = "assignments";
const MODE_KEY: &str = "mode";

pub(crate) fn decode_local(bytes: &[u8], id: &ProfileId) -> Result<Profile, FileProblem> {
    let mut fields = migrated(envelope::open_versioned(bytes, PROFILE_FORMAT)?);
    match fields.remove(ID_KEY) {
        Some(Value::String(found)) if found == id.as_str() => {}
        _ => {
            return Err(FileProblem::invalid(
                "the profile id does not match the file name",
            ));
        }
    }
    checked(envelope::decode(fields)?)
}

pub(crate) fn encode_local(id: &ProfileId, profile: &Profile) -> Vec<u8> {
    let mut leading = Map::new();
    leading.insert(ID_KEY.into(), Value::String(id.as_str().into()));
    envelope::seal(PROFILE_FORMAT, leading, profile)
}

pub(crate) fn decode_export(bytes: &[u8]) -> Result<Profile, FileProblem> {
    checked(envelope::decode(migrated(envelope::open_versioned(
        bytes,
        EXPORT_FORMAT,
    )?))?)
}

pub(crate) fn encode_export(profile: &Profile) -> Vec<u8> {
    envelope::seal(EXPORT_FORMAT, Map::new(), profile)
}

/// Version 1 knew only the mode "each device", so every effect of a version 1 file gets that mode.
fn migrated((mut fields, major): (Map<String, Value>, u32)) -> Map<String, Value> {
    if major == 1
        && let Some(Value::Array(assignments)) = fields.get_mut(ASSIGNMENTS_KEY)
    {
        for assignment in assignments {
            if let Value::Object(assignment) = assignment {
                assignment.insert(
                    MODE_KEY.into(),
                    serde_json::to_value(ApplyMode::EachDevice).expect("a mode is plain data"),
                );
            }
        }
    }
    fields
}

fn checked(profile: Profile) -> Result<Profile, FileProblem> {
    profile.validate()?;
    Ok(profile)
}
