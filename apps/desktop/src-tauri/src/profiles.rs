use std::fs;
use std::path::{Path, PathBuf};

use prismforge_core::DeviceRef;
use prismforge_core::profile::{NAME_LIMIT, ProfileId};
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{
    ApplyOutcome, ExportReport, IMPORT_LIMIT, ImportReport, ProfileDetails, Profiles, Saved,
    StoreNotice,
};

use crate::apply_lock::Permit;
use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::stores::StoreView;
use crate::view::{ApplyingProfile, CommandError, FileRule};

const EXPORT_EXTENSION: &str = ".json";
const FALLBACK_FILE_NAME: &str = "profile";

impl<S: ViewSink> Shell<S> {
    pub async fn store_view(&self) -> Result<StoreView, CommandError> {
        let (handle, kind) = self.current().await;
        let profiles = self.stores.profiles(kind).await?;
        let mut summary = profiles.summary(handle.as_ref()).await;
        summary
            .profiles
            .sort_by_cached_key(|profile| profile.name.to_lowercase());
        Ok(StoreView {
            kind,
            folder: self.stores.folder(kind).display().to_string(),
            summary,
            names: profiles.names().await,
            name_limit: NAME_LIMIT,
        })
    }

    pub async fn profile_details(&self, id: &ProfileId) -> Result<ProfileDetails, CommandError> {
        let (handle, profiles) = self.store().await?;
        Ok(profiles.details(id, handle.as_ref()).await?)
    }

    pub async fn create_profile(&self, name: &str) -> Result<Saved, CommandError> {
        let (_permit, handle, profiles) = self.permitted_session_store().await?;
        let saved = profiles.create_from_session(&handle, name).await;
        self.publisher.touch();
        Ok(saved?)
    }

    pub async fn preview_update(&self, id: &ProfileId) -> Result<Saved, CommandError> {
        let (handle, profiles) = self.session_store().await?;
        Ok(profiles.preview_update(&handle, id).await?)
    }

    pub async fn update_profile(&self, id: &ProfileId) -> Result<Saved, CommandError> {
        let (_permit, handle, profiles) = self.permitted_session_store().await?;
        let saved = profiles.update_from_session(&handle, id).await;
        self.publisher.touch();
        Ok(saved?)
    }

    pub async fn rename_profile(&self, id: &ProfileId, name: &str) -> Result<(), CommandError> {
        let (_, profiles) = self.store().await?;
        let renamed = profiles.rename(id, name).await;
        self.report_on_connect().await;
        Ok(renamed?)
    }

    pub async fn duplicate_profile(&self, id: &ProfileId) -> Result<ProfileId, CommandError> {
        let (_, profiles) = self.store().await?;
        Ok(profiles.duplicate(id).await?)
    }

    pub async fn delete_profile(&self, id: &ProfileId) -> Result<(), CommandError> {
        let (permit, _, kind) = self.permitted().await?;
        let profiles = self.stores.profiles(kind).await?;
        let deleted = profiles.delete(id).await;
        drop(permit);
        self.report_on_connect().await;
        Ok(deleted?)
    }

    /// Holds the apply lock for the whole apply, so no other output command interleaves with it.
    pub async fn apply_profile(&self, id: &ProfileId) -> Result<ApplyOutcome, CommandError> {
        let (handle, profiles) = self.session_store().await?;
        let name = profiles.profile(id).await?.name;
        let _guard = self
            .lock
            .try_begin(ApplyingProfile {
                id: id.clone(),
                name,
                automatic: false,
            })
            .await?;
        Ok(profiles.apply(&handle, id).await?)
    }

    pub async fn set_apply_on_connect(&self, id: Option<ProfileId>) -> Result<(), CommandError> {
        let (_, profiles) = self.store().await?;
        let marked = profiles.set_apply_on_connect(id).await;
        self.report_on_connect().await;
        Ok(marked?)
    }

    pub async fn export_file_name(&self, id: &ProfileId) -> Result<String, CommandError> {
        let (_, profiles) = self.store().await?;
        let name = profiles.profile(id).await?.name;
        Ok(file_name(&name))
    }

