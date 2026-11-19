mod close;
mod start_entry;
mod startup;
mod stop_all;
mod tray_model;

use std::sync::{Mutex, MutexGuard, PoisonError};

use prismforge_core::store::{CloseAction, OpenRgbAddress, StoreNotice};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::stores::StoreUnavailable;
use crate::view::{CommandError, MarkProblem, SettingProblem};

pub use close::{ClosePrompt, CloseStep, decide_close};
pub use start_entry::{StartEntry, StartEntryState, read_entry, switch_entry};
pub use startup::{AUTOSTART_ARG, StartupPlan, launched_by_system, startup_plan};
pub use stop_all::{DeviceStop, StopReport, StopResult};
pub use tray_model::{TrayModel, tray_model};

pub trait LifecycleSink: Send + Sync + 'static {
    fn deliver(&self, view: &LifecycleView);
}

/// The main window; `show` also restores a minimised window and gives it focus.
pub trait WindowControl: Sync {
    fn show(&self);
    fn hide(&self);
}

/// Asks the app to exit; the exit handler stops the shell before the process ends.
pub trait ExitControl: Sync {
    fn exit(&self);
}

/// Everything the Settings screen and the first-close question show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct LifecycleView {
    pub revision: u64,
    pub settings: SettingsState,
    pub tray: TrayState,
    pub start_with_system: StartEntryState,
    /// An entry for every user of this computer, left by an earlier run as Administrator.
    pub machine_start_entry: bool,
    pub window_visible: bool,
    /// Quit has started; further quit requests from the user are ignored until the process ends.
    pub quitting: bool,
    /// The window asked to close and waits for the user's first choice.
    pub close_prompt: bool,
    /// The result of the last "Stop all PrismForge effects" from the tray.
    pub tray_stop: Option<StopReport>,
    /// The profile that connect at start applies, from the real store's mark.
    pub start_profile: StartProfile,
    /// The per-user folder that holds the settings, both stores and the folder lock.
    pub folder: String,
}

