#[cfg(target_os = "macos")]
mod app_menu;
mod apply_lock;
mod automation;
mod automation_commands;
mod commands;
mod effects;
mod exit;
mod forward;
mod group_commands;
mod groups;
mod lifecycle;
mod lifecycle_commands;
pub mod marks;
mod native;
mod on_connect;
mod profile_commands;
mod profiles;
mod publisher;
mod settings;
mod setup;
mod setup_commands;
mod shell;
mod stores;
mod system_entry;
#[cfg(test)]
#[allow(dead_code)]
#[path = "../tests/support/temp.rs"]
mod temp;
mod tray;
mod view;

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};

pub use automation::{
    Automation, AutomationView, OpenRgbCheck, ProfileChoice, QUIT_RULE_LIMIT, RECORD_LIMIT,
    RULE_LOCK_WAIT, RuleProfile, RuleView, SESSION_END_LIMIT, SLEEP_LIMIT, WAKE_APPLY_LIMIT,
    WAKE_STEP_LIMIT,
};
pub use effects::{EffectCatalog, preview_effect};
pub use exit::{
    ExitGuard, LAST_STOP_LIMIT, QUIT_STOP_LIMIT, SESSION_END_WAIT, end_of_session, quit_then_exit,
    quit_with_rule, stop_at_exit, stop_then_exit, stop_within,
};
pub use forward::forward;
pub use groups::{GroupView, GroupsStore, GroupsView};
pub use lifecycle::{
    AUTOSTART_ARG, AppSettingChange, ClosePrompt, CloseStep, DeviceStop, ExitControl, Lifecycle,
    LifecycleSink, LifecycleView, SettingsState, SettingsView, StartEntry, StartEntryState,
    StartProfile, StartupPlan, StopReport, StopResult, TrayModel, TrayState, WindowControl,
    decide_close, launched_by_system, read_entry, startup_plan, switch_entry, tray_model,
};
use prismforge_system_events as system_events;
pub use prismforge_system_events::{
    Dispatcher, Done, EventSink, EventSource, EventSupport, SystemEvent, platform_source,
};
pub use publisher::{Publisher, ViewSink};
pub use settings::SettingsStore;
pub use setup::{
    AcrossPreview, AcrossPreviewRequest, AcrossReport, NotPlaced, PREVIEW_COLOUR_LIMIT,
    PREVIEW_LED_LIMIT, PreviewTarget, SetupLimits, SetupStoreView, preview_across,
};
pub use shell::{Shell, ShellConfig};
pub use stores::{StoreKind, StoreUnavailable, StoreView, Stores};
pub use system_entry::{SystemStartEntry, run_command, startup_disabled};
pub use view::{
    ApplyingProfile, CommandError, ConnectApply, ConnectOutcome, FileRule, MarkProblem, Marked,
    OnConnect, SettingProblem, ShellView,
};

pub const VIEW_EVENT: &str = "session-view";
pub const LIVE_EVENT: &str = "live-output";

type DesktopShell = Shell<WindowSink>;
type DesktopLifecycle = Lifecycle<native::LifecycleWindowSink>;
type DesktopAutomation = Automation;

pub struct WindowSink(AppHandle);

impl ViewSink for WindowSink {
    fn deliver(&self, view: &ShellView) {
        let _ = self.0.emit(VIEW_EVENT, view);
    }

    fn deliver_live(&self, live: &prismforge_core::session::LiveOutput) {
        let _ = self.0.emit(LIVE_EVENT, live);
    }
}

pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let builder = builder
        .menu(app_menu::build)
        .on_menu_event(app_menu::on_event);
    builder
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            tray::act(app, tray::Action::SecondInstance);
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let stores = Stores::new(app.path().app_config_dir()?);
            app.manage(Shell::new(
                WindowSink(handle.clone()),
                ShellConfig::default(),
                stores,
            ));
            app.manage(ExitGuard::default());
            let support = platform_source().start(Arc::new(dispatcher(handle.clone())));
            app.manage(Automation::new(support));
            let lifecycle = Lifecycle::new(
                native::LifecycleWindowSink(handle.clone()),
                SystemStartEntry,
            );
            let tray = tray::create(&handle);
            lifecycle.set_tray(match &tray {
                Ok(_) => TrayState::Available,
                Err(error) => TrayState::Unavailable {
                    detail: error.to_string(),
                },
            });
            app.manage(lifecycle);
            tauri::async_runtime::spawn(start_up(handle.clone()));
            if let Ok(tray) = tray {
                tauri::async_runtime::spawn(tray::follow(handle, tray));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                tray::act(window.app_handle(), tray::Action::CloseRequested);
            }
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
            commands::apply_effect_batch,
            commands::stop_effect_batch,
            commands::stop_all_effects,
            commands::watch_live_output,
            commands::select_mode,
            commands::declare_external,
            commands::clear_external,
            commands::set_frame_rate,
            commands::preview_effect,
            group_commands::groups,
            group_commands::create_group,
            group_commands::rename_group,
            group_commands::add_group_members,
            group_commands::remove_group_member,
            group_commands::delete_group,
            setup_commands::setup,
            setup_commands::save_setup,
            setup_commands::apply_across,
            setup_commands::preview_across,
            profile_commands::profile_store,
            profile_commands::profile_details,
            profile_commands::create_profile,
            profile_commands::preview_profile_update,
            profile_commands::update_profile,
            profile_commands::rename_profile,
            profile_commands::duplicate_profile,
            profile_commands::delete_profile,
            profile_commands::apply_profile,
            profile_commands::export_profile,
            profile_commands::import_profile,
            profile_commands::set_device_name,
            profile_commands::set_zone_name,
            profile_commands::dismiss_store_notice,
            automation_commands::automation,
            automation_commands::set_rule,
            automation_commands::mark_runs_seen,
            lifecycle_commands::lifecycle,
            lifecycle_commands::change_setting,
            lifecycle_commands::answer_close,
            lifecycle_commands::quit,
        ])
        .build(tauri::generate_context!())
        .expect("PrismForge could not start")
        .run(on_run_event);
}

/// The window starts hidden; it is shown here unless the start-up plan keeps it in the tray.
async fn start_up(app: AppHandle) {
    let shell = app.state::<DesktopShell>();
    let lifecycle = app.state::<DesktopLifecycle>();
    let window = native::MainWindow::of(&app);
    let shell = &*shell;
    lifecycle
        .begin(
            shell,
            launched_by_system(std::env::args()),
            &window,
            |address| async move {
                let _ = commands::connect_at_start(shell, &address.host, address.port).await;
            },
        )
        .await;
}

/// Each system event runs on the async runtime; the event thread waits for it only as long as the event allows.
fn dispatcher(app: AppHandle) -> Dispatcher {
    let tauri::async_runtime::RuntimeHandle::Tokio(runtime) = tauri::async_runtime::handle();
    Dispatcher::new(runtime, move |event| {
        let app = app.clone();
        Box::pin(async move {
            let (Some(shell), Some(automation)) = (
                app.try_state::<DesktopShell>(),
                app.try_state::<DesktopAutomation>(),
            ) else {
                return;
            };
            automation.on_event(&shell, event).await;
        })
    })
}

fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        RunEvent::ExitRequested { api, .. } => {
            if !app.state::<ExitGuard>().hold() {
                return;
            }
            api.prevent_exit();
            native::MainWindow::of(app).hide();
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let shell = app.state::<DesktopShell>();
                let automation = app.state::<DesktopAutomation>();
                quit_with_rule(&shell, &automation, &app.state::<ExitGuard>(), || {
                    app.exit(0)
                })
                .await;
            });
        }
        RunEvent::Exit => {
            let shell = app.state::<DesktopShell>();
            let automation = app.state::<DesktopAutomation>();
            let guard = app.state::<ExitGuard>();
            tauri::async_runtime::block_on(end_of_session(&shell, &automation, &guard));
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => tray::act(app, tray::Action::Show),
        _ => {}
    }
}
