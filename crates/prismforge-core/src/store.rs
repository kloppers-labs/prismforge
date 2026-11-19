mod durable;
pub(crate) mod envelope;
mod files;
mod notice;
mod portable;
mod registry;
mod service;
mod settings;
mod summary;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Map;

use crate::device::DeviceRef;
use crate::identity::KnownDevice;
use crate::profile::{
    NAME_LIMIT, NameProblem, Profile, ProfileId, clean_name, decode_export, decode_local,
    encode_export, encode_local,
};
use crate::session::{SessionDevice, SessionError};
use crate::store::durable::{Loaded, StoreFile, Unwritten};

pub use durable::{STORE_FILE_LIMIT, WriteFailure, WriteStep};
pub use envelope::{
    EXPORT_FORMAT, FileProblem, PROFILE_FORMAT, REGISTRY_FORMAT, SCHEMA_VERSION, SETTINGS_FORMAT,
    SchemaVersion,
};
pub use files::{FileOps, StdFileOps};
pub use notice::{NoticeEvent, StoreNotice};
pub use registry::{Registry, RegistryDevice, ZoneName};
pub use service::{ApplyOutcome, ConnectTrigger, Profiles, Saved};
pub use settings::Settings;
pub use summary::{
    ExportReport, ExportedProfile, ImportError, ImportReport, ProfileDetails, ProfileSummary,
    ResolutionCounts, StoreSummary, UnreadableProfile, UnsupportedEffect,
};

/// Largest file `import` accepts.
pub const IMPORT_LIMIT: u64 = 1024 * 1024;

const PROFILES: &str = "profiles";
const PROFILE_EXTENSION: &str = ".json";
const REGISTRY_FILE: &str = "devices.json";
const SETTINGS_FILE: &str = "settings.json";

/// An application store operation that did not happen; nothing here ever touches device memory.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StoreError {
    #[error("there is no profile {id}")]
    UnknownProfile { id: ProfileId },
    #[error("{file} needs a newer PrismForge, so this version does not change it")]
    NewerFile { file: String, problem: FileProblem },
    #[error("the name is not valid: {problem}")]
    InvalidName { problem: NameProblem },
    #[error("another profile is already called {name}")]
    NameTaken { name: String },
    #[error("device {device} is not known")]
    UnknownDevice { device: DeviceRef },
    #[error("zone {zone} does not exist on device {device}")]
    UnknownZone { device: DeviceRef, zone: usize },
    #[error("the profile is not valid: {problem}")]
    InvalidProfile { problem: FileProblem },
    #[error("the device list is not valid: {problem}")]
    InvalidRegistry { problem: FileProblem },
    /// An update needs the profile applied or saved in this session; `active` is the one that is.
    #[error("profile {id} was not applied in this session")]
    NotActive {
        id: ProfileId,
        active: Option<ProfileId>,
    },
    /// The session did not answer, so the user's changes could not be read; nothing was written.
    #[error("the session could not be asked for the user's changes: {error}")]
    SessionUnavailable { error: SessionError },
    #[error("{file} would have {bytes} bytes; the limit is {limit}")]
    TooLarge {
        file: String,
        bytes: u64,
        limit: u64,
    },
    #[error(transparent)]
    Write(#[from] WriteFailure),
    #[error("{file} could not be removed: {detail}")]
    Remove { file: String, detail: String },
    #[error("the store folder could not be prepared: {detail}")]
    Folder { detail: String },
}

impl From<Unwritten> for StoreError {
    fn from(unwritten: Unwritten) -> Self {
        match unwritten {
            Unwritten::TooLarge { file, bytes } => Self::TooLarge {
                file,
                bytes,
                limit: STORE_FILE_LIMIT,
            },
            Unwritten::Failed(failure) => Self::Write(failure),
        }
    }
}

