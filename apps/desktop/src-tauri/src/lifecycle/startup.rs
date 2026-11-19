use prismforge_core::store::{AppSettings, OpenRgbAddress};

/// The argument the start entry passes, so a start by the system can be told from a start by the user.
pub const AUTOSTART_ARG: &str = "--autostart";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPlan {
    pub show_window: bool,
    pub connect: Option<OpenRgbAddress>,
}

/// Unreadable settings count as all off: the window shows and nothing connects.
pub fn startup_plan(settings: Option<&AppSettings>, by_system: bool, tray: bool) -> StartupPlan {
    let Some(settings) = settings else {
        return StartupPlan {
            show_window: true,
            connect: None,
        };
    };
    StartupPlan {
        show_window: !(by_system && tray && settings.start_hidden),
        connect: settings
            .connect_at_start
            .then(|| settings.last_address.clone())
            .flatten(),
    }
}

pub fn launched_by_system(args: impl IntoIterator<Item = String>) -> bool {
    args.into_iter().any(|arg| arg == AUTOSTART_ARG)
}
