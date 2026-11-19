use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::lifecycle::{ExitControl, LifecycleSink, LifecycleView, WindowControl};

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

pub struct LifecycleWindowSink(pub AppHandle);

impl LifecycleSink for LifecycleWindowSink {
    fn deliver(&self, view: &LifecycleView) {
        let _ = self.0.emit(LIFECYCLE_EVENT, view);
    }
}
