use prismforge_core::RgbBackend;
use prismforge_core::automation::{EventProfile, EventProfiles};
use prismforge_openrgb::protocol::{
    PayloadError, SERVER_FLAG_SUPPORTS_SETTINGS_MANAGER, decode_event_profiles,
    decode_server_flags, encode_settings_key, packet_id,
};
use prismforge_testkit::{FakeServer, wire};

use crate::support::{backend, strip};

fn reply(json: &str) -> Vec<u8> {
    let mut bytes = json.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

#[test]
fn the_settings_packets_have_the_ids_of_protocol_6() {
    assert_eq!(packet_id::SET_CLIENT_FLAGS, 52);
    assert_eq!(packet_id::SET_SERVER_FLAGS, 53);
    assert_eq!(packet_id::SETTINGS_GET, 250);
    assert_eq!(SERVER_FLAG_SUPPORTS_SETTINGS_MANAGER, 1 << 4);
}

#[test]
fn a_settings_request_is_the_key_with_one_nul() {
    assert_eq!(encode_settings_key("ProfileManager"), b"ProfileManager\0");
}

#[test]
fn server_flags_are_one_little_endian_word() {
    assert_eq!(decode_server_flags(&[0x11, 0, 0, 0]), Ok(0x11));
    assert_eq!(
        decode_server_flags(&[0x11, 0]),
        Err(PayloadError::Length {
            expected: 4,
            actual: 2
        })
    );
}

#[test]
fn decodes_the_six_event_profiles_and_ignores_other_keys() {
    let payload = reply(
        r#"{"open_profile":{"enabled":true,"name":"Day"},"exit_profile":{"enabled":false,"name":"Night"},
        "resume_profile":{"enabled":true,"name":"Day"},"suspend_profile":{"enabled":true,"name":"Dark"},
        "service_startup_profile":{"enabled":false,"name":""},"service_shutdown_profile":{"enabled":true,"name":"Off"},
        "something_newer":{"enabled":true}}"#,
    );
    let profile = |enabled: bool, name: &str| EventProfile {
        enabled,
        name: name.into(),
    };
    assert_eq!(
        decode_event_profiles(&payload),
        Ok(EventProfiles {
            open: profile(true, "Day"),
            exit: profile(false, "Night"),
            resume: profile(true, "Day"),
            suspend: profile(true, "Dark"),
            service_startup: profile(false, ""),
            service_shutdown: profile(true, "Off"),
        })
    );
}

#[test]
fn a_missing_event_setting_is_off() {
    assert_eq!(
        decode_event_profiles(&reply("{}")),
        Ok(EventProfiles::default())
    );
}

#[test]
fn malformed_settings_are_refused_not_guessed() {
    for bad in [
        b"{}".to_vec(),
        reply("not json"),
        reply("[]"),
        reply(r#"{"suspend_profile":{"enabled":"yes","name":"Dark"}}"#),
        reply(r#"{"suspend_profile":{"enabled":true}}"#),
    ] {
        assert!(
            decode_event_profiles(&bad).is_err(),
            "{:?}",
            String::from_utf8_lossy(&bad)
        );
    }
}

#[tokio::test(start_paused = true)]
async fn the_backend_asks_for_the_flags_then_reads_profile_manager_and_never_changes_settings() {
    let server = FakeServer::new();
    server.add_device(0, strip("a", 4));
    server.set_settings(
        "ProfileManager",
        r#"{"resume_profile":{"enabled":true,"name":"Day"}}"#,
    );
    let mut backend = backend(&server);
    backend.connect().await.expect("connect");

    let read = backend.event_profiles().await.expect("read");
    assert_eq!(read.expect("supported").resume.name, "Day");
    let flags = server.received_with_id(wire::SET_CLIENT_FLAGS);
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].payload, 0u32.to_le_bytes());
    let gets = server.received_with_id(wire::SETTINGS_GET);
    assert_eq!(gets.len(), 1);
    assert_eq!(gets[0].payload, b"ProfileManager\0");
    for changing in 251..=254 {
        assert!(server.received_with_id(changing).is_empty());
    }
}

#[tokio::test(start_paused = true)]
async fn a_server_without_a_settings_manager_is_not_asked() {
    let server = FakeServer::new();
    server.add_device(0, strip("a", 4));
    server.set_server_flags(1);
    let mut backend = backend(&server);
    backend.connect().await.expect("connect");
    assert_eq!(backend.event_profiles().await, Ok(None));
    assert!(server.received_with_id(wire::SETTINGS_GET).is_empty());
}
