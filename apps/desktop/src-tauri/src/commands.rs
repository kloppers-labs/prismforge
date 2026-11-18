use prismforge_core::session::{IdentifyOutcome, RestoreOutcome};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::{EffectInstance, PreviewBatch, PreviewRequest};
use prismforge_openrgb::{ClientConfig, OpenRgbBackend, TcpConnector};
use prismforge_testkit::DemoBackend;
use tauri::State;

use crate::DesktopShell;
use crate::effects::EffectCatalog;
use crate::view::{CommandError, ShellView};

#[tauri::command]
pub fn view(shell: State<'_, DesktopShell>) -> ShellView {
    shell.view()
}

#[tauri::command]
pub async fn connect_openrgb(
    shell: State<'_, DesktopShell>,
    host: String,
    port: u16,
) -> Result<ShellView, CommandError> {
    let host = host.trim();
    if host.is_empty() {
        return Err(CommandError::EmptyHost);
    }
    let connector = TcpConnector::new(host, port);
    shell
        .start(OpenRgbBackend::new(connector, ClientConfig::default()))
        .await
}

#[tauri::command]
pub async fn start_demo(shell: State<'_, DesktopShell>) -> Result<ShellView, CommandError> {
    shell.start(DemoBackend::new()).await
}

#[tauri::command]
pub async fn disconnect(shell: State<'_, DesktopShell>) -> Result<ShellView, CommandError> {
    Ok(shell.stop().await)
}

#[tauri::command]
pub async fn refresh(shell: State<'_, DesktopShell>) -> Result<(), CommandError> {
    shell.refresh().await
}

#[tauri::command]
pub async fn identify(
    shell: State<'_, DesktopShell>,
    device: DeviceRef,
    zone: Option<usize>,
) -> Result<IdentifyOutcome, CommandError> {
    shell.identify(device, zone).await
}

#[tauri::command]
pub async fn effect_catalog(shell: State<'_, DesktopShell>) -> Result<EffectCatalog, CommandError> {
    shell.catalog().await
}

#[tauri::command]
pub async fn apply_effect(
    shell: State<'_, DesktopShell>,
    target: OutputTarget,
    effect: EffectInstance,
) -> Result<(), CommandError> {
    shell.apply_effect(target, effect).await
}

#[tauri::command]
pub async fn stop_effect(
    shell: State<'_, DesktopShell>,
    target: OutputTarget,
) -> Result<Option<RestoreOutcome>, CommandError> {
    shell.stop_effect(target).await
}

#[tauri::command]
pub async fn select_mode(
    shell: State<'_, DesktopShell>,
    device: DeviceRef,
    mode: usize,
) -> Result<(), CommandError> {
    shell.select_mode(device, mode).await
}

#[tauri::command]
pub async fn declare_external(
    shell: State<'_, DesktopShell>,
    device: DeviceRef,
    label: String,
) -> Result<(), CommandError> {
    shell.declare_external(device, label).await
}

#[tauri::command]
pub async fn clear_external(
    shell: State<'_, DesktopShell>,
    device: DeviceRef,
) -> Result<(), CommandError> {
    shell.clear_external(device).await
}

#[tauri::command]
pub async fn set_frame_rate(
    shell: State<'_, DesktopShell>,
    frame_rate: u32,
) -> Result<u32, CommandError> {
    shell.set_frame_rate(frame_rate).await
}

#[tauri::command]
pub fn preview_effect(request: PreviewRequest) -> Result<PreviewBatch, CommandError> {
    crate::preview_effect(request)
}
