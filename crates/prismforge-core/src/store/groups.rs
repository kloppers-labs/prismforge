mod model;
mod resolve;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Map;

use crate::profile::{Extra, clean_name};
use crate::scene::OutputTarget;
use crate::session::SessionDevice;
use crate::store::durable::{self, Loaded, StoreFile};
use crate::store::envelope::{self, FileProblem};
use crate::store::files::FileOps;
use crate::store::notice::{NoticeEvent, StoreNotice};
use crate::store::{StoreError, mix};

pub use model::{GROUP_LIMIT, Group, GroupId, GroupMember, MEMBER_LIMIT};
pub(crate) use resolve::target_state;
pub use resolve::{MemberState, MemberStatus, resolve_members};

use model::GroupsBody;

pub const GROUPS_FORMAT: &str = "prismforge.groups";

const GROUPS_FILE: &str = "groups.json";

/// The groups of one store in one file, written durably like every store file; never part of a profile or an export.
pub struct GroupsFile {
    file: StoreFile,
    ops: Box<dyn FileOps>,
    body: GroupsBody,
    newer: Option<FileProblem>,
    notices: Vec<StoreNotice>,
}

impl GroupsFile {
    /// Reads the file, recovering from its backup; never fails, a damaged file is renamed and never deleted.
    pub fn open(root: &Path, ops: impl FileOps + 'static) -> Self {
        let file = StoreFile::new(root, None, GROUPS_FILE);
        let mut notices = Vec::new();
        let mut note = |event| {
            notices.push(StoreNotice {
                file: file.name.clone(),
                event,
            })
        };
        let (body, newer) = match durable::load(&ops, &file, &decode) {
            Loaded::Absent => (GroupsBody::default(), None),
            Loaded::Ready(body) => (body, None),
            Loaded::Recovered(body, event) => {
                note(event);
                (body, None)
            }
            Loaded::Empty(event) => {
                note(event);
                (GroupsBody::default(), None)
            }
            Loaded::Untouched(problem) => {
                note(NoticeEvent::LeftUntouched {
                    problem: problem.clone(),
                });
                (GroupsBody::default(), Some(problem))
            }
        };
        Self {
            file,
            ops: Box::new(ops),
            body,
            newer,
            notices,
        }
    }

    pub fn groups(&self) -> &[Group] {
        &self.body.groups
    }

    pub fn group(&self, id: &GroupId) -> Option<&Group> {
        self.body.groups.iter().find(|group| &group.id == id)
    }

    pub fn notices(&self) -> &[StoreNotice] {
        &self.notices
    }

    /// Every target must be a device or zone of `devices` now; its name is kept with it.
    pub fn create(
        &mut self,
        name: &str,
        targets: &[OutputTarget],
        devices: &[SessionDevice],
    ) -> Result<GroupId, StoreError> {
        if self.body.groups.len() >= GROUP_LIMIT {
            return Err(StoreError::TooManyGroups { limit: GROUP_LIMIT });
        }
        let name = self.free_name(name, None)?;
        let members = joined(&[], targets, devices)?;
        let id = self.new_id();
        let group = Group {
            id: id.clone(),
            name,
            members,
            extra: Extra::new(),
        };
        self.change(|body| body.groups.push(group))?;
        Ok(id)
    }

    pub fn rename(&mut self, id: &GroupId, name: &str) -> Result<(), StoreError> {
        let index = self.index(id)?;
        let name = self.free_name(name, Some(id))?;
        self.change(|body| body.groups[index].name = name)
    }

    /// Adds the targets that are not members yet; members missing today stay as they are.
    pub fn add_members(
        &mut self,
        id: &GroupId,
        targets: &[OutputTarget],
        devices: &[SessionDevice],
    ) -> Result<(), StoreError> {
        let index = self.index(id)?;
        let members = joined(&self.body.groups[index].members, targets, devices)?;
        self.change(|body| body.groups[index].members = members)
    }

