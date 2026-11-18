use std::future::ready;
use std::io;
use std::time::Duration;

use prismforge_core::{
    BackendError, BackendEvent, ConnectionState, LossCause, RgbBackend, StateReason,
};
use prismforge_openrgb::OpenRgbBackend;
use prismforge_testkit::{FakeServer, wire};
use tokio::io::DuplexStream;

use crate::support::{self, strip};

fn lost(cause: LossCause) -> ConnectionState {
    ConnectionState::Disconnected {
        reason: Some(StateReason::ConnectionLost { cause }),
    }
}

#[tokio::test]
async fn server_close_is_reported_as_server_closed() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    backend.refresh_inventory().await.expect("inventory");

    server.disconnect_all();
    let event = backend.next_event().await;

    assert!(
        matches!(
            event,
            Some(BackendEvent::ConnectionLost {
                cause: LossCause::ServerClosed,
                ..
            })
        ),
        "{event:?}"
    );
    assert_eq!(backend.connection_state(), &lost(LossCause::ServerClosed));
}

#[tokio::test(start_paused = true)]
async fn server_close_during_request_is_reported_as_server_closed() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.silence(wire::REQUEST_CONTROLLER_COUNT);
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let closer = server.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        closer.disconnect_all();
    });

    let error = backend.refresh_inventory().await.unwrap_err();

    assert!(
        matches!(
            error,
            BackendError::ConnectionLost {
                cause: LossCause::ServerClosed,
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(backend.connection_state(), &lost(LossCause::ServerClosed));
}

#[tokio::test(start_paused = true)]
async fn request_timeout_is_reported_as_timeout() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.silence(wire::REQUEST_CONTROLLER_DATA);
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    let error = backend.refresh_inventory().await.unwrap_err();

    assert!(matches!(error, BackendError::Timeout { .. }), "{error:?}");
    assert_eq!(backend.connection_state(), &lost(LossCause::Timeout));
}

#[tokio::test]
async fn refused_connect_is_reported_as_unreachable() {
    let refused = || {
        ready(Err::<DuplexStream, _>(io::Error::from(
            io::ErrorKind::ConnectionRefused,
        )))
    };
    let mut backend = OpenRgbBackend::new(refused, support::config());

    let error = backend.connect().await.unwrap_err();

    assert!(
        matches!(error, BackendError::Unreachable { .. }),
        "{error:?}"
    );
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Disconnected {
            reason: Some(StateReason::ServerUnreachable)
        }
    );
}