/// The single source is the per-profile mark "Apply this profile when PrismForge connects".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StartProfile {
    NotMarked,
    Marked { name: String },
    Unreadable { problem: MarkProblem },
    Unavailable { reason: StoreUnavailable },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SettingsState {
    Ready {
        settings: SettingsView,
        notices: Vec<StoreNotice>,
    },
    Unavailable {
        reason: StoreUnavailable,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SettingsView {
    /// `None` until the user chooses; the first close then asks.
    pub on_close: Option<CloseAction>,
    pub start_hidden: bool,
    pub connect_at_start: bool,
    /// The OpenRGB address of the user's last Connect; connect at start uses it.
    pub address: Option<OpenRgbAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum TrayState {
    Available,
    Unavailable { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum AppSettingChange {
    OnClose(CloseAction),
    StartWithSystem(bool),
    StartHidden(bool),
    ConnectAtStart(bool),
}

struct State {
    revision: u64,
    tray: TrayState,
    window_visible: bool,
    prompt: ClosePrompt,
    tray_stop: Option<StopReport>,
    quitting: bool,
}

/// Window, tray and start-up behaviour; tray actions, window events and commands all go through it.
pub struct Lifecycle<L> {
    sink: L,
    entry: Box<dyn StartEntry>,
    state: Mutex<State>,
    // Lock order: taken before the store; serialises publishes so a view never overtakes a newer one.
    publishing: tokio::sync::Mutex<()>,
    changes: watch::Sender<u64>,
}

impl<L: LifecycleSink> Lifecycle<L> {
    pub fn new(sink: L, entry: impl StartEntry) -> Self {
        Self {
            sink,
            entry: Box::new(entry),
            state: Mutex::new(State {
                revision: 0,
                tray: TrayState::Unavailable {
                    detail: "the tray has not been set up".into(),
                },
                window_visible: false,
                prompt: ClosePrompt::default(),
                tray_stop: None,
                quitting: false,
            }),
            publishing: tokio::sync::Mutex::new(()),
            changes: watch::channel(0).0,
        }
    }

    pub fn set_tray(&self, tray: TrayState) {
        self.lock().tray = tray;
    }

    pub fn tray_available(&self) -> bool {
        self.lock().tray == TrayState::Available
    }

    pub fn window_visible(&self) -> bool {
        self.lock().window_visible
    }

    pub fn changes(&self) -> watch::Receiver<u64> {
        self.changes.subscribe()
    }

    pub async fn view<S: ViewSink>(&self, shell: &Shell<S>) -> LifecycleView {
        let settings = match shell.settings().await {
            Ok(store) => {
                let (settings, notices) = store.read().await;
                SettingsState::Ready {
                    settings: SettingsView {
                        on_close: settings.on_close,
                        start_hidden: settings.start_hidden,
                        connect_at_start: settings.connect_at_start,
                        address: settings.last_address,
                    },
                    notices,
                }
            }
            Err(reason) => SettingsState::Unavailable { reason },
        };
        let start_profile = shell.start_profile().await;
        let start_with_system = read_entry(self.entry.as_ref());
        let machine_start_entry = self.entry.machine_wide();
        let state = self.lock();
        LifecycleView {
            revision: state.revision,
            settings,
            tray: state.tray.clone(),
            start_with_system,
            machine_start_entry,
            window_visible: state.window_visible,
            quitting: state.quitting,
            close_prompt: state.prompt.asking(),
            tray_stop: state.tray_stop.clone(),
            start_profile,
            folder: shell.folder(),
        }
    }

    pub async fn publish<S: ViewSink>(&self, shell: &Shell<S>) -> LifecycleView {
        let _publishing = self.publishing.lock().await;
        let mut view = self.view(shell).await;
        let mut state = self.lock();
        state.revision += 1;
        view.revision = state.revision;
        view.window_visible = state.window_visible;
        view.quitting = state.quitting;
        view.close_prompt = state.prompt.asking();
        view.tray_stop = state.tray_stop.clone();
        self.sink.deliver(&view);
        self.changes.send_modify(|count| *count += 1);
        view
    }

    /// Publishes the state that holds afterwards, also when the change was refused.
    pub async fn change<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        change: AppSettingChange,
    ) -> Result<LifecycleView, CommandError> {
        let changed = self.apply(shell, change).await;
        let view = self.publish(shell).await;
        changed.map(|()| view)
    }

    async fn apply<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        change: AppSettingChange,
    ) -> Result<(), CommandError> {
        let tray = self.tray_available();
        match change {
            AppSettingChange::StartWithSystem(on) => {
                switch_entry(self.entry.as_ref(), on)?;
                return Ok(());
            }
            AppSettingChange::OnClose(CloseAction::Background)
            | AppSettingChange::StartHidden(true)
                if !tray =>
            {
                return Err(SettingProblem::TrayUnavailable.into());
            }
            _ => {}
        }
        let store = shell.settings().await?;
        if change == AppSettingChange::ConnectAtStart(true)
            && store.read().await.0.last_address.is_none()
        {
            return Err(SettingProblem::NoAddress.into());
        }
        Ok(store
            .change(move |settings| match change {
                AppSettingChange::OnClose(action) => settings.on_close = Some(action),
                AppSettingChange::StartHidden(on) => settings.start_hidden = on,
                AppSettingChange::ConnectAtStart(on) => settings.connect_at_start = on,
                AppSettingChange::StartWithSystem(_) => {}
            })
            .await?)
    }

    /// Records the address of a Connect by the user; connect at start uses only this address.
    pub async fn remember_address<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        host: &str,
        port: u16,
    ) -> Result<(), CommandError> {
        let address = OpenRgbAddress {
            host: host.trim().to_owned(),
            port,
        };
        let store = shell.settings().await?;
        store
            .change(move |settings| settings.last_address = Some(address))
            .await?;
        self.publish(shell).await;
        Ok(())
    }

    /// Shows the window unless the plan keeps it in the tray, then connects through `connect` if chosen.
    pub async fn begin<S, F, Fut>(
        &self,
        shell: &Shell<S>,
        by_system: bool,
        window: &dyn WindowControl,
        connect: F,
    ) -> StartupPlan
    where
        S: ViewSink,
        F: FnOnce(OpenRgbAddress) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let plan = self.startup(shell, by_system).await;
        if plan.show_window {
            self.show(shell, window).await;
        } else {
            self.publish(shell).await;
        }
        if let Some(address) = plan.connect.clone() {
            connect(address).await;
        }
        plan
    }

    pub async fn startup<S: ViewSink>(&self, shell: &Shell<S>, by_system: bool) -> StartupPlan {
        let settings = match shell.settings().await {
            Ok(store) => Some(store.read().await.0),
            Err(_) => None,
        };
        startup_plan(settings.as_ref(), by_system, self.tray_available())
    }

    pub async fn close_requested<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        window: &dyn WindowControl,
        exit: &dyn ExitControl,
    ) {
        let on_close = match shell.settings().await {
            Ok(store) => store.read().await.0.on_close,
            Err(_) => None,
        };
        let step = decide_close(on_close, self.tray_available());
        let step = self.lock().prompt.requested(step);
        self.perform(shell, step, window, exit).await;
    }

    /// Performs the user's answer even when it cannot be stored; the store error is returned after.
    pub async fn answer_close<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        choice: Option<CloseAction>,
        window: &dyn WindowControl,
        exit: &dyn ExitControl,
    ) -> Result<(), CommandError> {
        let Some(step) = self.lock().prompt.answered(choice) else {
            self.publish(shell).await;
            return Ok(());
        };
        let stored = match choice {
            Some(action) => self.apply(shell, AppSettingChange::OnClose(action)).await,
            None => Ok(()),
        };
        self.perform(shell, step, window, exit).await;
        stored
    }

    async fn perform<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        step: CloseStep,
        window: &dyn WindowControl,
        exit: &dyn ExitControl,
    ) {
        match step {
            CloseStep::Hide => self.hide(shell, window).await,
            CloseStep::Quit => {
                self.quit(shell, window, exit).await;
            }
            CloseStep::Ask => {
                self.publish(shell).await;
            }
        }
    }

    pub async fn show<S: ViewSink>(&self, shell: &Shell<S>, window: &dyn WindowControl) {
        window.show();
        self.lock().window_visible = true;
        self.publish(shell).await;
    }

    pub async fn hide<S: ViewSink>(&self, shell: &Shell<S>, window: &dyn WindowControl) {
        window.hide();
        self.lock().window_visible = false;
        self.publish(shell).await;
    }

    pub async fn toggle<S: ViewSink>(&self, shell: &Shell<S>, window: &dyn WindowControl) {
        if self.window_visible() {
            self.hide(shell, window).await;
        } else {
            self.show(shell, window).await;
        }
    }

    pub async fn second_instance<S: ViewSink>(&self, shell: &Shell<S>, window: &dyn WindowControl) {
        self.show(shell, window).await;
    }

    /// Starts the quit once: hides the window and asks for the exit; false when a quit already runs.
    pub async fn quit<S: ViewSink>(
        &self,
        shell: &Shell<S>,
        window: &dyn WindowControl,
        exit: &dyn ExitControl,
    ) -> bool {
        {
            let mut state = self.lock();
            if state.quitting {
                return false;
            }
            state.quitting = true;
            state.window_visible = false;
        }
        window.hide();
        exit.exit();
        self.publish(shell).await;
        true
    }

    /// Stops every PrismForge effect device by device through the same command as the window.
    pub async fn stop_all<S: ViewSink>(&self, shell: &Shell<S>) -> StopReport {
        let stops = stop_all::stop_each(shell).await;
        let report = {
            let mut state = self.lock();
            let sequence = state.tray_stop.as_ref().map_or(1, |last| last.sequence + 1);
            let report = StopReport { sequence, stops };
            state.tray_stop = Some(report.clone());
            report
        };
        self.publish(shell).await;
        report
    }

    pub async fn tray_model<S: ViewSink>(&self, shell: &Shell<S>) -> TrayModel {
        let active = shell.active_profile_name().await;
        let (visible, quitting) = {
            let state = self.lock();
            (state.window_visible, state.quitting)
        };
        tray_model(&shell.view(), active.as_deref(), visible, quitting)
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
