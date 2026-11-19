use std::fmt;

use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::profile::{Extra, NAME_LIMIT, clean_name};
use crate::scene::OutputTarget;
use crate::store::envelope::FileProblem;

/// Most groups one store keeps.
pub const GROUP_LIMIT: usize = 100;
/// Most members one group holds; the same as the targets of one batch.
pub const MEMBER_LIMIT: usize = crate::session::MAX_BATCH_TARGETS;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(transparent)]
pub struct GroupId(String);

impl GroupId {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `g-` and lower-case letters, digits and hyphens only.
    pub fn parse(value: &str) -> Option<Self> {
        let valid = (3..=64).contains(&value.len())
            && value.starts_with("g-")
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        valid.then(|| Self(value.to_owned()))
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A named set of targets the user picks again; it selects, it owns no lighting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<GroupMember>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// One target as it was when it joined: `label` and `zone_name` tell a missing member apart and show a changed zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupMember {
    pub device: DeviceRef,
    pub zone: Option<usize>,
    pub label: String,
    pub zone_name: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl GroupMember {
    pub fn target(&self) -> OutputTarget {
        OutputTarget {
            device: self.device.clone(),
            zone: self.zone,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GroupsBody {
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl GroupsBody {
    pub fn validate(&self) -> Result<(), FileProblem> {
        if self.groups.len() > GROUP_LIMIT {
            return Err(FileProblem::invalid(format!(
                "more than {GROUP_LIMIT} groups"
            )));
        }
        for (index, group) in self.groups.iter().enumerate() {
            if GroupId::parse(group.id.as_str()).is_none() {
                return Err(FileProblem::invalid(format!(
                    "group {index} has no valid id"
                )));
            }
            if self.groups[..index]
                .iter()
                .any(|other| other.id == group.id)
            {
                return Err(FileProblem::invalid(format!("group id {} twice", group.id)));
            }
            if clean_name(&group.name).as_deref() != Ok(group.name.as_str()) {
                return Err(FileProblem::invalid(format!(
                    "group {} has no valid name of at most {NAME_LIMIT} characters",
                    group.id
                )));
            }
            if group.members.len() > MEMBER_LIMIT {
                return Err(FileProblem::invalid(format!(
                    "group {} has more than {MEMBER_LIMIT} members",
                    group.id
                )));
            }
        }
        Ok(())
    }
}
