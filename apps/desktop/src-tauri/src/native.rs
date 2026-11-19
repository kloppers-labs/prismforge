use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

use crate::lifecycle::{ExitControl, LifecycleSink, LifecycleView, StartEntry, WindowControl};

pub const LIFECYCLE_EVENT: &str = "lifecycle-view";
pub const MAIN_WINDOW: &str = "main";

pub struct MainWindow(Option<WebviewWindow>);

impl MainWindow {
    pub fn of(app: &AppHandle) -> Self {
        Self(app.get_webview_window(MAIN_WINDOW))
    }
}

impl WindowControl for MainWindow {
    fn show(&self) {
        if let Some(window) = &self.0 {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }

    fn hide(&self) {
        if let Some(window) = &self.0 {
            let _ = window.hide();
        }
    }
}

pub struct AppExit(pub AppHandle);

impl ExitControl for AppExit {
    fn exit(&self) {
        self.0.exit(0);
    }
}

/// The per-user entry of `tauri-plugin-autostart`: the HKCU Run key on Windows, a LaunchAgent on macOS.
pub struct SystemStartEntry(pub AppHandle);

impl StartEntry for SystemStartEntry {
    fn is_registered(&self) -> Result<bool, String> {
        self.0
            .autolaunch()
            .is_enabled()
            .map_err(|error| error.to_string())
    }

    fn register(&self) -> Result<(), String> {
        self.0
            .autolaunch()
            .enable()
            .map_err(|error| error.to_string())
    }

    fn unregister(&self) -> Result<(), String> {
        self.0
            .autolaunch()
            .disable()
            .map_err(|error| error.to_string())
    }
}

pub struct LifecycleWindowSink(pub AppHandle);

impl LifecycleSink for LifecycleWindowSink {
    fn deliver(&self, view: &LifecycleView) {
        let _ = self.0.emit(LIFECYCLE_EVENT, view);
    }
}
