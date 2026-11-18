mod commands;
mod forward;
mod publisher;
mod shell;
mod view;

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, RunEvent};

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

#[derive(Default)]
struct Exit {
    stopping: AtomicBool,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let sink = WindowSink(app.handle().clone());
            app.manage(Shell::new(sink, ShellConfig::default()));
            app.manage(Exit::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::view,
            commands::connect_openrgb,
            commands::start_demo,
            commands::disconnect,
            commands::refresh,
            commands::identify,
        ])
        .build(tauri::generate_context!())
        .expect("PrismForge could not start")
        .run(on_run_event);
}

/// Holds the exit until the session has stopped, so a running identify is restored first.
fn on_run_event(app: &AppHandle, event: RunEvent) {
    let RunEvent::ExitRequested { api, .. } = event else {
        return;
    };
    if app.state::<Exit>().stopping.swap(true, Ordering::SeqCst) {
        return;
    }
    api.prevent_exit();
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        app.state::<DesktopShell>().stop().await;
        app.exit(0);
    });
}
