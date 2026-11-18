use std::future::{Future, poll_fn};
use std::task::Poll;
use std::time::Duration;

use prismforge_core::{ConnectionState, RgbBackend};
use prismforge_openrgb::{ClientError, OpenRgbBackend, OpenRgbClient};
use prismforge_testkit::{FakeServer, wire};
use tokio::task::JoinHandle;

use crate::support::{self, RED, solid, strip};

const DRIVER_ROUNDS: usize = 50;

async fn release_acks_until_done<T>(server: &FakeServer, task: JoinHandle<T>) -> T {
    for _ in 0..DRIVER_ROUNDS {
        if task.is_finished() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        server.release_held_acks();
    }
    task.await.expect("exchange task")
}

#[tokio::test(start_paused = true)]
async fn late_count_ack_never_answers_the_next_count_request() {
    let server = FakeServer::new();
    server.add_device(3, strip("alpha", 2));
    let mut client = OpenRgbClient::connect(server.connect(), support::config())
        .await
        .expect("connect");
    server.hold_acks(wire::REQUEST_CONTROLLER_COUNT);

    let exchange = tokio::spawn(async move {
        let first = client.controller_ids().await;
        let second = client.controller_ids().await;
        (first, second)
    });
    let (first, second) = release_acks_until_done(&server, exchange).await;

    assert_eq!(first, Ok(vec![3]));
    assert_eq!(second, Ok(vec![3]));
}

#[tokio::test(start_paused = true)]
async fn late_data_ack_never_makes_a_present_controller_vanish() {
    let server = FakeServer::new();
    server.add_device(5, strip("alpha", 2));
    let mut client = OpenRgbClient::connect(server.connect(), support::config())
        .await
        .expect("connect");
    server.hold_acks(wire::REQUEST_CONTROLLER_DATA);

    let exchange = tokio::spawn(async move {
        let first = client.controller_data(5).await.map(|data| data.serial);
        let second = client.controller_data(5).await.map(|data| data.serial);
        (first, second)
    });
    let (first, second) = release_acks_until_done(&server, exchange).await;

    assert_eq!(first, Ok("SN-alpha".to_owned()));
    assert_eq!(second, Ok("SN-alpha".to_owned()));
}

#[tokio::test(start_paused = true)]
async fn back_to_back_refreshes_of_an_empty_server_survive_late_acks() {
    let server = FakeServer::new();
    server.hold_acks(wire::REQUEST_CONTROLLER_COUNT);
    let mut backend = OpenRgbBackend::new(server.connector(), support::config());

    let exchange = tokio::spawn(async move {
        backend.connect().await.expect("connect");
        let first = backend
            .refresh_inventory()
            .await
            .map(|inventory| inventory.devices.len());
        let second = backend
            .refresh_inventory()
            .await
            .map(|inventory| inventory.devices.len());
        (first, second, backend.connection_state().clone())
    });
    let (first, second, state) = release_acks_until_done(&server, exchange).await;

    assert_eq!(first, Ok(0));
    assert_eq!(second, Ok(0));
    assert_eq!(state, ConnectionState::Connected);
}

#[tokio::test(start_paused = true)]
async fn late_frame_and_client_name_acks_never_answer_later_requests() {
    let server = FakeServer::new();
    server.add_device(5, strip("alpha", 2));
    server.hold_acks(wire::SET_CLIENT_NAME);
    server.hold_acks(wire::UPDATE_LEDS);
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");
    let inventory = backend.refresh_inventory().await.expect("inventory");
    let alpha = support::reference_by_serial(inventory, "SN-alpha");

    backend
        .write_device_frame(&alpha, &solid(RED, 2))
        .await
        .expect("frame");
    backend
        .enable_direct_mode(&alpha)
        .await
        .expect("direct mode");
    let inventory = backend.refresh_inventory().await.expect("refresh");

    assert_eq!(inventory.devices.len(), 1);
    assert_eq!(backend.connection_state(), &ConnectionState::Connected);
}

#[tokio::test]
async fn abandoned_request_closes_the_connection_instead_of_answering_the_next() {
    let server = FakeServer::new();
    server.add_device(3, strip("alpha", 2));
    let mut client = OpenRgbClient::connect(server.connect(), support::config())
        .await
        .expect("connect");

    let mut abandoned = Box::pin(client.controller_ids());
    let first_poll = poll_fn(|context| Poll::Ready(abandoned.as_mut().poll(context))).await;
    assert!(first_poll.is_pending(), "{first_poll:?}");
    drop(abandoned);

    let next = client.controller_ids().await;
    assert!(matches!(next, Err(ClientError::Closed { .. })), "{next:?}");
}
