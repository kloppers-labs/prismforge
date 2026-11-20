use prismforge_core::automation::{RuleAction, Trigger};
use tauri::State;

use crate::automation::AutomationView;
use crate::view::CommandError;
use crate::{DesktopAutomation, DesktopLifecycle, DesktopShell, marks};

#[tauri::command]
pub async fn automation(
    shell: State<'_, DesktopShell>,
    automation: State<'_, DesktopAutomation>,
) -> Result<AutomationView, CommandError> {
    automation.view(&shell).await
}

#[tauri::command]
pub async fn set_rule(
    shell: State<'_, DesktopShell>,
    lifecycle: State<'_, DesktopLifecycle>,
    automation: State<'_, DesktopAutomation>,
    trigger: Trigger,
    enabled: bool,
    action: RuleAction,
) -> Result<AutomationView, CommandError> {
    marks::set_rule(&shell, &lifecycle, &automation, trigger, enabled, action).await
}
