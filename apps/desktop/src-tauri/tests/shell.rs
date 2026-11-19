mod support;

use std::time::Duration;

use prismforge_core::session::{
    ReconnectPolicy, RestoreOutcome, SessionConfig, SessionError, SessionSnapshot,
};
use prismforge_core::{BackendOrigin, ConnectionState, DeviceRef};
use prismforge_desktop::{CommandError, Shell, ShellConfig};
use prismforge_testkit::{DemoBackend, DemoOutput};
use tokio::time::Instant;

use support::logged::{Log, LoggedBackend};
use support::temp::TempDir;
use support::{Recorder, is_connected, is_retrying, stores};

fn config() -> ShellConfig {
    ShellConfig {
        session: SessionConfig {
            reconnect: ReconnectPolicy {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(800),
            },
            ..SessionConfig::default()
        },
        view_spacing: Duration::from_millis(50),
        identify_duration: Duration::from_secs(30),
    }
}

fn shell() -> (Shell<Recorder>, Recorder, TempDir) {
    let recorder = Recorder::default();
    let (stores, dir) = stores("shell");
    (
        Shell::new(recorder.clone(), config(), stores),
        recorder,
        dir,
    )
}

fn unreachable_demo() -> DemoBackend {
    let backend = DemoBackend::new();
    backend.controls().fail_connect(true);
    backend
}

fn device_count(snapshot: &Option<SessionSnapshot>) -> usize {
    snapshot
        .as_ref()
        .map_or(0, |snapshot| snapshot.devices.len())
}

#[tokio::test(start_paused = true)]
async fn demo_mode_shows_only_simulated_devices() {
    let (shell, recorder, _dir) = shell();

    shell.start(DemoBackend::new()).await.expect("start");
    let view = recorder.wait_for(is_connected).await;

    let session = view.session.expect("session");
    assert_eq!(session.backend.origin, BackendOrigin::Simulated);
    assert_eq!(session.devices.len(), 5);
    assert!(
        session
            .devices
            .iter()
            .all(|device| device.descriptor.origin == BackendOrigin::Simulated)
    );
    assert_eq!(
        shell.view(),
        recorder.views().last().cloned().expect("view")
    );
}

#[tokio::test(start_paused = true)]
async fn switching_mode_stops_the_previous_session_before_the_next_connects() {
    let (shell, recorder, _dir) = shell();
    let log = Log::default();
    shell
        .start(LoggedBackend {
            name: "demo",
            inner: DemoBackend::new(),
            log: log.clone(),
        })
        .await
        .expect("start demo");
    recorder.wait_for(is_connected).await;

    let switched = shell
        .start(LoggedBackend {
            name: "unreachable",
            inner: unreachable_demo(),
            log: log.clone(),
        })
        .await
        .expect("start second");
    recorder.wait_for(is_retrying).await;

    let entries = log.entries();
    let last_demo = entries.iter().rposition(|entry| entry.starts_with("demo"));
    let first_next = entries
        .iter()
        .position(|entry| entry == "unreachable connect");
    assert_eq!(entries[last_demo.expect("demo entries")], "demo disconnect");
    assert!(last_demo < first_next, "{entries:?}");
    let later: Vec<_> = recorder
        .views()
        .into_iter()
        .filter(|view| view.revision >= switched.revision)
        .collect();
    assert!(!later.is_empty());
    assert!(later.iter().all(|view| device_count(&view.session) == 0));
}

#[tokio::test(start_paused = true)]
async fn stopping_restores_a_running_identify_at_once() {
    let (shell, recorder, _dir) = shell();
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start");
    recorder.wait_for(is_connected).await;
    let started = Instant::now();

    let (outcome, stopped) = tokio::join!(
        shell.identify(DeviceRef::new("demo-graphics-card"), None),
        async {
            recorder
                .wait_for(|snapshot| snapshot.identify.is_some())
                .await;
            shell.stop().await
        }
    );

    assert_eq!(outcome.expect("identify").restore, RestoreOutcome::Restored);
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(stopped.session, None);
    assert!(matches!(
        controls.outputs().last(),
        Some(DemoOutput::ModeReapplied { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn commands_without_a_session_are_refused() {
    let (shell, recorder, _dir) = shell();

    let refresh = shell.refresh().await;
    let identify = shell.identify(DeviceRef::new("demo-light-bar"), None).await;
    let stopped = shell.stop().await;

    assert_eq!(refresh, Err(CommandError::NoSession));
    assert_eq!(identify, Err(CommandError::NoSession));
    assert_eq!(stopped.session, None);
    assert_eq!(recorder.views().last(), Some(&stopped));
}

#[tokio::test(start_paused = true)]
async fn start_returns_while_a_slow_backend_is_still_connecting() {
    let (shell, _recorder, _dir) = shell();
    let backend = DemoBackend::new();
    backend.controls().set_call_delay(Duration::from_secs(10));
    let started = Instant::now();

    let view = shell.start(backend).await.expect("start");

    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(
        view.session.expect("session").connection,
        ConnectionState::Connecting
    );
    assert!(shell.view().revision >= view.revision);
}

#[tokio::test(start_paused = true)]
async fn a_full_command_queue_answers_busy_without_waiting() {
    let recorder = Recorder::default();
    let (stores, _dir) = stores("busy");
    let shell = Shell::new(
        recorder,
        ShellConfig {
            session: SessionConfig {
                command_capacity: 1,
                ..config().session
            },
            ..config()
        },
        stores,
    );
    let backend = DemoBackend::new();
    backend.controls().set_call_delay(Duration::from_secs(10));
    shell.start(backend).await.expect("start");
    let started = Instant::now();

    let (first, second) = tokio::join!(shell.refresh(), async {
        let answer = shell.refresh().await;
        (answer, started.elapsed())
    });

    assert_eq!(
        second,
        (
            Err(CommandError::Session(SessionError::Busy)),
            Duration::ZERO
        )
    );
    assert!(first.is_ok(), "{first:?}");
}
