use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::connection::ConnectionState;
use crate::device::DeviceRef;
use crate::identity::KnownDevice;
use crate::profile::{
    ApplyError, ApplyReport, FrameRateChange, Profile, ProfileId, SettingChange, Updated,
    all_added, apply, is_live, update,
};
use crate::session::{IntentLog, SessionDevice, SessionEventKind, SessionHandle, SessionSnapshot};
use crate::setup::Setup;
use crate::store::notice::{NoticeEvent, StoreNotice};
use crate::store::summary::{
    ExportReport, ImportError, ImportReport, ProfileDetails, StoreSummary,
};
use crate::store::{DeviceNames, FileOps, ProfileStore, StoreError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ApplyOutcome {
    Applied(ApplyReport),
    /// Not connected; the profile is marked to apply on connect, so it applies then.
    WaitingForConnection,
}

/// A profile written from the session, with what happened to each setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Saved {
    pub id: ProfileId,
    pub changes: Vec<SettingChange>,
    pub frame_rate: Option<FrameRateChange>,
}

/// The store for async callers: every file operation runs on a blocking thread, never in the session task.
#[derive(Clone)]
pub struct Profiles {
    store: Arc<Mutex<ProfileStore>>,
    active: Arc<Mutex<Option<Active>>>,
}

/// The profile last applied or written from one session; user records after `baseline` are not in it yet.
#[derive(Debug, Clone)]
struct Active {
    session: u64,
    profile: ProfileId,
    baseline: u64,
}

