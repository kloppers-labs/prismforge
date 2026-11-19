use prismforge_core::session::{OutputCondition, SessionSnapshot};
use prismforge_core::{BackendOrigin, ConnectionState, OutputOwner};

use crate::view::ShellView;

/// The tray's texts; every line comes from the published view or the store, never from a guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayModel {
    pub toggle: String,
    pub connection: String,
    pub effects: String,
    pub profile: String,
    pub tooltip: String,
    pub stop_enabled: bool,
    pub quit: String,
    pub quit_enabled: bool,
}

pub fn tray_model(
    view: &ShellView,
    active: Option<&str>,
    window_visible: bool,
    quitting: bool,
) -> TrayModel {
    let connection = view
        .session
        .as_ref()
        .map_or_else(|| "Not connected".to_owned(), connection_line);
    TrayModel {
        toggle: if window_visible {
            "Hide PrismForge window"
        } else {
            "Show PrismForge window"
        }
        .into(),
        tooltip: format!("PrismForge: {connection}"),
        connection,
        effects: view
            .session
            .as_ref()
            .map_or_else(|| effects_line(0, 0), effects_of),
        profile: match (&view.applying, active) {
            (Some(applying), _) => format!("Applying profile: {}", applying.name),
            (None, Some(name)) => format!("Active profile: {name}"),
            (None, None) => "No active profile".into(),
        },
        stop_enabled: !quitting
            && view
                .session
                .as_ref()
                .is_some_and(|session| !session.output.scene.assignments.is_empty()),
        quit: if quitting {
            "Quitting PrismForge"
        } else {
            "Quit PrismForge"
        }
        .into(),
        quit_enabled: !quitting,
    }
}

fn connection_line(session: &SessionSnapshot) -> String {
    if session.backend.origin == BackendOrigin::Simulated {
        return match &session.connection {
            ConnectionState::Connected => "Demo mode: simulated devices, no hardware",
            ConnectionState::Connecting => "Demo mode: starting",
            ConnectionState::Retrying { .. } => "Demo mode: retrying",
            ConnectionState::Degraded { .. } => "Demo mode: running with problems",
            ConnectionState::Disconnected { .. } => "Demo mode stopped",
        }
        .into();
    }
    let server = session.backend.address.as_ref().map_or_else(
        || "OpenRGB".to_owned(),
        |address| format!("OpenRGB at {address}"),
    );
    match &session.connection {
        ConnectionState::Connected => format!("Connected to {server}"),
        ConnectionState::Connecting => format!("Connecting to {server}"),
        ConnectionState::Retrying { .. } => format!("{server} is not reachable, retrying"),
        ConnectionState::Degraded { .. } => format!("Connected to {server}, with problems"),
        ConnectionState::Disconnected { .. } => format!("Connection to {server} stopped"),
    }
}

fn effects_of(session: &SessionSnapshot) -> String {
    let effects = session
        .output
        .devices
        .iter()
        .filter(|device| device.owner == OutputOwner::Effect);
    let (running, waiting) = effects.fold((0, 0), |(running, waiting), device| {
        if device.condition == OutputCondition::Running {
            (running + 1, waiting)
        } else {
            (running, waiting + 1)
        }
    });
    effects_line(running, waiting)
}

fn effects_line(running: usize, waiting: usize) -> String {
    match (running, waiting) {
        (0, 0) => "No PrismForge effects running".into(),
        (running, 0) => format!("PrismForge effects running on {}", devices(running)),
        (0, waiting) => format!("PrismForge effects waiting on {}", devices(waiting)),
        (running, waiting) => format!(
            "PrismForge effects running on {}, waiting on {waiting}",
            devices(running)
        ),
    }
}

fn devices(count: usize) -> String {
    if count == 1 {
        "1 device".into()
    } else {
        format!("{count} devices")
    }
}
