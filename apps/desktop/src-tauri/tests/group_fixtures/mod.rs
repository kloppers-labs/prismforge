use std::time::Duration;

use prismforge_core::session::SessionSnapshot;
use prismforge_core::{ConnectionState, DeviceRef, OutputTarget, RgbBackend};
use prismforge_desktop::{Shell, ShellConfig, ShellView, Stores, ViewSink};
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice};

use crate::Module;
use crate::support::temp::TempDir;

struct Silent;

impl ViewSink for Silent {
    fn deliver(&self, _: &ShellView) {}
}

fn target(device: &DeviceRef, zone: Option<usize>) -> OutputTarget {
    OutputTarget {
        device: device.clone(),
        zone,
    }
}

fn strip(label: &str, name: &str) -> FixtureDevice {
    FixtureDevice::new(label, name)
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .zone("Strip", 10)
}

async fn started<B: RgbBackend + Send + 'static>(
    shell: &Shell<Silent>,
    backend: B,
) -> SessionSnapshot {
    shell.start(backend).await.expect("start");
    loop {
        if let Some(snapshot) = shell.snapshot().await
            && snapshot.connection == ConnectionState::Connected
            && !snapshot.devices.is_empty()
        {
            return snapshot;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn reference(snapshot: &SessionSnapshot, serial: &str) -> DeviceRef {
    snapshot
        .devices
        .iter()
        .find(|device| device.descriptor.serial == serial)
        .and_then(|device| device.identity.reference())
        .cloned()
        .expect("resolved device")
}

/// Group views from real stores: a demo store, and a real store whose group lost a device.
pub async fn generate(groups: &mut Module, sessions: &mut Module) {
    let dir = TempDir::new("fixtures-groups");
    let shell = Shell::new(
        Silent,
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    started(&shell, DemoBackend::new()).await;
    groups.add("groupsEmpty", &shell.groups_view().await.expect("groups"));
    let gpu = DeviceRef::new("demo-graphics-card");
    let board = DeviceRef::new("demo-motherboard");
    shell
        .create_group("Desk", vec![target(&gpu, None), target(&board, Some(1))])
        .await
        .expect("desk");
    let demo = shell
        .create_group(
            "Memory",
            vec![
                target(&DeviceRef::new("demo-memory-1"), None),
                target(&DeviceRef::new("demo-memory-2"), None),
            ],
        )
        .await
        .expect("memory");
    groups.add("groupsDemo", &demo);

    let server = FakeServer::new();
    server.add_device(0, strip("shelf", "Fixture Shelf Strip"));
    server.add_device(1, strip("desk", "Fixture Desk Strip"));
    let snapshot = started(
        &shell,
        OpenRgbBackend::new(server.connector(), ClientConfig::default()),
    )
    .await;
    let shelf = reference(&snapshot, "SN-shelf");
    let desk = reference(&snapshot, "SN-desk");
    shell
        .create_group(
            "Shelf and desk",
            vec![target(&shelf, None), target(&desk, None)],
        )
        .await
        .expect("real group");
    shell.stop().await;
    let server = FakeServer::new();
    server.add_device(0, strip("desk", "Fixture Desk Strip"));
    let snapshot = started(
        &shell,
        OpenRgbBackend::new(server.connector(), ClientConfig::default()),
    )
    .await;
    groups.add(
        "groupsRealMissing",
        &shell.groups_view().await.expect("groups"),
    );
    sessions.add("openRgbShelfGone", &snapshot);
    shell.stop().await;
}