    pub async fn export_profile(
        &self,
        id: &ProfileId,
        destination: PathBuf,
    ) -> Result<ExportReport, CommandError> {
        self.check_destination(&destination)?;
        let (_, profiles) = self.store().await?;
        Ok(profiles.export_to(id, destination).await?)
    }

    /// Checks the path and the size before anything is read; stores a new profile and writes nothing to hardware.
    pub async fn import_profile(&self, source: PathBuf) -> Result<ImportReport, CommandError> {
        self.check_source(&source)?;
        let (handle, profiles) = self.store().await?;
        Ok(profiles.import_from(source, handle.as_ref()).await?)
    }

    pub async fn set_device_name(
        &self,
        device: DeviceRef,
        name: Option<String>,
    ) -> Result<(), CommandError> {
        let (_, profiles) = self.store().await?;
        Ok(profiles.set_device_name(&device, name).await?)
    }

    pub async fn set_zone_name(
        &self,
        device: DeviceRef,
        zone: usize,
        name: Option<String>,
    ) -> Result<(), CommandError> {
        let (_, profiles) = self.store().await?;
        Ok(profiles.set_zone_name(&device, zone, name).await?)
    }

    pub async fn dismiss_notice(&self, notice: StoreNotice) -> Result<bool, CommandError> {
        let (_, profiles) = self.store().await?;
        Ok(profiles.dismiss_notice(notice).await)
    }

    async fn store(&self) -> Result<(Option<SessionHandle>, Profiles), CommandError> {
        let (handle, kind) = self.current().await;
        Ok((handle, self.stores.profiles(kind).await?))
    }

    async fn permitted_session_store(
        &self,
    ) -> Result<(Permit, SessionHandle, Profiles), CommandError> {
        let (permit, handle, kind) = self.permitted().await?;
        let handle = handle.ok_or(CommandError::NoSession)?;
        Ok((permit, handle, self.stores.profiles(kind).await?))
    }

    async fn session_store(&self) -> Result<(SessionHandle, Profiles), CommandError> {
        let (handle, profiles) = self.store().await?;
        Ok((handle.ok_or(CommandError::NoSession)?, profiles))
    }

    fn check_destination(&self, path: &Path) -> Result<(), FileRule> {
        if !path.is_absolute() {
            return Err(FileRule::NotAbsolute);
        }
        let folder = path.parent().ok_or(FileRule::NotAFile)?;
        self.outside_store(&canonical(folder)?)
    }

    fn check_source(&self, path: &Path) -> Result<(), FileRule> {
        if !path.is_absolute() {
            return Err(FileRule::NotAbsolute);
        }
        let file = canonical(path)?;
        self.outside_store(&file)?;
        let metadata = fs::metadata(&file).map_err(unreadable)?;
        if !metadata.is_file() {
            return Err(FileRule::NotAFile);
        }
        if metadata.len() > IMPORT_LIMIT {
            return Err(FileRule::TooLarge {
                limit: IMPORT_LIMIT,
            });
        }
        Ok(())
    }

    fn outside_store(&self, path: &Path) -> Result<(), FileRule> {
        match fs::canonicalize(self.stores.root()) {
            Ok(root) if path.starts_with(&root) => Err(FileRule::InStoreFolder),
            _ => Ok(()),
        }
    }
}

fn canonical(path: &Path) -> Result<PathBuf, FileRule> {
    fs::canonicalize(path).map_err(unreadable)
}

fn unreadable(error: std::io::Error) -> FileRule {
    FileRule::Unreadable {
        detail: error.to_string(),
    }
}

fn file_name(profile: &str) -> String {
    let safe: String = profile
        .chars()
        .filter(|character| !character.is_control() && !r#"/\:*?"<>|"#.contains(*character))
        .collect();
    let safe = safe.trim().trim_matches('.');
    let stem = if safe.is_empty() {
        FALLBACK_FILE_NAME
    } else {
        safe
    };
    format!("{stem}{EXPORT_EXTENSION}")
}