    pub fn remove_member(&mut self, id: &GroupId, target: &OutputTarget) -> Result<(), StoreError> {
        let index = self.index(id)?;
        self.change(|body| {
            body.groups[index]
                .members
                .retain(|member| &member.target() != target)
        })
    }

    pub fn delete(&mut self, id: &GroupId) -> Result<(), StoreError> {
        let index = self.index(id)?;
        self.change(|body| {
            body.groups.remove(index);
        })
    }

    fn index(&self, id: &GroupId) -> Result<usize, StoreError> {
        self.body
            .groups
            .iter()
            .position(|group| &group.id == id)
            .ok_or_else(|| StoreError::UnknownGroup { id: id.clone() })
    }

    fn free_name(&self, name: &str, keep: Option<&GroupId>) -> Result<String, StoreError> {
        let name = clean_name(name).map_err(|problem| StoreError::InvalidName { problem })?;
        let taken = self.body.groups.iter().any(|group| {
            Some(&group.id) != keep && group.name.to_lowercase() == name.to_lowercase()
        });
        if taken {
            return Err(StoreError::GroupNameTaken { name });
        }
        Ok(name)
    }

    fn new_id(&self) -> GroupId {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        loop {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos() as u64);
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let id = GroupId::new(format!("g-{:016x}", mix(nanos ^ mix(count))));
            if self.group(&id).is_none() {
                return id;
            }
        }
    }

    /// Writes first; the groups in memory change only after the write.
    fn change(&mut self, edit: impl FnOnce(&mut GroupsBody)) -> Result<(), StoreError> {
        if let Some(problem) = &self.newer {
            return Err(StoreError::NewerFile {
                file: self.file.name.clone(),
                problem: problem.clone(),
            });
        }
        let mut body = self.body.clone();
        edit(&mut body);
        if body == self.body {
            return Ok(());
        }
        body.validate()
            .map_err(|problem| StoreError::InvalidGroups { problem })?;
        if let Some(folder) = self.file.path.parent() {
            self.ops
                .create_dir_all(folder)
                .map_err(|error| StoreError::Folder {
                    detail: error.to_string(),
                })?;
        }
        let bytes = envelope::seal(GROUPS_FORMAT, Map::new(), &body);
        durable::write(self.ops.as_ref(), &self.file, &bytes, &|bytes| {
            decode(bytes).is_ok()
        })?;
        self.body = body;
        Ok(())
    }
}

fn joined(
    members: &[GroupMember],
    targets: &[OutputTarget],
    devices: &[SessionDevice],
) -> Result<Vec<GroupMember>, StoreError> {
    if targets.len() > MEMBER_LIMIT {
        return Err(StoreError::TooManyMembers {
            limit: MEMBER_LIMIT,
        });
    }
    let mut joined = members.to_vec();
    for target in targets {
        if joined.iter().any(|member| &member.target() == target) {
            continue;
        }
        let device = devices
            .iter()
            .find(|device| device.identity.reference() == Some(&target.device))
            .ok_or_else(|| StoreError::UnknownDevice {
                device: target.device.clone(),
            })?;
        let zone_name = match target.zone {
            None => None,
            Some(zone) => Some(
                device
                    .descriptor
                    .zones
                    .get(zone)
                    .ok_or_else(|| StoreError::UnknownZone {
                        device: target.device.clone(),
                        zone,
                    })?
                    .name
                    .clone(),
            ),
        };
        joined.push(GroupMember {
            device: target.device.clone(),
            zone: target.zone,
            label: device.descriptor.name.clone(),
            zone_name,
            extra: Extra::new(),
        });
    }
    if joined.len() > MEMBER_LIMIT {
        return Err(StoreError::TooManyMembers {
            limit: MEMBER_LIMIT,
        });
    }
    Ok(joined)
}

fn decode(bytes: &[u8]) -> Result<GroupsBody, FileProblem> {
    let body: GroupsBody = envelope::decode(envelope::open(bytes, GROUPS_FORMAT)?)?;
    body.validate()?;
    Ok(body)
}
