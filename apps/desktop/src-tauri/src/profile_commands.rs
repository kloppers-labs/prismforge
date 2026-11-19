use std::path::PathBuf;

use prismforge_core::DeviceRef;
use prismforge_core::profile::ProfileId;
use prismforge_core::store::{
    ApplyOutcome, ExportReport, ImportReport, ProfileDetails, Saved, StoreNotice,
};
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};
use tokio::sync::oneshot;

use crate::DesktopShell;
use crate::stores::StoreView;
use crate::view::{CommandError, FileRule};

const FILTER_NAME: &str = "PrismForge profile";
const FILTER_EXTENSIONS: &[&str] = &["json"];

#[tauri::command]
pub async fn profile_store(shell: State<'_, DesktopShell>) -> Result<StoreView, CommandError> {
    shell.store_view().await
}

#[tauri::command]
pub async fn profile_details(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<ProfileDetails, CommandError> {
    shell.profile_details(&id).await
}

#[tauri::command]
pub async fn create_profile(
    shell: State<'_, DesktopShell>,
    name: String,
) -> Result<Saved, CommandError> {
    shell.create_profile(&name).await
}

#[tauri::command]
pub async fn preview_profile_update(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<Saved, CommandError> {
    shell.preview_update(&id).await
}

#[tauri::command]
pub async fn update_profile(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<Saved, CommandError> {
    shell.update_profile(&id).await
}

#[tauri::command]
pub async fn rename_profile(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
    name: String,
) -> Result<(), CommandError> {
    shell.rename_profile(&id, &name).await
}

#[tauri::command]
pub async fn duplicate_profile(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<ProfileId, CommandError> {
    shell.duplicate_profile(&id).await
}

#[tauri::command]
pub async fn delete_profile(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<(), CommandError> {
    shell.delete_profile(&id).await
}

#[tauri::command]
pub async fn apply_profile(
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<ApplyOutcome, CommandError> {
    shell.apply_profile(&id).await
}

#[tauri::command]
pub async fn set_apply_on_connect(
    shell: State<'_, DesktopShell>,
    id: Option<ProfileId>,
) -> Result<(), CommandError> {
    shell.set_apply_on_connect(id).await
}

/// The WebView never names a path: Rust opens the native dialog and uses only the path it returns.
#[tauri::command]
pub async fn export_profile(
    app: AppHandle,
    window: WebviewWindow,
    shell: State<'_, DesktopShell>,
    id: ProfileId,
) -> Result<Option<ExportReport>, CommandError> {
    let name = shell.export_file_name(&id).await?;
    let dialog = file_dialog(&app, &window).set_file_name(name);
    let (sender, chosen) = oneshot::channel();
    dialog.save_file(move |path| {
        let _ = sender.send(path);
    });
    match path_of(chosen.await)? {
        Some(path) => shell.export_profile(&id, path).await.map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn import_profile(
    app: AppHandle,
    window: WebviewWindow,
    shell: State<'_, DesktopShell>,
) -> Result<Option<ImportReport>, CommandError> {
    let (sender, chosen) = oneshot::channel();
    file_dialog(&app, &window).pick_file(move |path| {
        let _ = sender.send(path);
    });
    match path_of(chosen.await)? {
        Some(path) => shell.import_profile(path).await.map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn set_device_name(
    shell: State<'_, DesktopShell>,
    device: DeviceRef,
    name: Option<String>,
) -> Result<(), CommandError> {
    shell.set_device_name(device, name).await
}

#[tauri::command]
pub async fn set_zone_name(
    shell: State<'_, DesktopShell>,
    device: DeviceRef,
    zone: usize,
    name: Option<String>,
) -> Result<(), CommandError> {
    shell.set_zone_name(device, zone, name).await
}

#[tauri::command]
pub async fn dismiss_store_notice(
    shell: State<'_, DesktopShell>,
    notice: StoreNotice,
) -> Result<bool, CommandError> {
    shell.dismiss_notice(notice).await
}

fn file_dialog(app: &AppHandle, window: &WebviewWindow) -> FileDialogBuilder<tauri::Wry> {
    app.dialog()
        .file()
        .set_parent(window)
        .add_filter(FILTER_NAME, FILTER_EXTENSIONS)
}

fn path_of(
    chosen: Result<Option<FilePath>, oneshot::error::RecvError>,
) -> Result<Option<PathBuf>, CommandError> {
    let Ok(Some(path)) = chosen else {
        return Ok(None);
    };
    path.into_path().map(Some).map_err(|error| {
        CommandError::File(FileRule::Unreadable {
            detail: error.to_string(),
        })
    })
}
