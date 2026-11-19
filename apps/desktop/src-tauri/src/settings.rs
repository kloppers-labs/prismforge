use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use prismforge_core::store::{AppSettings, AppSettingsFile, StdFileOps, StoreError, StoreNotice};

/// The app settings file for async callers; every file operation runs on a blocking thread.
#[derive(Clone)]
pub struct SettingsStore {
    file: Arc<Mutex<AppSettingsFile>>,
}

impl SettingsStore {
    pub async fn open(root: PathBuf) -> Self {
        let file = tokio::task::spawn_blocking(move || AppSettingsFile::open(&root, StdFileOps))
            .await
            .expect("opening the settings does not panic");
        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }

    pub async fn read(&self) -> (AppSettings, Vec<StoreNotice>) {
        self.with(|file| (file.settings().clone(), file.notices().to_vec()))
            .await
    }

    pub async fn change(
        &self,
        edit: impl FnOnce(&mut AppSettings) + Send + 'static,
    ) -> Result<(), StoreError> {
        self.with(move |file| file.change(edit)).await
    }

    async fn with<T: Send + 'static>(
        &self,
        work: impl FnOnce(&mut AppSettingsFile) -> T + Send + 'static,
    ) -> T {
        let file = self.file.clone();
        tokio::task::spawn_blocking(move || {
            work(&mut file.lock().unwrap_or_else(PoisonError::into_inner))
        })
        .await
        .expect("settings work does not panic")
    }
}
