mod support;

use prismforge_core::store::{GroupId, MemberState, StoreError};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_desktop::{CommandError, GroupsView, Shell, ShellConfig, StoreKind, Stores};
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice};

use support::temp::TempDir;
use support::{Recorder, is_connected};

const GPU: &str = "demo-graphics-card";

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

fn shell_on(dir: &TempDir) -> (Shell<Recorder>, Recorder) {
    let recorder = Recorder::default();
    let stores = Stores::new(dir.path().to_path_buf());
    (
        Shell::new(recorder.clone(), ShellConfig::default(), stores),
        recorder,
    )
}

async fn demo(shell: &Shell<Recorder>, recorder: &Recorder) {
    shell.start(DemoBackend::new()).await.expect("start demo");
    recorder.wait_for(is_connected).await;
}

fn group<'a>(view: &'a GroupsView, name: &str) -> &'a prismforge_desktop::GroupView {
    view.groups
        .iter()
        .find(|group| group.name == name)
        .unwrap_or_else(|| panic!("no group {name}"))
}

#[tokio::test(start_paused = true)]
async fn groups_are_created_edited_and_deleted_through_the_shell() {
    let dir = TempDir::new("groups-shell");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;

    let created = shell
        .create_group("Case fans", vec![whole(GPU), whole("demo-memory-1")])
        .await
        .expect("create");
    let id = group(&created, "Case fans").id.clone();
    let renamed = shell.rename_group(&id, "Fans").await.expect("rename");
    let added = shell
        .add_group_members(&id, vec![whole("demo-memory-2")])
        .await
        .expect("add");
    let removed = shell
        .remove_group_member(&id, whole(GPU))
        .await
        .expect("remove");
    let taken = shell.create_group("fans", Vec::new()).await;
    let deleted = shell.delete_group(&id).await.expect("delete");
    let unknown = shell.delete_group(&id).await;

    assert_eq!(created.kind, StoreKind::Demo);
    assert_eq!(group(&renamed, "Fans").members.len(), 2);
    assert_eq!(group(&added, "Fans").members.len(), 3);
    let left: Vec<&str> = group(&removed, "Fans")
        .members
        .iter()
        .map(|member| member.target.device.as_str())
        .collect();
    assert_eq!(left, vec!["demo-memory-1", "demo-memory-2"]);
    assert!(
        group(&removed, "Fans")
            .members
            .iter()
            .all(|member| member.state == MemberState::Resolved)
    );
    assert_eq!(
        taken,
        Err(CommandError::Store(StoreError::GroupNameTaken {
            name: "fans".into()
        }))
    );
    assert!(deleted.groups.is_empty());
    assert_eq!(
        unknown,
        Err(CommandError::Store(StoreError::UnknownGroup { id }))
    );
    assert!(dir.path().join("demo-store").join("groups.json").exists());
    assert!(!dir.path().join("store").join("groups.json").exists());
}

#[tokio::test(start_paused = true)]
async fn demo_groups_never_meet_real_devices_and_members_missing_today_are_shown() {
    let dir = TempDir::new("groups-apart");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    shell
        .create_group("Demo desk", vec![whole(GPU)])
        .await
        .expect("create");
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Desk strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Strip", 4),
    );

    shell
        .start(OpenRgbBackend::new(
            server.connector(),
            ClientConfig::default(),
        ))
        .await
        .expect("openrgb");
    recorder.wait_for(is_connected).await;
    let real = shell.groups_view().await.expect("real groups");
    shell.stop().await;
    let offline = shell.groups_view().await.expect("no session");
    demo(&shell, &recorder).await;
    let back = shell.groups_view().await.expect("demo groups");

    assert_eq!(real.kind, StoreKind::Real);
    assert!(real.groups.is_empty());
    assert_eq!(offline.kind, StoreKind::Real);
    let members = &group(&back, "Demo desk").members;
    assert_eq!(members[0].state, MemberState::Resolved);
    assert_eq!(members[0].label, "Demo Graphics Card");
    assert!(back.notices.is_empty());
    assert_eq!(
        shell
            .rename_group(&GroupId::parse("g-0").expect("id"), "Nope")
            .await,
        Err(CommandError::Store(StoreError::UnknownGroup {
            id: GroupId::parse("g-0").expect("id")
        }))
    );
}
