use prismforge_core::DeviceRef;
use prismforge_core::session::IdentifyOutcome;
use prismforge_openrgb::{ClientConfig, OpenRgbBackend, TcpConnector};
use prismforge_testkit::DemoBackend;
use tauri::State;

use crate::DesktopShell;
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