/// Profiles, the known-device registry and settings in one folder; one file each, written durably.
pub struct ProfileStore {
    root: PathBuf,
    ops: Box<dyn FileOps>,
    registry: Registry,
    registry_lock: Option<FileProblem>,
    settings: Settings,
    settings_lock: Option<FileProblem>,
    profiles: BTreeMap<ProfileId, Profile>,
    locked: BTreeMap<ProfileId, FileProblem>,
    notices: Vec<StoreNotice>,
}

impl ProfileStore {
    /// Reads everything in `root`, recovering from backups; damaged files are renamed, never deleted.
    pub fn open(root: impl Into<PathBuf>, ops: impl FileOps + 'static) -> Result<Self, StoreError> {
        let root = root.into();
        ops.create_dir_all(&root.join(PROFILES))
            .map_err(|error| StoreError::Folder {
                detail: error.to_string(),
            })?;
        let mut store = Self {
            root,
            ops: Box::new(ops),
            registry: Registry::default(),
            registry_lock: None,
            settings: Settings::default(),
            settings_lock: None,
            profiles: BTreeMap::new(),
            locked: BTreeMap::new(),
            notices: Vec::new(),
        };
        let file = store.registry_file();
        (store.registry, store.registry_lock) = store.load_or_default(&file, &decode_registry);
        let file = store.settings_file();
        (store.settings, store.settings_lock) = store.load_or_default(&file, &decode_settings);
        store.load_profiles()?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn notices(&self) -> &[StoreNotice] {
        &self.notices
    }

    pub fn note(&mut self, notice: StoreNotice) {
        self.notices.push(notice);
    }

    pub fn profile(&self, id: &ProfileId) -> Result<&Profile, StoreError> {
        if let Some(problem) = self.locked.get(id) {
            return Err(self.newer(&self.profile_file(id), problem));
        }
        self.profiles
            .get(id)
            .ok_or_else(|| StoreError::UnknownProfile { id: id.clone() })
    }

    pub fn profiles(&self) -> impl Iterator<Item = (&ProfileId, &Profile)> {
        self.profiles.iter()
    }

    /// Stores `profile` under a new id; a name another profile has gets a number.
    pub fn create(&mut self, mut profile: Profile) -> Result<ProfileId, StoreError> {
        profile.name = self.distinct_name(&checked_name(&profile.name)?, None);
        let id = self.fresh_id();
        self.write_profile(&id, profile)?;
        Ok(id)
    }

    /// Replaces the stored content of `id` and keeps its name.
    pub fn update(&mut self, id: &ProfileId, mut profile: Profile) -> Result<(), StoreError> {
        profile.name = self.profile(id)?.name.clone();
        self.write_profile(id, profile)
    }

    pub fn rename(&mut self, id: &ProfileId, name: &str) -> Result<(), StoreError> {
        let name = checked_name(name)?;
        let mut profile = self.profile(id)?.clone();
        if self.distinct_name(&name, Some(id)) != name {
            return Err(StoreError::NameTaken { name });
        }
        profile.name = name;
        self.write_profile(id, profile)
    }

    pub fn duplicate(&mut self, id: &ProfileId) -> Result<ProfileId, StoreError> {
        let profile = self.profile(id)?.clone();
        self.create(profile)
    }

    /// Removes the profile's file and backup; clears the apply-on-connect choice if it was this profile.
    pub fn delete(&mut self, id: &ProfileId) -> Result<(), StoreError> {
        self.profile(id)?;
        if self.settings.apply_on_connect.as_ref() == Some(id) {
            self.set_apply_on_connect(None)?;
        }
        let file = self.profile_file(id);
        durable::remove(self.ops.as_ref(), &file).map_err(|error| StoreError::Remove {
            file: file.name.clone(),
            detail: error.to_string(),
        })?;
        self.profiles.remove(id);
        Ok(())
    }

    pub fn apply_on_connect(&self) -> Option<&ProfileId> {
        self.settings.apply_on_connect.as_ref()
    }

    /// Marks the one profile to apply when PrismForge connects, or none.
    pub fn set_apply_on_connect(&mut self, id: Option<ProfileId>) -> Result<(), StoreError> {
        if let Some(id) = &id {
            self.profile(id)?;
        }
        if self.settings.apply_on_connect == id {
            return Ok(());
        }
        let mut settings = self.settings.clone();
        settings.apply_on_connect = id;
        let file = self.settings_file();
        self.unlocked(&file, self.settings_lock.as_ref())?;
        let bytes = envelope::seal(SETTINGS_FORMAT, Map::new(), &settings);
        durable::write(self.ops.as_ref(), &file, &bytes, &|bytes| {
            decode_settings(bytes).is_ok()
        })?;
        self.settings = settings;
        Ok(())
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn known_devices(&self) -> Vec<KnownDevice> {
        self.registry.known_devices()
    }

    /// Stores what the backend reports; writes only when something changed.
    pub fn record_known_devices(&mut self, known: &[KnownDevice]) -> Result<bool, StoreError> {
        let mut registry = self.registry.clone();
        if !registry.record(known) {
            return Ok(false);
        }
        self.write_registry(registry)?;
        Ok(true)
    }

    /// Sets or, with `None`, clears the user's name for a device.
    pub fn set_device_name(
        &mut self,
        device: &DeviceRef,
        name: Option<&str>,
    ) -> Result<(), StoreError> {
        let name = name.map(checked_name).transpose()?;
        let mut registry = self.registry.clone();
        let entry = registry
            .device_mut(device)
            .ok_or_else(|| StoreError::UnknownDevice {
                device: device.clone(),
            })?;
        entry.name = name;
        self.write_registry(registry)
    }

    pub fn set_zone_name(
        &mut self,
        device: &DeviceRef,
        zone: usize,
        name: Option<&str>,
    ) -> Result<(), StoreError> {
        let name = name.map(checked_name).transpose()?;
        let mut registry = self.registry.clone();
        let entry = registry
            .device_mut(device)
            .ok_or_else(|| StoreError::UnknownDevice {
                device: device.clone(),
            })?;
        if zone >= entry.identity.zones.len() {
            return Err(StoreError::UnknownZone {
                device: device.clone(),
                zone,
            });
        }
        entry.zone_names.retain(|entry| entry.zone != zone);
        if let Some(name) = name {
            entry.zone_names.push(ZoneName {
                zone,
                name,
                extra: Map::new(),
            });
            entry.zone_names.sort_by_key(|entry| entry.zone);
        }
        self.write_registry(registry)
    }

    /// The portable form of a profile; private machine data and unknown fields are left out.
    pub fn export(&self, id: &ProfileId) -> Result<ExportedProfile, StoreError> {
        let (portable, omitted_fields) = portable::portable(self.profile(id)?);
        Ok(ExportedProfile {
            bytes: encode_export(&portable),
            omitted_fields,
        })
    }

    /// Writes the export to `destination` through a synced temporary file; no backup is kept there.
    pub fn export_to(
        &self,
        id: &ProfileId,
        destination: &Path,
    ) -> Result<ExportReport, StoreError> {
        let exported = self.export(id)?;
        let file = StoreFile {
            path: destination.to_path_buf(),
            name: destination
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
        };
        durable::write(self.ops.as_ref(), &file, &exported.bytes, &|_| false)?;
        Ok(ExportReport {
            omitted_fields: exported.omitted_fields,
        })
    }

    pub fn import_file(
        &mut self,
        source: &Path,
        devices: &[SessionDevice],
    ) -> Result<ImportReport, ImportError> {
        let bytes = durable::read_bounded(self.ops.as_ref(), source, IMPORT_LIMIT)
            .map_err(|problem| ImportError::Unreadable { problem })?;
        self.import(&bytes, devices)
    }

    /// Checks the whole file before anything is stored, then stores it as a new profile.
    pub fn import(
        &mut self,
        bytes: &[u8],
        devices: &[SessionDevice],
    ) -> Result<ImportReport, ImportError> {
        if bytes.len() as u64 > IMPORT_LIMIT {
            return Err(ImportError::Unreadable {
                problem: FileProblem::TooLarge {
                    limit: IMPORT_LIMIT,
                },
            });
        }
        let profile =
            decode_export(bytes).map_err(|problem| ImportError::Unreadable { problem })?;
        let requested = profile.name.clone();
        let id = self.create(profile)?;
        let profile = self.profile(&id)?;
        Ok(ImportReport {
            renamed_from: (profile.name != requested).then_some(requested),
            details: ProfileDetails::of(&id, profile, devices, self.apply_on_connect()),
        })
    }

    pub fn details(
        &self,
        id: &ProfileId,
        devices: &[SessionDevice],
    ) -> Result<ProfileDetails, StoreError> {
        Ok(ProfileDetails::of(
            id,
            self.profile(id)?,
            devices,
            self.apply_on_connect(),
        ))
    }

    /// Every profile with its match against `devices`, or without counts when `devices` is `None`.
    pub fn summary(&self, devices: Option<&[SessionDevice]>) -> StoreSummary {
        StoreSummary {
            profiles: self
                .profiles
                .iter()
                .map(|(id, profile)| ProfileSummary::of(id, profile, devices))
                .collect(),
            unreadable: self
                .locked
                .iter()
                .map(|(id, problem)| UnreadableProfile {
                    id: id.clone(),
                    problem: problem.clone(),
                })
                .collect(),
            apply_on_connect: self.settings.apply_on_connect.clone(),
            active: None,
            unsaved_changes: None,
            notices: self.notices.clone(),
        }
    }

    fn load_profiles(&mut self) -> Result<(), StoreError> {
        let names =
            self.ops
                .list(&self.root.join(PROFILES))
                .map_err(|error| StoreError::Folder {
                    detail: error.to_string(),
                })?;
        let ids: BTreeSet<ProfileId> = names
            .iter()
            .filter_map(|name| {
                let main = durable::main_of_backup(name).unwrap_or(name);
                main.strip_suffix(PROFILE_EXTENSION)
                    .and_then(ProfileId::parse)
            })
            .collect();
        for id in ids {
            let file = self.profile_file(&id);
            let decode = |bytes: &[u8]| decode_local(bytes, &id);
            match durable::load(self.ops.as_ref(), &file, &decode) {
                Loaded::Absent => {}
                Loaded::Ready(profile) => {
                    self.profiles.insert(id, profile);
                }
                Loaded::Recovered(profile, event) => {
                    self.note_file(&file, event);
                    self.profiles.insert(id, profile);
                }
                Loaded::Empty(event) => self.note_file(&file, event),
                Loaded::Untouched(problem) => {
                    self.note_file(
                        &file,
                        NoticeEvent::LeftUntouched {
                            problem: problem.clone(),
                        },
                    );
                    self.locked.insert(id, problem);
                }
            }
        }
        Ok(())
    }

    fn load_or_default<T: Default>(
        &mut self,
        file: &StoreFile,
        decode: &dyn Fn(&[u8]) -> Result<T, FileProblem>,
    ) -> (T, Option<FileProblem>) {
        match durable::load(self.ops.as_ref(), file, decode) {
            Loaded::Absent => (T::default(), None),
            Loaded::Ready(value) => (value, None),
            Loaded::Recovered(value, event) => {
                self.note_file(file, event);
                (value, None)
            }
            Loaded::Empty(event) => {
                self.note_file(file, event);
                (T::default(), None)
            }
            Loaded::Untouched(problem) => {
                self.note_file(
                    file,
                    NoticeEvent::LeftUntouched {
                        problem: problem.clone(),
                    },
                );
                (T::default(), Some(problem))
            }
        }
    }

    /// Writes nothing when the content is what is stored, so the backup keeps the previous version.
    fn write_profile(&mut self, id: &ProfileId, profile: Profile) -> Result<(), StoreError> {
        if self.profiles.get(id) == Some(&profile) {
            return Ok(());
        }
        profile
            .validate()
            .map_err(|problem| StoreError::InvalidProfile { problem })?;
        let file = self.profile_file(id);
        let bytes = encode_local(id, &profile);
        durable::write(self.ops.as_ref(), &file, &bytes, &|bytes| {
            decode_local(bytes, id).is_ok()
        })?;
        self.profiles.insert(id.clone(), profile);
        Ok(())
    }

    fn write_registry(&mut self, registry: Registry) -> Result<(), StoreError> {
        if registry == self.registry {
            return Ok(());
        }
        let file = self.registry_file();
        self.unlocked(&file, self.registry_lock.as_ref())?;
        registry
            .validate()
            .map_err(|problem| StoreError::InvalidRegistry { problem })?;
        let bytes = envelope::seal(REGISTRY_FORMAT, Map::new(), &registry);
        durable::write(self.ops.as_ref(), &file, &bytes, &|bytes| {
            decode_registry(bytes).is_ok()
        })?;
        self.registry = registry;
        Ok(())
    }

    fn unlocked(&self, file: &StoreFile, lock: Option<&FileProblem>) -> Result<(), StoreError> {
        lock.map_or(Ok(()), |problem| Err(self.newer(file, problem)))
    }

    fn newer(&self, file: &StoreFile, problem: &FileProblem) -> StoreError {
        StoreError::NewerFile {
            file: file.name.clone(),
            problem: problem.clone(),
        }
    }

    fn note_file(&mut self, file: &StoreFile, event: NoticeEvent) {
        self.notices.push(StoreNotice {
            file: file.name.clone(),
            event,
        });
    }

    fn distinct_name(&self, name: &str, except: Option<&ProfileId>) -> String {
        let taken = |candidate: &str| {
            self.profiles
                .iter()
                .any(|(id, profile)| Some(id) != except && profile.name == candidate)
        };
        if !taken(name) {
            return name.to_owned();
        }
        (2..)
            .map(|number| {
                let suffix = format!(" ({number})");
                let room = NAME_LIMIT - suffix.chars().count();
                let base: String = name.chars().take(room).collect();
                format!("{}{suffix}", base.trim_end())
            })
            .find(|candidate| !taken(candidate))
            .expect("some number is free")
    }

    fn fresh_id(&self) -> ProfileId {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        loop {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos() as u64);
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let id = ProfileId::new(format!("p-{:016x}", mix(nanos ^ mix(count))));
            let file = self.profile_file(&id);
            if !self.profiles.contains_key(&id)
                && !self.locked.contains_key(&id)
                && !self.ops.exists(&file.path)
            {
                return id;
            }
        }
    }

    fn profile_file(&self, id: &ProfileId) -> StoreFile {
        StoreFile::new(
            &self.root,
            Some(PROFILES),
            &format!("{id}{PROFILE_EXTENSION}"),
        )
    }

    fn registry_file(&self) -> StoreFile {
        StoreFile::new(&self.root, None, REGISTRY_FILE)
    }

    fn settings_file(&self) -> StoreFile {
        StoreFile::new(&self.root, None, SETTINGS_FILE)
    }
}

fn checked_name(name: &str) -> Result<String, StoreError> {
    clean_name(name).map_err(|problem| StoreError::InvalidName { problem })
}

fn decode_registry(bytes: &[u8]) -> Result<Registry, FileProblem> {
    let registry: Registry = envelope::decode(envelope::open(bytes, REGISTRY_FORMAT)?)?;
    registry.validate()?;
    Ok(registry)
}

fn decode_settings(bytes: &[u8]) -> Result<Settings, FileProblem> {
    envelope::decode(envelope::open(bytes, SETTINGS_FORMAT)?)
}

fn mix(value: u64) -> u64 {
    let mut value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