impl Profiles {
    pub async fn open(root: PathBuf, ops: impl FileOps + 'static) -> Result<Self, StoreError> {
        let store = tokio::task::spawn_blocking(move || ProfileStore::open(root, ops))
            .await
            .expect("opening the store does not panic")?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            active: Arc::new(Mutex::new(None)),
        })
    }

    /// With `handle`, also matches its devices and counts the settings an update of the active profile would change.
    pub async fn summary(&self, handle: Option<&SessionHandle>) -> StoreSummary {
        let devices = handle.map(|handle| handle.snapshot().devices);
        let active = handle.and_then(|handle| self.active_in(handle));
        let unsaved = match (handle, &active) {
            (Some(handle), Some(active)) => {
                self.pending(handle, &active.profile)
                    .await
                    .ok()
                    .map(|(_, updated)| {
                        updated.changes.len() + usize::from(updated.frame_rate.is_some())
                    })
            }
            _ => None,
        };
        let mut summary = self
            .with(move |store| store.summary(devices.as_deref()))
            .await;
        summary.active = active.map(|active| active.profile);
        summary.unsaved_changes = unsaved;
        summary
    }

    pub async fn details(
        &self,
        id: &ProfileId,
        handle: Option<&SessionHandle>,
    ) -> Result<ProfileDetails, StoreError> {
        let now = handle.map(SessionHandle::snapshot);
        let id = id.clone();
        self.with(move |store| store.details(&id, now.as_ref()))
            .await
    }

    /// The name of the profile applied or saved in this session; asks the session nothing.
    pub async fn active_name(&self, handle: &SessionHandle) -> Option<String> {
        let id = self.active_in(handle)?.profile;
        self.with(move |store| store.profile(&id).ok().map(|profile| profile.name.clone()))
            .await
    }

    pub async fn profile(&self, id: &ProfileId) -> Result<Profile, StoreError> {
        let id = id.clone();
        self.with(move |store| store.profile(&id).cloned()).await
    }

    /// Saves the session as a new profile: the active profile plus the user's changes since, or what runs now.
    pub async fn create_from_session(
        &self,
        handle: &SessionHandle,
        name: &str,
    ) -> Result<Saved, StoreError> {
        let active = self.active_in(handle);
        let log = intents(
            handle,
            active.as_ref().map_or(u64::MAX, |active| active.baseline),
        )
        .await?;
        let snapshot = handle.snapshot();
        let name = name.to_owned();
        let sequence = log.sequence;
        let (id, updated) = self
            .with(move |store| {
                let updated = match active {
                    Some(active) => {
                        let stored = store.profile(&active.profile)?.clone();
                        let mut profile =
                            updated_from(store, &stored, &log, &snapshot.devices).profile;
                        profile.name = name;
                        Updated {
                            changes: all_added(&profile),
                            frame_rate: None,
                            profile,
                        }
                    }
                    None => {
                        let profile = Profile::capture(&snapshot, &name, &names_in(store));
                        Updated {
                            changes: all_added(&profile),
                            frame_rate: None,
                            profile,
                        }
                    }
                };
                let id = store.create(updated.profile.clone())?;
                Ok::<_, StoreError>((id, updated))
            })
            .await?;
        Ok(self.activate(handle, id, sequence, updated))
    }

    /// Adds the user's changes since the apply or the last save to the active profile `id`; nothing else changes.
    pub async fn update_from_session(
        &self,
        handle: &SessionHandle,
        id: &ProfileId,
    ) -> Result<Saved, StoreError> {
        let (log, updated) = self.pending(handle, id).await?;
        if !updated.changes.is_empty() || updated.frame_rate.is_some() {
            let key = id.clone();
            let profile = updated.profile.clone();
            self.with(move |store| store.update(&key, profile)).await?;
        }
        Ok(self.activate(handle, id.clone(), log.sequence, updated))
    }

    /// What `update_from_session` would save now, without writing anything.
    pub async fn preview_update(
        &self,
        handle: &SessionHandle,
        id: &ProfileId,
    ) -> Result<Saved, StoreError> {
        let (_, updated) = self.pending(handle, id).await?;
        Ok(Saved {
            id: id.clone(),
            changes: updated.changes,
            frame_rate: updated.frame_rate,
        })
    }

    async fn pending(
        &self,
        handle: &SessionHandle,
        id: &ProfileId,
    ) -> Result<(IntentLog, Updated), StoreError> {
        let active = self.active_in(handle);
        let Some(active) = active.filter(|active| &active.profile == id) else {
            return Err(StoreError::NotActive {
                id: id.clone(),
                active: self.active_in(handle).map(|active| active.profile),
            });
        };
        let log = intents(handle, active.baseline).await?;
        let devices = handle.snapshot().devices;
        let key = id.clone();
        self.with(move |store| {
            let stored = store.profile(&key)?.clone();
            let updated = updated_from(store, &stored, &log, &devices);
            Ok((log, updated))
        })
        .await
    }

    /// Replaces the content of profile `id`; its name stays.
    pub async fn replace(&self, id: &ProfileId, profile: Profile) -> Result<(), StoreError> {
        let id = id.clone();
        self.with(move |store| store.update(&id, profile)).await
    }

    pub async fn rename(&self, id: &ProfileId, name: &str) -> Result<(), StoreError> {
        let (id, name) = (id.clone(), name.to_owned());
        self.with(move |store| store.rename(&id, &name)).await
    }

    pub async fn duplicate(&self, id: &ProfileId) -> Result<ProfileId, StoreError> {
        let id = id.clone();
        self.with(move |store| store.duplicate(&id)).await
    }

    pub async fn delete(&self, id: &ProfileId) -> Result<(), StoreError> {
        let key = id.clone();
        self.with(move |store| store.delete(&key)).await?;
        self.lock_active().take_if(|active| &active.profile == id);
        Ok(())
    }

    pub async fn set_apply_on_connect(&self, id: Option<ProfileId>) -> Result<(), StoreError> {
        self.with(move |store| store.set_apply_on_connect(id)).await
    }

    /// Applies profile `id` now; while not connected only a profile marked to apply on connect waits.
    pub async fn apply(
        &self,
        handle: &SessionHandle,
        id: &ProfileId,
    ) -> Result<ApplyOutcome, ApplyError> {
        let key = id.clone();
        let (profile, marked, setup) = self
            .with(move |store| {
                let profile = store.profile(&key)?.clone();
                let marked = store.apply_on_connect() == Some(&key);
                Ok::<_, StoreError>((profile, marked, store.setup().ok().cloned()))
            })
            .await?;
        if marked && !is_live(&handle.snapshot().connection) {
            return Ok(ApplyOutcome::WaitingForConnection);
        }
        let report = apply(handle, &profile, setup.as_ref()).await?;
        *self.lock_active() = intents(handle, u64::MAX).await.ok().map(|log| Active {
            session: handle.session_id(),
            profile: id.clone(),
            baseline: log.sequence,
        });
        Ok(ApplyOutcome::Applied(report))
    }

    /// Applies the profile marked to apply on connect; `None` when no profile is marked.
    pub async fn apply_marked(
        &self,
        handle: &SessionHandle,
    ) -> Result<Option<ApplyReport>, ApplyError> {
        let Some(id) = self.with(|store| store.apply_on_connect().cloned()).await else {
            return Ok(None);
        };
        match self.apply(handle, &id).await? {
            ApplyOutcome::Applied(report) => Ok(Some(report)),
            ApplyOutcome::WaitingForConnection => Err(ApplyError::NotConnected),
        }
    }

    /// Writes the portable form of profile `id` to `destination`, with the same durable steps as the store.
    pub async fn export_to(
        &self,
        id: &ProfileId,
        destination: PathBuf,
    ) -> Result<ExportReport, StoreError> {
        let id = id.clone();
        self.with(move |store| store.export_to(&id, &destination))
            .await
    }

    /// Reads `source` and stores it as a new profile; writes nothing to hardware.
    pub async fn import_from(
        &self,
        source: PathBuf,
        handle: Option<&SessionHandle>,
    ) -> Result<ImportReport, ImportError> {
        let now = handle.map(SessionHandle::snapshot);
        self.with(move |store| store.import_file(&source, now.as_ref()))
            .await
    }

    /// The user's names for devices and zones; devices without any name are left out.
    pub async fn names(&self) -> Vec<DeviceNames> {
        self.with(|store| store.registry().names()).await
    }

    pub async fn dismiss_notice(&self, notice: StoreNotice) -> bool {
        self.with(move |store| store.dismiss(&notice)).await
    }

    pub async fn known_devices(&self) -> Vec<KnownDevice> {
        self.with(|store| store.known_devices()).await
    }

    pub async fn record_known_devices(&self, known: Vec<KnownDevice>) -> Result<bool, StoreError> {
        self.with(move |store| store.record_known_devices(&known))
            .await
    }

    pub async fn set_device_name(
        &self,
        device: &DeviceRef,
        name: Option<String>,
    ) -> Result<(), StoreError> {
        let device = device.clone();
        self.with(move |store| store.set_device_name(&device, name.as_deref()))
            .await
    }

    pub async fn set_zone_name(
        &self,
        device: &DeviceRef,
        zone: usize,
        name: Option<String>,
    ) -> Result<(), StoreError> {
        let device = device.clone();
        self.with(move |store| store.set_zone_name(&device, zone, name.as_deref()))
            .await
    }

    /// The stored setup; refused with `NewerFile` while its file needs a newer PrismForge.
    pub async fn setup(&self) -> Result<Setup, StoreError> {
        self.with(|store| store.setup().cloned()).await
    }

    /// Stores `setup`; outputs that are new or changed must fit the devices the session reports now.
    pub async fn save_setup(
        &self,
        setup: Setup,
        handle: Option<&SessionHandle>,
    ) -> Result<(), StoreError> {
        let devices = handle.map_or_else(Vec::new, |handle| handle.snapshot().devices);
        self.with(move |store| store.save_setup(setup, &devices))
            .await
    }

    /// Writes the session's registry each time it changes, until the session stops.
    pub async fn keep_registry(&self, mut known: watch::Receiver<Vec<KnownDevice>>) {
        let mut ended = false;
        loop {
            let devices = known.borrow_and_update().clone();
            if let Err(StoreError::Write(failure)) = self.record_known_devices(devices).await {
                self.with(move |store| {
                    store.note(StoreNotice {
                        file: failure.file.clone(),
                        event: NoticeEvent::WriteFailed { failure },
                    });
                })
                .await;
            }
            if ended {
                return;
            }
            ended = known.changed().await.is_err();
        }
    }

    fn active_in(&self, handle: &SessionHandle) -> Option<Active> {
        self.lock_active()
            .clone()
            .filter(|active| active.session == handle.session_id())
    }

    fn activate(
        &self,
        handle: &SessionHandle,
        id: ProfileId,
        baseline: u64,
        updated: Updated,
    ) -> Saved {
        *self.lock_active() = Some(Active {
            session: handle.session_id(),
            profile: id.clone(),
            baseline,
        });
        Saved {
            id,
            changes: updated.changes,
            frame_rate: updated.frame_rate,
        }
    }

    fn lock_active(&self) -> std::sync::MutexGuard<'_, Option<Active>> {
        self.active.lock().unwrap_or_else(PoisonError::into_inner)
    }

    async fn with<T: Send + 'static>(
        &self,
        work: impl FnOnce(&mut ProfileStore) -> T + Send + 'static,
    ) -> T {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            work(&mut store.lock().unwrap_or_else(PoisonError::into_inner))
        })
        .await
        .expect("store work does not panic")
    }
}

