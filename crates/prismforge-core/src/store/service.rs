use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::connection::ConnectionState;
use crate::device::DeviceRef;
use crate::identity::KnownDevice;
use crate::profile::{ApplyError, ApplyReport, Profile, ProfileId, apply, is_live};
use crate::session::{SessionDevice, SessionEventKind, SessionHandle, SessionSnapshot};
use crate::store::notice::{NoticeEvent, StoreNotice};
use crate::store::summary::{
    ExportReport, ImportError, ImportReport, ProfileDetails, StoreSummary,
};
use crate::store::{FileOps, ProfileStore, StoreError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ApplyOutcome {
    Applied(ApplyReport),
    /// Not connected; the profile is marked to apply on connect, so it applies then.
    WaitingForConnection,
}

/// The store for async callers: every file operation runs on a blocking thread, never in the session task.
#[derive(Clone)]
pub struct Profiles {
    store: Arc<Mutex<ProfileStore>>,
    active: Arc<Mutex<Option<ProfileId>>>,
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

    pub async fn summary(&self, handle: Option<&SessionHandle>) -> StoreSummary {
        let devices = handle.map(|handle| handle.snapshot().devices);
        self.with(move |store| store.summary(devices.as_deref()))
            .await
    }

    pub async fn details(
        &self,
        id: &ProfileId,
        handle: Option<&SessionHandle>,
    ) -> Result<ProfileDetails, StoreError> {
        let devices = devices_of(handle);
        let id = id.clone();
        self.with(move |store| store.details(&id, &devices)).await
    }

    pub async fn profile(&self, id: &ProfileId) -> Result<Profile, StoreError> {
        let id = id.clone();
        self.with(move |store| store.profile(&id).cloned()).await
    }

    /// Captures the session as a new profile; devices of the last applied profile that are not here now stay in it.
    pub async fn create_from_session(
        &self,
        handle: &SessionHandle,
        name: &str,
    ) -> Result<ProfileId, StoreError> {
        let snapshot = handle.snapshot();
        let name = name.to_owned();
        let active = self.active();
        self.with(move |store| {
            let base = active.and_then(|id| store.profile(&id).ok().cloned());
            let profile = capture(store, &snapshot, &name, base.as_ref());
            store.create(profile)
        })
        .await
    }

    /// Captures the session into profile `id`, keeping its devices that are not here now.
    pub async fn update_from_session(
        &self,
        handle: &SessionHandle,
        id: &ProfileId,
    ) -> Result<(), StoreError> {
        let snapshot = handle.snapshot();
        let id = id.clone();
        self.with(move |store| {
            let base = store.profile(&id)?.clone();
            let profile = capture(store, &snapshot, &base.name, Some(&base));
            store.update(&id, profile)
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
        let id = id.clone();
        self.with(move |store| store.delete(&id)).await
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
        let (profile, marked) = self
            .with(move |store| {
                let profile = store.profile(&key)?.clone();
                Ok::<_, StoreError>((profile, store.apply_on_connect() == Some(&key)))
            })
            .await?;
        if marked && !is_live(&handle.snapshot().connection) {
            return Ok(ApplyOutcome::WaitingForConnection);
        }
        let report = apply(handle, &profile).await?;
        *self.active.lock().unwrap_or_else(PoisonError::into_inner) = Some(id.clone());
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
        let devices = devices_of(handle);
        self.with(move |store| store.import_file(&source, &devices))
            .await
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

    /// Writes the session's registry each time it changes, until the session stops.
    pub async fn keep_registry(&self, mut known: watch::Receiver<Vec<KnownDevice>>) {
        known.mark_unchanged();
        loop {
            let ended = known.changed().await.is_err();
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
        }
    }

    fn active(&self) -> Option<ProfileId> {
        self.active
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
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

fn capture(
    store: &ProfileStore,
    snapshot: &SessionSnapshot,
    name: &str,
    base: Option<&Profile>,
) -> Profile {
    let registry = store.registry();
    let names = |device: &DeviceRef| registry.device(device).and_then(|entry| entry.name.clone());
    Profile::capture(snapshot, name, base, &names)
}

fn devices_of(handle: Option<&SessionHandle>) -> Vec<SessionDevice> {
    handle
        .map(|handle| handle.snapshot().devices)
        .unwrap_or_default()
}
