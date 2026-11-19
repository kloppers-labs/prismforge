use prismforge_core::OutputTarget;
use prismforge_core::store::GroupId;
use tauri::State;

use crate::DesktopShell;
use crate::groups::GroupsView;
use crate::view::CommandError;

#[tauri::command]
pub async fn groups(shell: State<'_, DesktopShell>) -> Result<GroupsView, CommandError> {
    shell.groups_view().await
}

#[tauri::command]
pub async fn create_group(
    shell: State<'_, DesktopShell>,
    name: String,
    targets: Vec<OutputTarget>,
) -> Result<GroupsView, CommandError> {
    shell.create_group(&name, targets).await
}

#[tauri::command]
pub async fn rename_group(
    shell: State<'_, DesktopShell>,
    id: GroupId,
    name: String,
) -> Result<GroupsView, CommandError> {
    shell.rename_group(&id, &name).await
}

#[tauri::command]
pub async fn add_group_members(
    shell: State<'_, DesktopShell>,
    id: GroupId,
    targets: Vec<OutputTarget>,
) -> Result<GroupsView, CommandError> {
    shell.add_group_members(&id, targets).await
}

#[tauri::command]
pub async fn remove_group_member(
    shell: State<'_, DesktopShell>,
    id: GroupId,
    target: OutputTarget,
) -> Result<GroupsView, CommandError> {
    shell.remove_group_member(&id, target).await
}

#[tauri::command]
pub async fn delete_group(
    shell: State<'_, DesktopShell>,
    id: GroupId,
) -> Result<GroupsView, CommandError> {
    shell.delete_group(&id).await
}
