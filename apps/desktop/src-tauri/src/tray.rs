use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::lifecycle::TrayModel;
use crate::native::{AppExit, MainWindow};
use crate::{DesktopLifecycle, DesktopShell};

const TOGGLE: &str = "toggle";
const STOP_ALL: &str = "stop-all";
const QUIT: &str = "quit";
const SPACING: Duration = Duration::from_millis(250);

pub struct Tray {
    icon: TrayIcon,
    toggle: MenuItem<Wry>,
    connection: MenuItem<Wry>,
    effects: MenuItem<Wry>,
    profile: MenuItem<Wry>,
    stop_all: MenuItem<Wry>,
}

/// Fails where the system has no tray; the caller then makes closing the window quit.
pub fn create(app: &AppHandle) -> tauri::Result<Tray> {
    let toggle = MenuItem::with_id(app, TOGGLE, "Show PrismForge window", true, None::<&str>)?;
    let connection = MenuItem::new(app, "Not connected", false, None::<&str>)?;
    let effects = MenuItem::new(app, "No PrismForge effects running", false, None::<&str>)?;
    let profile = MenuItem::new(app, "No active profile", false, None::<&str>)?;
    let stop_all = MenuItem::with_id(
        app,
        STOP_ALL,
        "Stop all PrismForge effects",
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, QUIT, "Quit PrismForge", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &toggle,
            &PredefinedMenuItem::separator(app)?,
            &connection,
            &effects,
            &profile,
            &PredefinedMenuItem::separator(app)?,
            &stop_all,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let builder = TrayIconBuilder::with_id("main")
        .tooltip("PrismForge")
        .menu(&menu)
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(|app, event| match event.id().as_ref() {
            TOGGLE => act(app, Action::Toggle),
            STOP_ALL => act(app, Action::StopAll),
            QUIT => act(app, Action::Quit),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
                && !cfg!(target_os = "macos")
            {
                act(tray.app_handle(), Action::Show);
            }
        });
    let builder = if cfg!(target_os = "macos") {
        builder
            .icon(tauri::include_image!("icons/tray-template.png"))
            .icon_as_template(true)
    } else {
        match app.default_window_icon() {
            Some(icon) => builder.icon(icon.clone()),
            None => builder,
        }
    };
    Ok(Tray {
        icon: builder.build(app)?,
        toggle,
        connection,
        effects,
        profile,
        stop_all,
    })
}

#[derive(Clone, Copy)]
pub enum Action {
    Show,
    Toggle,
    StopAll,
    Quit,
}

/// Runs a tray or system action through the same lifecycle and shell methods as the commands.
pub fn act(app: &AppHandle, action: Action) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let shell = app.state::<DesktopShell>();
        let lifecycle = app.state::<DesktopLifecycle>();
        let window = MainWindow::of(&app);
        match action {
            Action::Show => lifecycle.show(&shell, &window).await,
            Action::Toggle => lifecycle.toggle(&shell, &window).await,
            Action::StopAll => {
                lifecycle.stop_all(&shell).await;
            }
            Action::Quit => lifecycle.quit(&window, &AppExit(app.clone())),
        }
    });
}

/// Follows the view and the lifecycle state, at most once per `SPACING`; it never polls.
pub async fn follow(app: AppHandle, tray: Tray) {
    let shell = app.state::<DesktopShell>();
    let lifecycle = app.state::<DesktopLifecycle>();
    let mut views = shell.changes();
    let mut states = lifecycle.changes();
    let mut shown = None;
    loop {
        let model = lifecycle.tray_model(&shell).await;
        if shown.as_ref() != Some(&model) {
            show(&tray, &model);
            shown = Some(model);
        }
        tokio::time::sleep(SPACING).await;
        tokio::select! {
            changed = views.changed() => if changed.is_err() { return },
            changed = states.changed() => if changed.is_err() { return },
        }
    }
}

fn show(tray: &Tray, model: &TrayModel) {
    let _ = tray.toggle.set_text(&model.toggle);
    let _ = tray.connection.set_text(&model.connection);
    let _ = tray.effects.set_text(&model.effects);
    let _ = tray.profile.set_text(&model.profile);
    let _ = tray.stop_all.set_enabled(model.stop_enabled);
    let _ = tray.icon.set_tooltip(Some(&model.tooltip));
}
