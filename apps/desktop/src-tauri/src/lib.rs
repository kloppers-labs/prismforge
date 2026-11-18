mod commands;
mod effects;
mod exit;
mod forward;
mod publisher;
mod shell;
mod view;

use tauri::{AppHandle, Emitter, Manager, RunEvent};

pub use effects::{EffectCatalog, preview_effect};
pub use exit::{ExitGuard, stop_then_exit};
pub use forward::forward;
pub use publisher::{Publisher, ViewSink};
pub use shell::{Shell, ShellConfig};
pub use view::{CommandError, ShellView};

pub const VIEW_EVENT: &str = "session-view";

type DesktopShell = Shell<WindowSink>;

pub struct WindowSink(AppHandle);

impl ViewSink for WindowSink {
    fn deliver(&self, view: &ShellView) {
        let _ = self.0.emit(VIEW_EVENT, view);
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let sink = WindowSink(app.handle().clone());
            app.manage(Shell::new(sink, ShellConfig::default()));
            app.manage(ExitGuard::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::view,
            commands::connect_openrgb,
            commands::start_demo,
            commands::disconnect,
            commands::refresh,
            commands::identify,
            commands::effect_catalog,
            commands::apply_effect,
            commands::stop_effect,
            commands::select_mode,
            commands::declare_external,
            commands::clear_external,
            commands::set_frame_rate,
            commands::preview_effect,
        ])
        .build(tauri::generate_context!())
        .expect("PrismForge could not start")
        .run(on_run_event);
}

fn on_run_event(app: &AppHandle, event: RunEvent) {
    let RunEvent::ExitRequested { api, .. } = event else {
        return;
    };
    if !app.state::<ExitGuard>().hold() {
        return;
    }
    api.prevent_exit();
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let shell = app.state::<DesktopShell>();
        stop_then_exit(&shell, || app.exit(0)).await;
    });
}
