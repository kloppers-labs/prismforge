use prismforge_core::OutputTarget;
use prismforge_core::setup::{Setup, ViewId};
use prismforge_effects::{EffectInstance, Flow};
use tauri::State;

use crate::DesktopShell;
use crate::setup::{AcrossPreview, AcrossPreviewRequest, AcrossReport, SetupStoreView};
use crate::view::CommandError;

#[tauri::command]
pub async fn setup(shell: State<'_, DesktopShell>) -> Result<SetupStoreView, CommandError> {
    shell.setup_view().await
}

#[tauri::command]
pub async fn save_setup(
    shell: State<'_, DesktopShell>,
    setup: Setup,
) -> Result<SetupStoreView, CommandError> {
    shell.save_setup(setup).await
}

#[tauri::command]
pub async fn apply_across(
    shell: State<'_, DesktopShell>,
    view: ViewId,
    flow: Flow,
    effect: EffectInstance,
    targets: Option<Vec<OutputTarget>>,
) -> Result<AcrossReport, CommandError> {
    shell.apply_across(view, flow, effect, targets).await
}

#[tauri::command]
pub fn preview_across(request: AcrossPreviewRequest) -> Result<AcrossPreview, CommandError> {
    crate::preview_across(request)
}
