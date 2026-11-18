use std::time::Duration;

use prismforge_core::session::RestoreOutcome;
use prismforge_core::{ConnectionState, LossCause, StateReason};
use prismforge_testkit::FakeServer;

use crate::support::{
    self, is_connected, is_retrying, output_packets, reference, strip, wait_until,
};

const BLINK: Duration = Duration::from_secs(1);

#[tokio::test(start_paused = true)]
async fn backend_restart_reconnects_and_keeps_references_on_their_devices() {
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    server.add_device(1, strip("beta", 2));
    let handle = support::connected_openrgb(&server).await;
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    let beta = reference(&handle.snapshot(), "SN-beta");
    let first = handle
        .identify(alpha.clone(), None, BLINK)
        .await
        .expect("identify before restart");
    assert_eq!(first.restore, RestoreOutcome::Restored);

    server.restart(vec![
        (0, strip("beta", 2).into()),
        (1, strip("alpha", 3).into()),
        (2, strip("gamma", 4).into()),
    ]);
    let retrying = wait_until(&handle, is_retrying).await;
    let reconnected = wait_until(&handle, is_connected).await;
    let second = handle
        .identify(alpha.clone(), None, BLINK)
        .await
        .expect("identify after restart");

    assert_eq!(
        retrying.connection,
        ConnectionState::Retrying {
            attempt: 1,
            reason: StateReason::ConnectionLost {
                cause: LossCause::ServerClosed
            }
        }
    );
    assert_eq!(second.restore, RestoreOutcome::Restored);
    assert_eq!(reconnected.devices.len(), 3);
    assert_eq!(reference(&reconnected, "SN-alpha"), alpha);
    assert_eq!(reference(&reconnected, "SN-beta"), beta);
    assert!(reconnected.devices.iter().all(|device| device.available));
    assert_eq!(reconnected.diagnostics.counters.inventory_refreshes, 2);
    server.wait_for_color_writes(4).await;
    let packets = output_packets(&server);
    assert_eq!(packets.len(), 6, "{packets:?}");
    assert!(
        packets
            .iter()
            .all(|packet| packet.target.as_deref() == Some("alpha")),
        "{packets:?}"
    );
    let ids: Vec<(u32, u32)> = packets
        .iter()
        .map(|packet| (packet.session, packet.device_id))
        .collect();
    assert_eq!(ids, [(0, 0), (0, 0), (0, 0), (1, 1), (1, 1), (1, 1)]);
}
