use std::future::ready;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use prismforge_core::session::{Operation, SessionError};
use prismforge_core::{BackendError, BackendOrigin, ConnectionState, LossCause, StateReason};
use prismforge_openrgb::OpenRgbBackend;
use prismforge_testkit::{FakeServer, wire};
use tokio::io::DuplexStream;
use tokio::time::Instant;

use crate::support::{
    self, FIRST_RETRY, MAX_RETRY, client_config, config, is_connected, is_retrying, start, strip,
    wait_until,
};

fn refused() -> impl Fn() -> std::future::Ready<io::Result<DuplexStream>> + Send + Sync {
    || ready(Err(io::Error::from(io::ErrorKind::ConnectionRefused)))
}

#[tokio::test(start_paused = true)]
async fn connect_refreshes_the_inventory_automatically() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 2));

    let handle = support::connected_openrgb(&server).await;

    let snapshot = handle.snapshot();
    assert_eq!(snapshot.connection, ConnectionState::Connected);
    assert_eq!(snapshot.devices.len(), 2);
    assert!(snapshot.devices.iter().all(|device| device.available));
    assert!(
        snapshot
            .devices
            .iter()
            .all(|device| device.descriptor.origin == BackendOrigin::OpenRgb)
    );
    assert_eq!(snapshot.diagnostics.counters.connect_attempts, 1);
    assert_eq!(snapshot.diagnostics.counters.inventory_refreshes, 1);
}

#[tokio::test(start_paused = true)]
async fn session_that_never_connected_has_no_devices() {
    let handle = start(OpenRgbBackend::new(refused(), client_config()), config());
    let initial = handle.snapshot();
    assert_eq!(
        initial.connection,
        ConnectionState::Disconnected { reason: None }
    );
    assert!(initial.devices.is_empty());

    let error = handle.connect().await.unwrap_err();

    assert!(
        matches!(
            error,
            SessionError::Backend(BackendError::Unreachable { .. })
        ),
        "{error:?}"
    );
    let snapshot = handle.snapshot();
    assert_eq!(
        snapshot.connection,
        ConnectionState::Retrying {
            attempt: 1,
            reason: StateReason::ServerUnreachable
        }
    );
    assert!(snapshot.devices.is_empty());
    assert_eq!(snapshot.backend.origin, BackendOrigin::OpenRgb);
    let last_error = snapshot.diagnostics.last_error.expect("last error");
    assert_eq!(last_error.operation, Operation::Connect);
    assert!(matches!(last_error.error, BackendError::Unreachable { .. }));
}

#[tokio::test(start_paused = true)]
async fn lost_connection_keeps_last_devices_marked_unavailable() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 2));
    let handle = support::connected_openrgb(&server).await;
    let before = handle.snapshot().devices;

    server.disconnect_all();
    let retrying = wait_until(&handle, is_retrying).await;

    assert_eq!(
        retrying.connection,
        ConnectionState::Retrying {
            attempt: 1,
            reason: StateReason::ConnectionLost {
                cause: LossCause::ServerClosed
            }
        }
    );
    assert_eq!(retrying.devices.len(), 2);
    for (kept, original) in retrying.devices.iter().zip(&before) {
        assert!(!kept.available);
        assert_eq!(kept.identity, original.identity);
        assert_eq!(kept.descriptor, original.descriptor);
    }
    let reconnected = wait_until(&handle, is_connected).await;
    assert!(reconnected.devices.iter().all(|device| device.available));
}

#[tokio::test(start_paused = true)]
async fn request_timeout_cause_reaches_the_snapshot() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let handle = support::connected_openrgb(&server).await;
    server.silence(wire::REQUEST_CONTROLLER_DATA);

    let error = handle.refresh().await.unwrap_err();

    assert!(
        matches!(error, SessionError::Backend(BackendError::Timeout { .. })),
        "{error:?}"
    );
    let lost = StateReason::ConnectionLost {
        cause: LossCause::Timeout,
    };
    let snapshot = handle.snapshot();
    assert_eq!(
        snapshot.connection,
        ConnectionState::Retrying {
            attempt: 1,
            reason: lost.clone()
        }
    );
    let last_error = snapshot.diagnostics.last_error.expect("last error");
    assert_eq!(last_error.operation, Operation::Refresh);
    assert_eq!(last_error.ended_connection, Some(lost));
}

#[tokio::test(start_paused = true)]
async fn unsupported_server_is_not_retried() {
    let server = FakeServer::new();
    server.set_protocol_version(Some(5));
    let handle = start(support::openrgb(&server), config());

    let error = handle.connect().await.unwrap_err();
    tokio::time::sleep(Duration::from_secs(3600)).await;

    assert!(
        matches!(
            error,
            SessionError::Backend(BackendError::UnsupportedProtocol { .. })
        ),
        "{error:?}"
    );
    let snapshot = handle.snapshot();
    assert_eq!(
        snapshot.connection,
        ConnectionState::Disconnected {
            reason: Some(StateReason::UnsupportedProtocol {
                server_version: Some(5)
            })
        }
    );
    assert_eq!(snapshot.diagnostics.counters.connect_attempts, 1);
}

#[tokio::test(start_paused = true)]
async fn reconnect_backs_off_to_a_cap_one_attempt_at_a_time_until_disconnect() {
    const CONNECT_TIME: Duration = Duration::from_millis(10);
    let starts = Arc::new(Mutex::new(Vec::new()));
    let in_flight = Arc::new(AtomicUsize::new(0));
    let most_in_flight = Arc::new(AtomicUsize::new(0));
    let connector = {
        let (starts, in_flight, most_in_flight) =
            (starts.clone(), in_flight.clone(), most_in_flight.clone());
        move || {
            let (starts, in_flight, most_in_flight) =
                (starts.clone(), in_flight.clone(), most_in_flight.clone());
            async move {
                starts.lock().unwrap().push(Instant::now());
                let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                most_in_flight.fetch_max(now, Ordering::SeqCst);
                tokio::time::sleep(CONNECT_TIME).await;
                in_flight.fetch_sub(1, Ordering::SeqCst);
                Err::<DuplexStream, _>(io::Error::from(io::ErrorKind::ConnectionRefused))
            }
        }
    };
    let handle = start(OpenRgbBackend::new(connector, client_config()), config());

    let _ = handle.connect().await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    let retrying = handle.snapshot();
    handle.disconnect().await.expect("disconnect");
    let attempts_at_disconnect = starts.lock().unwrap().len();
    tokio::time::sleep(Duration::from_secs(3600)).await;

    let starts = starts.lock().unwrap().clone();
    let delays: Vec<Duration> = starts
        .windows(2)
        .map(|pair| pair[1] - pair[0] - CONNECT_TIME)
        .collect();
    assert!(delays.len() >= 6, "{delays:?}");
    assert_eq!(
        delays[..4],
        [FIRST_RETRY, FIRST_RETRY * 2, FIRST_RETRY * 4, MAX_RETRY]
    );
    assert!(
        delays[4..].iter().all(|delay| *delay == MAX_RETRY),
        "{delays:?}"
    );
    assert_eq!(most_in_flight.load(Ordering::SeqCst), 1);
    assert!(matches!(
        retrying.connection,
        ConnectionState::Retrying {
            reason: StateReason::ServerUnreachable,
            ..
        }
    ));
    assert_eq!(starts.len(), attempts_at_disconnect);
    let snapshot = handle.snapshot();
    assert_eq!(
        snapshot.connection,
        ConnectionState::Disconnected { reason: None }
    );
    assert_eq!(
        snapshot.diagnostics.counters.connect_attempts,
        starts.len() as u64
    );
}
