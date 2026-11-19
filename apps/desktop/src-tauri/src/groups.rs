use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use prismforge_core::OutputTarget;
use prismforge_core::profile::NAME_LIMIT;
use prismforge_core::session::SessionDevice;
use prismforge_core::store::{
    GROUP_LIMIT, GroupId, GroupsFile, MEMBER_LIMIT, MemberStatus, StdFileOps, StoreError,
    StoreNotice, resolve_members,
};
use serde::Serialize;

use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::stores::StoreKind;
use crate::view::CommandError;

/// The groups of the store in use, each member matched against the session's devices of now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct GroupsView {
    pub kind: StoreKind,
    pub groups: Vec<GroupView>,
    pub notices: Vec<StoreNotice>,
    pub name_limit: usize,
    pub group_limit: usize,
    pub member_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct GroupView {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<MemberStatus>,
}

/// The groups file of one store for async callers; every file operation runs on a blocking thread.
#[derive(Clone)]
pub struct GroupsStore {
    file: Arc<Mutex<GroupsFile>>,
}

impl GroupsStore {
    pub async fn open(folder: PathBuf) -> Self {
        let file = tokio::task::spawn_blocking(move || GroupsFile::open(&folder, StdFileOps))
            .await
            .expect("opening the groups does not panic");
        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }

    async fn with<T: Send + 'static>(
        &self,
        work: impl FnOnce(&mut GroupsFile) -> T + Send + 'static,
    ) -> T {
        let file = self.file.clone();
        tokio::task::spawn_blocking(move || {
            work(&mut file.lock().unwrap_or_else(PoisonError::into_inner))
        })
        .await
        .expect("group work does not panic")
    }
}

impl<S: ViewSink> Shell<S> {
    pub async fn groups_view(&self) -> Result<GroupsView, CommandError> {
        self.on_groups(|_, _| Ok(())).await
    }

    pub async fn create_group(
        &self,
        name: &str,
        targets: Vec<OutputTarget>,
    ) -> Result<GroupsView, CommandError> {
        let name = name.to_owned();
        self.on_groups(move |file, devices| file.create(&name, &targets, devices).map(drop))
            .await
    }

    pub async fn rename_group(&self, id: &GroupId, name: &str) -> Result<GroupsView, CommandError> {
        let (id, name) = (id.clone(), name.to_owned());
        self.on_groups(move |file, _| file.rename(&id, &name)).await
    }

    pub async fn add_group_members(
        &self,
        id: &GroupId,
        targets: Vec<OutputTarget>,
    ) -> Result<GroupsView, CommandError> {
        let id = id.clone();
        self.on_groups(move |file, devices| file.add_members(&id, &targets, devices))
            .await
    }

    pub async fn remove_group_member(
        &self,
        id: &GroupId,
        target: OutputTarget,
    ) -> Result<GroupsView, CommandError> {
        let id = id.clone();
        self.on_groups(move |file, _| file.remove_member(&id, &target))
            .await
    }

    pub async fn delete_group(&self, id: &GroupId) -> Result<GroupsView, CommandError> {
        let id = id.clone();
        self.on_groups(move |file, _| file.delete(&id)).await
    }

    /// Takes the session slot only to read the devices and the store kind, then works on the store alone.
    async fn on_groups(
        &self,
        work: impl FnOnce(&mut GroupsFile, &[SessionDevice]) -> Result<(), StoreError> + Send + 'static,
    ) -> Result<GroupsView, CommandError> {
        let (handle, kind) = self.current().await;
        let devices = handle.map_or_else(Vec::new, |handle| handle.snapshot().devices);
        let store = self.stores.groups(kind).await?;
        let (result, groups, notices) = store
            .with(move |file| {
                let result = work(file, &devices);
                let groups = file
                    .groups()
                    .iter()
                    .map(|group| GroupView {
                        id: group.id.clone(),
                        name: group.name.clone(),
                        members: resolve_members(&group.members, &devices),
                    })
                    .collect();
                (result, groups, file.notices().to_vec())
            })
            .await;
        result?;
        Ok(GroupsView {
            kind,
            groups,
            notices,
            name_limit: NAME_LIMIT,
            group_limit: GROUP_LIMIT,
            member_limit: MEMBER_LIMIT,
        })
    }
}