/// Says when the apply-on-connect profile is due: once each time the session becomes live after a disconnect.
#[derive(Debug, Default)]
pub struct ConnectTrigger {
    applied: bool,
    seen: Option<u64>,
}

impl ConnectTrigger {
    /// True once it came due in this connect; it is armed again only after a disconnect.
    pub fn fired(&self) -> bool {
        self.applied
    }

    pub fn due(&mut self, snapshot: &SessionSnapshot) -> bool {
        let disconnected = snapshot
            .diagnostics
            .events
            .iter()
            .filter(|event| self.seen.is_none_or(|seen| event.sequence > seen))
            .any(|event| event.kind == SessionEventKind::Disconnected);
        if let Some(last) = snapshot.diagnostics.events.last() {
            self.seen = Some(last.sequence);
        }
        if disconnected || matches!(snapshot.connection, ConnectionState::Disconnected { .. }) {
            self.applied = false;
        }
        if is_live(&snapshot.connection) && !self.applied {
            self.applied = true;
            return true;
        }
        false
    }
}

fn names_in(store: &ProfileStore) -> impl Fn(&DeviceRef) -> Option<String> + '_ {
    let registry = store.registry();
    move |device: &DeviceRef| registry.device(device).and_then(|entry| entry.name.clone())
}

fn updated_from(
    store: &ProfileStore,
    stored: &Profile,
    log: &IntentLog,
    devices: &[SessionDevice],
) -> Updated {
    update(stored, &log.records, devices, &names_in(store))
}

async fn intents(handle: &SessionHandle, after: u64) -> Result<IntentLog, StoreError> {
    handle
        .intents_after(after)
        .await
        .map_err(|error| StoreError::SessionUnavailable { error })
}
