use prismforge_core::session::{IdentifyOutcome, RestoreOutcome};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::{EffectInstance, PreviewBatch, PreviewRequest};
use prismforge_openrgb::{ClientConfig, OpenRgbBackend, TcpConnector};
use prismforge_testkit::DemoBackend;
use tauri::State;

use crate::effects::EffectCatalog;
use crate::view::{CommandError, ShellView};
use crate::{DesktopLifecycle, DesktopShell, marks};

#[tauri::command]
pub fn view(shell: State<'_, DesktopShell>) -> ShellView {
    shell.view()
}

/// A store failure does not stop the connect; only connect at start needs the remembered address.
#[tauri::command]
pub async fn connect_openrgb(
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
    host: String,
    port: u16,
) -> Result<ShellView, CommandError> {
    let host = host.trim();
    if host.is_empty() {
        return Err(CommandError::EmptyHost);
    }
    marks::connect_openrgb(&shell, &lifecycle, host, port, openrgb(host, port)).await
}

/// The one connect path, for the Connect button and for connect at start.
pub async fn connect(
    shell: &DesktopShell,
    host: &str,
    port: u16,
) -> Result<ShellView, CommandError> {
    shell.start(openrgb(host, port)).await
}

fn openrgb(host: &str, port: u16) -> OpenRgbBackend<TcpConnector> {
    OpenRgbBackend::new(TcpConnector::new(host, port), ClientConfig::default())
}

#[tauri::command]
pub async fn start_demo(
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
) -> Result<ShellView, CommandError> {
    marks::start(&shell, &lifecycle, DemoBackend::new()).await
}

#[tauri::command]
pub async fn disconnect(
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
) -> Result<ShellView, CommandError> {
    Ok(marks::stop(&shell, &lifecycle).await)
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
