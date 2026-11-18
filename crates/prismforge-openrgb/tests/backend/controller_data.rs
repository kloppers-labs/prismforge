use std::collections::BTreeMap;

use prismforge_core::{
    BackendError, ConnectionState, IssueReason, LossCause, RgbBackend, StateReason,
};
use prismforge_openrgb::protocol::{FrameError, MAX_PACKET_SIZE};
use prismforge_openrgb::{ClientError, OpenRgbClient};
use prismforge_testkit::{ControllerBlock, FakeServer, wire};

use crate::support::{self, RED, solid, strip};

const NAME_LEN_OFFSET: usize = 8;

pub fn with_declared_size(mut bytes: Vec<u8>) -> Vec<u8> {
    let len = bytes.len() as u32;
    bytes[..4].copy_from_slice(&len.to_le_bytes());
    bytes
}

pub fn truncated_block() -> Vec<u8> {
    let mut bytes = strip("truncated", 4).encode();
    bytes.truncate(bytes.len() / 2);
    with_declared_size(bytes)
}

pub fn name_length_beyond_packet() -> Vec<u8> {
    let mut bytes = strip("overlong", 4).encode();
    bytes[NAME_LEN_OFFSET..NAME_LEN_OFFSET + 2].copy_from_slice(&60_000u16.to_le_bytes());
    bytes
}

pub fn unterminated_name() -> Vec<u8> {
    let mut bytes = strip("unterminated", 4).encode();
    let name_len = u16::from_le_bytes([bytes[NAME_LEN_OFFSET], bytes[NAME_LEN_OFFSET + 1]]);
    bytes[NAME_LEN_OFFSET + 2 + name_len as usize - 1] = b'X';
    bytes
}

#[tokio::test]
async fn rejects_malformed_controller_data() {
    let server = FakeServer::new();
    server.add_device(0, strip("good", 4));
    server.add_controller(1, "truncated", ControllerBlock::Raw(truncated_block()));
    server.add_controller(
        2,
        "overlong",
        ControllerBlock::Raw(name_length_beyond_packet()),
    );
    server.add_controller(3, "unterminated", ControllerBlock::Raw(unterminated_name()));
    let mut uneven = strip("uneven", 4);
    uneven.led_count_override = Some(3);
    server.add_device(4, uneven);
    server.add_device(5, strip("inverted", 0).resizable_zone("Header", 10, 5, 7));
    let mut bad_mode = strip("bad-mode", 4);
    bad_mode.active_mode = 9;
    server.add_device(6, bad_mode);
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    let inventory = backend
        .refresh_inventory()
        .await
        .expect("bad controllers must not fail the whole inventory");

    assert_eq!(inventory.devices.len(), 1);
    assert_eq!(inventory.devices[0].descriptor.serial, "SN-good");
    let issues: BTreeMap<u32, IssueReason> = inventory
        .issues
        .iter()
        .map(|issue| (issue.runtime_id, issue.reason))
        .collect();
    assert_eq!(
        issues,
        BTreeMap::from([
            (1, IssueReason::MalformedData),
            (2, IssueReason::MalformedData),
            (3, IssueReason::MalformedData),
            (4, IssueReason::InconsistentData),
            (5, IssueReason::InconsistentData),
            (6, IssueReason::InconsistentData),
        ])
    );
    let good = support::reference_by_serial(inventory, "SN-good");
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Degraded {
            reasons: vec![StateReason::ControllerDataRejected { controllers: 6 }]
        }
    );

    backend
        .write_device_frame(&good, &solid(RED, 4))
        .await
        .expect("valid device stays usable");
    let writes = server.wait_for_color_writes(1).await;
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].target.as_deref(), Some("good"));
}

#[tokio::test]
async fn rejects_oversized_packet_header_before_reading_body() {
    let server = FakeServer::new();
    let mut client = OpenRgbClient::connect(server.connect(), support::config())
        .await
        .expect("connect");

    server.broadcast(wire::header(
        0,
        wire::REQUEST_CONTROLLER_COUNT,
        MAX_PACKET_SIZE + 1,
    ));
    let error = client.controller_ids().await.unwrap_err();

    assert_eq!(
        error,
        ClientError::Frame(FrameError::TooLarge {
            size: MAX_PACKET_SIZE + 1,
            limit: MAX_PACKET_SIZE
        })
    );
}

#[tokio::test]
async fn oversized_packet_closes_backend_connection() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let mut backend = support::backend(&server);
    backend.connect().await.expect("connect");

    server.broadcast(wire::header(0, wire::SIGNAL_UPDATE, u32::MAX));
    let error = backend.refresh_inventory().await.unwrap_err();

    assert!(
        matches!(error, BackendError::ProtocolViolation { .. }),
        "{error:?}"
    );
    assert_eq!(
        backend.connection_state(),
        &ConnectionState::Disconnected {
            reason: Some(StateReason::ConnectionLost {
                cause: LossCause::ProtocolViolation
            })
        }
    );
}
