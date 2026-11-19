use prismforge_core::store::CloseAction;
use tauri::{AppHandle, State};

use crate::lifecycle::{AppSettingChange, LifecycleView};
use crate::native::{AppExit, MainWindow};
use crate::view::CommandError;
use crate::{DesktopLifecycle, DesktopShell};

#[tauri::command]
pub async fn lifecycle(
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
) -> Result<LifecycleView, CommandError> {
    Ok(lifecycle.publish(&shell).await)
}

#[tauri::command]
pub async fn change_setting(
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
    change: AppSettingChange,
) -> Result<LifecycleView, CommandError> {
    lifecycle.change(&shell, change).await
}

#[tauri::command]
pub async fn answer_close(
    app: AppHandle,
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
    choice: Option<CloseAction>,
) -> Result<(), CommandError> {
    let window = MainWindow::of(&app);
    lifecycle
        .answer_close(&shell, choice, &window, &AppExit(app.clone()))
        .await
}

#[tauri::command]
pub async fn quit(
    app: AppHandle,
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
) -> Result<(), CommandError> {
    let window = MainWindow::of(&app);
    lifecycle.quit(&shell, &window, &AppExit(app.clone())).await;
    Ok(())
}
