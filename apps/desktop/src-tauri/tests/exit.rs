mod support;

use std::cell::RefCell;
use std::time::Duration;

use prismforge_core::DeviceRef;
use prismforge_core::session::{ReconnectPolicy, RestoreOutcome, SessionConfig};
use prismforge_desktop::{ExitGuard, Shell, ShellConfig, ShellView, stop_then_exit};
use prismforge_testkit::{DemoBackend, DemoOutput};
use tokio::time::Instant;

use support::logged::{Log, LoggedBackend};
use support::{Recorder, is_connected};

struct AtExit {
    log: Vec<String>,
    last_output: Option<DemoOutput>,
    view: ShellView,
}

fn shell() -> (Shell<Recorder>, Recorder) {
    let recorder = Recorder::default();
    let config = ShellConfig {
        session: SessionConfig {
            reconnect: ReconnectPolicy {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(800),
            },
            ..SessionConfig::default()
        },
        view_spacing: Duration::from_millis(50),
        identify_duration: Duration::from_secs(30),
    };
    (Shell::new(recorder.clone(), config), recorder)
}

#[test]
fn only_the_first_exit_request_is_held() {
    let guard = ExitGuard::default();

    assert_eq!(
        [guard.hold(), guard.hold(), guard.hold()],
        [true, false, false]
    );
}

#[tokio::test(start_paused = true)]
async fn exit_waits_until_a_running_identify_is_restored_and_the_backend_is_disconnected() {
    let (shell, recorder) = shell();
    let log = Log::default();
    let demo = DemoBackend::new();
    let controls = demo.controls();
    shell
        .start(LoggedBackend {
            name: "demo",
            inner: demo,
            log: log.clone(),
        })
        .await
        .expect("start");
    recorder.wait_for(is_connected).await;
    let started = Instant::now();
    let at_exit = RefCell::new(None);

    let (outcome, ()) = tokio::join!(
        shell.identify(DeviceRef::new("demo-graphics-card"), None),
        async {
            recorder
                .wait_for(|snapshot| snapshot.identify.is_some())
                .await;
            stop_then_exit(&shell, || {
                *at_exit.borrow_mut() = Some(AtExit {
                    log: log.entries(),
                    last_output: controls.outputs().last().cloned(),
                    view: shell.view(),
                });
            })
            .await;
        }
    );

    let at_exit = at_exit.into_inner().expect("exit was called");
    assert_eq!(outcome.expect("identify").restore, RestoreOutcome::Restored);
    assert!(matches!(
        at_exit.last_output,
        Some(DemoOutput::ModeReapplied { .. })
    ));
    assert_eq!(
        at_exit.log.last().map(String::as_str),
        Some("demo disconnect")
    );
    assert_eq!(at_exit.view.session, None);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[tokio::test(start_paused = true)]
async fn exit_does_not_wait_when_the_session_is_already_gone() {
    let (shell, recorder) = shell();
    shell.start(DemoBackend::new()).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell.stop().await;
    let mut exits = 0;

    tokio::time::timeout(
        Duration::from_secs(1),
        stop_then_exit(&shell, || exits += 1),
    )
    .await
    .expect("exit did not hang");
    let (fresh, _) = self::shell();
    tokio::time::timeout(
        Duration::from_secs(1),
        stop_then_exit(&fresh, || exits += 1),
    )
    .await
    .expect("exit without any session did not hang");

    assert_eq!(exits, 2);
}
