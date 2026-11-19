use prismforge_core::profile::NameProblem;
use prismforge_core::session::SessionDevice;
use prismforge_core::store::{
    FileProblem, GROUP_LIMIT, GROUPS_FORMAT, GroupId, GroupsFile, MEMBER_LIMIT, MemberState,
    NoticeEvent, StdFileOps, StoreError, resolve_members,
};
use prismforge_core::{DeviceIdentity, DeviceRef, OutputTarget};
use serde_json::{Value, json};

use crate::session::demo;
use crate::support::TempDir;

const FILE: &str = "groups.json";
const GPU: &str = "demo-graphics-card";
const BOARD: &str = "demo-motherboard";

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

fn zone(device: &str, zone: usize) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: Some(zone),
    }
}

async fn devices() -> Vec<SessionDevice> {
    let (handle, _) = demo().await;
    handle.snapshot().devices
}

fn stored(dir: &TempDir) -> Value {
    serde_json::from_slice(&dir.read(FILE)).expect("json")
}

fn names(file: &GroupsFile) -> Vec<String> {
    file.groups()
        .iter()
        .map(|group| group.name.clone())
        .collect()
}

#[tokio::test(start_paused = true)]
async fn groups_round_trip_with_their_members_format_and_version() {
    let dir = TempDir::new("groups-round-trip");
    let devices = devices().await;
    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    assert!(file.groups().is_empty());
    assert!(dir.files().is_empty(), "opening writes nothing");

    let fans = file
        .create("  Case fans ", &[whole(GPU), zone(BOARD, 1)], &devices)
        .expect("create");
    file.create("Desk", &[zone(BOARD, 2)], &devices)
        .expect("second");
    let again = GroupsFile::open(dir.path(), StdFileOps);

    assert_eq!(names(&again), vec!["Case fans", "Desk"]);
    let group = again.group(&fans).expect("stored group");
    let members: Vec<(&str, Option<usize>, &str, Option<&str>)> = group
        .members
        .iter()
        .map(|member| {
            (
                member.device.as_str(),
                member.zone,
                member.label.as_str(),
                member.zone_name.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        members,
        vec![
            (GPU, None, "Demo Graphics Card", None),
            (
                BOARD,
                Some(1),
                "Demo Motherboard",
                Some("Addressable Header 1")
            ),
        ]
    );
    let json = stored(&dir);
    assert_eq!(json["format"], GROUPS_FORMAT);
    assert_eq!(json["schema_version"], json!({ "major": 1, "minor": 0 }));
    assert!(again.notices().is_empty());
}

#[tokio::test(start_paused = true)]
async fn rename_add_remove_and_delete_change_only_their_group() {
    let dir = TempDir::new("groups-edit");
    let devices = devices().await;
    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    let fans = file.create("Fans", &[whole(GPU)], &devices).expect("fans");
    let desk = file
        .create("Desk", &[zone(BOARD, 0)], &devices)
        .expect("desk");

    file.rename(&fans, "Case fans").expect("rename");
    file.add_members(&fans, &[zone(BOARD, 2), whole(GPU)], &devices)
        .expect("add");
    file.remove_member(&fans, &whole(GPU)).expect("remove");
    file.delete(&desk).expect("delete");
    let again = GroupsFile::open(dir.path(), StdFileOps);

    assert_eq!(names(&again), vec!["Case fans"]);
    let members: Vec<OutputTarget> = again
        .group(&fans)
        .expect("fans")
        .members
        .iter()
        .map(|member| member.target())
        .collect();
    assert_eq!(members, vec![zone(BOARD, 2)]);
    assert_eq!(
        file.delete(&desk),
        Err(StoreError::UnknownGroup { id: desk.clone() })
    );
}

#[tokio::test(start_paused = true)]
async fn names_members_and_counts_are_checked() {
    let dir = TempDir::new("groups-limits");
    let devices = devices().await;
    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    let fans = file.create("Fans", &[whole(GPU)], &devices).expect("fans");

    assert_eq!(
        file.create(" ", &[whole(GPU)], &devices),
        Err(StoreError::InvalidName {
            problem: NameProblem::Empty
        })
    );
    assert_eq!(
        file.create("fans", &[whole(GPU)], &devices),
        Err(StoreError::GroupNameTaken {
            name: "fans".into()
        })
    );
    assert_eq!(file.rename(&fans, "Fans"), Ok(()));
    assert_eq!(
        file.create("Ghost", &[whole("no-such-device")], &devices),
        Err(StoreError::UnknownDevice {
            device: DeviceRef::new("no-such-device")
        })
    );
    assert_eq!(
        file.create("Far", &[zone(GPU, 4)], &devices),
        Err(StoreError::UnknownZone {
            device: DeviceRef::new(GPU),
            zone: 4
        })
    );
    let many = vec![whole(GPU); MEMBER_LIMIT + 1];
    assert_eq!(
        file.add_members(&fans, &many, &devices),
        Err(StoreError::TooManyMembers {
            limit: MEMBER_LIMIT
        })
    );
    for index in 1..GROUP_LIMIT {
        file.create(&format!("Group {index}"), &[], &devices)
            .expect("within the limit");
    }
    assert_eq!(
        file.create("One more", &[], &devices),
        Err(StoreError::TooManyGroups { limit: GROUP_LIMIT })
    );
    assert_eq!(file.groups().len(), GROUP_LIMIT);
}

#[tokio::test(start_paused = true)]
async fn members_resolve_against_todays_devices_and_are_never_guessed() {
    let dir = TempDir::new("groups-resolve");
    let mut devices = devices().await;
    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    let id = file
        .create(
            "Everything I had",
            &[
                whole(GPU),
                zone(BOARD, 1),
                whole("demo-memory-1"),
                whole("demo-memory-2"),
                zone(BOARD, 2),
            ],
            &devices,
        )
        .expect("create");

    let board = devices
        .iter_mut()
        .find(|device| device.identity.reference().map(DeviceRef::as_str) == Some(BOARD))
        .expect("board");
    board.descriptor.zones[2].name = "Renamed by OpenRGB".into();
    let memory = devices
        .iter_mut()
        .find(|device| device.identity.reference().map(DeviceRef::as_str) == Some("demo-memory-1"))
        .expect("memory");
    memory.identity = DeviceIdentity::Ambiguous {
        candidates: vec![
            DeviceRef::new("demo-memory-1"),
            DeviceRef::new("demo-memory-9"),
        ],
    };
    devices.retain(|device| {
        device.identity.reference().map(DeviceRef::as_str) != Some("demo-memory-2")
    });
    for device in &mut devices {
        if device.identity.reference().map(DeviceRef::as_str) == Some(GPU) {
            device.available = false;
        }
    }

    let group = file.group(&id).expect("group");
    let states: Vec<MemberState> = resolve_members(&group.members, &devices)
        .into_iter()
        .map(|member| member.state)
        .collect();

    assert_eq!(
        states,
        vec![
            MemberState::Unavailable,
            MemberState::Resolved,
            MemberState::Ambiguous,
            MemberState::Missing,
            MemberState::Changed,
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn members_missing_today_are_kept_through_every_edit() {
    let dir = TempDir::new("groups-keep");
    let devices = devices().await;
    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    let id = file
        .create("Desk", &[whole(GPU), whole("demo-memory-2")], &devices)
        .expect("create");
    let today: Vec<SessionDevice> = devices
        .iter()
        .filter(|device| device.identity.reference().map(DeviceRef::as_str) != Some(GPU))
        .cloned()
        .collect();

    file.rename(&id, "Desk lights").expect("rename");
    file.add_members(&id, &[zone(BOARD, 0)], &today)
        .expect("add");
    file.remove_member(&id, &whole("demo-memory-2"))
        .expect("remove");
    let again = GroupsFile::open(dir.path(), StdFileOps);

    let group = again.group(&id).expect("group");
    let kept: Vec<(OutputTarget, MemberState)> = resolve_members(&group.members, &today)
        .into_iter()
        .map(|member| (member.target, member.state))
        .collect();
    assert_eq!(
        kept,
        vec![
            (whole(GPU), MemberState::Missing),
            (zone(BOARD, 0), MemberState::Resolved)
        ]
    );
    assert_eq!(group.members[0].label, "Demo Graphics Card");
}

#[tokio::test(start_paused = true)]
async fn unknown_fields_survive_a_rewrite() {
    let dir = TempDir::new("groups-extra");
    let devices = devices().await;
    dir.write(
        FILE,
        serde_json::to_vec_pretty(&json!({
            "format": GROUPS_FORMAT,
            "schema_version": { "major": 1, "minor": 4 },
            "later": "kept",
            "groups": [{
                "id": "g-00000000000000aa",
                "name": "Old",
                "colour_hint": "teal",
                "members": [{ "device": GPU, "zone": null, "label": "Card", "zone_name": null, "pinned": true }]
            }]
        }))
        .expect("json")
        .as_slice(),
    );
    let mut file = GroupsFile::open(dir.path(), StdFileOps);

    file.create("New", &[whole(GPU)], &devices).expect("create");

    let json = stored(&dir);
    assert_eq!(json["later"], "kept");
    assert_eq!(json["groups"][0]["colour_hint"], "teal");
    assert_eq!(json["groups"][0]["members"][0]["pinned"], true);
    assert_eq!(json["schema_version"], json!({ "major": 1, "minor": 0 }));
}

#[tokio::test(start_paused = true)]
async fn a_damaged_groups_file_is_restored_from_its_backup_with_a_notice() {
    let dir = TempDir::new("groups-backup");
    let devices = devices().await;
    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    file.create("Fans", &[whole(GPU)], &devices).expect("first");
    file.create("Desk", &[whole(GPU)], &devices)
        .expect("second");
    dir.write(FILE, b"{ not json");

    let again = GroupsFile::open(dir.path(), StdFileOps);

    assert_eq!(names(&again), vec!["Fans"]);
    assert!(matches!(
        again.notices()[0].event,
        NoticeEvent::RestoredFromBackup {
            problem: Some(FileProblem::NotJson { .. }),
            ..
        }
    ));
    assert_eq!(again.notices()[0].file, FILE);
}

#[tokio::test(start_paused = true)]
async fn an_invalid_groups_file_is_not_used() {
    let dir = TempDir::new("groups-invalid");
    dir.write(
        FILE,
        serde_json::to_vec(&json!({
            "format": GROUPS_FORMAT,
            "schema_version": { "major": 1, "minor": 0 },
            "groups": [{ "id": "NOT AN ID", "name": "Bad", "members": [] }]
        }))
        .expect("json")
        .as_slice(),
    );

    let file = GroupsFile::open(dir.path(), StdFileOps);

    assert!(file.groups().is_empty());
    assert!(matches!(
        file.notices()[0].event,
        NoticeEvent::StartedEmpty { .. }
    ));
}

#[tokio::test(start_paused = true)]
async fn a_newer_groups_file_is_left_untouched_and_refuses_changes() {
    let dir = TempDir::new("groups-newer");
    let devices = devices().await;
    let newer = serde_json::to_vec(&json!({
        "format": GROUPS_FORMAT,
        "schema_version": { "major": 2, "minor": 0 },
        "groups": []
    }))
    .expect("json");
    dir.write(FILE, &newer);

    let mut file = GroupsFile::open(dir.path(), StdFileOps);
    let refused = file.create("Fans", &[whole(GPU)], &devices);

    assert!(matches!(refused, Err(StoreError::NewerFile { .. })));
    assert!(matches!(
        file.notices()[0].event,
        NoticeEvent::LeftUntouched {
            problem: FileProblem::NewerVersion { found: 2, .. }
        }
    ));
    assert_eq!(dir.read(FILE), newer);
    assert_eq!(dir.files(), vec![FILE.to_owned()]);
}

#[test]
fn group_ids_are_safe_file_and_json_values() {
    assert!(GroupId::parse("g-00000000000000aa").is_some());
    assert!(GroupId::parse("G-1").is_none());
    assert!(GroupId::parse("g-../x").is_none());
    assert!(GroupId::parse("").is_none());
}
