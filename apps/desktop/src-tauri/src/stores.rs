use std::fs::{self, File, TryLockError};
use std::path::{Path, PathBuf};

use prismforge_core::BackendOrigin;
use prismforge_core::store::{DeviceNames, Profiles, StdFileOps, StoreSummary};
use serde::Serialize;
use tokio::sync::Mutex;

const REAL_FOLDER: &str = "store";
const DEMO_FOLDER: &str = "demo-store";
const LOCK_FILE: &str = "instance.lock";

/// Demo sessions use their own store, so a profile of simulated devices never meets real hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StoreKind {
    Real,
    Demo,
}

impl StoreKind {
    pub fn of(origin: BackendOrigin) -> Self {
        match origin {
            BackendOrigin::Simulated => Self::Demo,
            _ => Self::Real,
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Self::Real => REAL_FOLDER,
            Self::Demo => DEMO_FOLDER,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StoreUnavailable {
    /// Another PrismForge holds the store folder; this one reads and writes none of it.
    InUse,
    Folder {
        detail: String,
    },
}

/// The store the window works with now, with the user's names and the core's name limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct StoreView {
    pub kind: StoreKind,
    pub folder: String,
    pub summary: StoreSummary,
    pub names: Vec<DeviceNames>,
    pub name_limit: usize,
}

/// The real and the demo store below one folder; each opens on first use, after the folder lock is held.
pub struct Stores {
    root: PathBuf,
    lock: Mutex<Option<File>>,
    real: Mutex<Option<Profiles>>,
    demo: Mutex<Option<Profiles>>,
}

impl Stores {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            lock: Mutex::new(None),
            real: Mutex::new(None),
            demo: Mutex::new(None),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn folder(&self, kind: StoreKind) -> PathBuf {
        self.root.join(kind.folder())
    }

    pub async fn profiles(&self, kind: StoreKind) -> Result<Profiles, StoreUnavailable> {
        let mut slot = match kind {
            StoreKind::Real => self.real.lock().await,
            StoreKind::Demo => self.demo.lock().await,
        };
        if let Some(profiles) = slot.as_ref() {
            return Ok(profiles.clone());
        }
        self.hold_lock().await?;
        let profiles = Profiles::open(self.folder(kind), StdFileOps)
            .await
            .map_err(|error| StoreUnavailable::Folder {
                detail: error.to_string(),
            })?;
        Ok(slot.insert(profiles).clone())
    }

    async fn hold_lock(&self) -> Result<(), StoreUnavailable> {
        let mut lock = self.lock.lock().await;
        if lock.is_none() {
            let root = self.root.clone();
            let file = tokio::task::spawn_blocking(move || lock_folder(&root))
                .await
                .map_err(|error| StoreUnavailable::Folder {
                    detail: error.to_string(),
                })??;
            *lock = Some(file);
        }
        Ok(())
    }
}

/// The OS releases the lock when the process ends, so a crash never leaves the folder locked.
fn lock_folder(root: &Path) -> Result<File, StoreUnavailable> {
    let failed = |error: std::io::Error| StoreUnavailable::Folder {
        detail: error.to_string(),
    };
    fs::create_dir_all(root).map_err(failed)?;
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join(LOCK_FILE))
        .map_err(failed)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(StoreUnavailable::InUse),
        Err(TryLockError::Error(error)) => Err(failed(error)),
    }
}
