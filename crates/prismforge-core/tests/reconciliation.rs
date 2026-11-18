use prismforge_core::{
    AmbiguousGroup, DeviceFingerprint, DeviceKind, DeviceRef, IdentityMatch, KnownDevice, ZoneKind,
    ZoneShape, reconcile,
};

fn fingerprint(model: &str, serial: Option<&str>, location: Option<&str>) -> DeviceFingerprint {
    DeviceFingerprint {
        kind: DeviceKind::Keyboard,
        vendor: "Vendor".into(),
        model: model.into(),
        serial: serial.map(Into::into),
        location: location.map(Into::into),
        zones: vec![ZoneShape {
            name: "Keys".into(),
            kind: ZoneKind::Matrix,
            leds_min: 10,
            leds_max: 10,
        }],
    }
}

fn known(reference: &str, fingerprint: DeviceFingerprint) -> KnownDevice {
    KnownDevice {
        reference: DeviceRef::new(reference),
        fingerprint,
    }
}

fn matched(reference: &str, current: usize) -> IdentityMatch {
    IdentityMatch {
        reference: DeviceRef::new(reference),
        current,
    }
}

#[test]
fn matches_devices_by_identity_not_list_position() {
    let a = fingerprint("Alpha", Some("SN-A"), Some("usb:1"));
    let b = fingerprint("Beta", None, Some("i2c:0x58"));
    let known_devices = [known("ref-a", a.clone()), known("ref-b", b.clone())];

    let result = reconcile(&known_devices, &[b, a]);

    assert_eq!(
        result.matched,
        vec![matched("ref-b", 0), matched("ref-a", 1)]
    );
    assert!(result.ambiguous.is_empty());
    assert!(result.missing.is_empty());
    assert!(result.new.is_empty());
}

#[test]
fn serial_device_keeps_reference_after_location_change() {
    let before = fingerprint("Alpha", Some("SN-A"), Some("usb:1"));
    let after = fingerprint("Alpha", Some("SN-A"), Some("usb:7"));

    let result = reconcile(&[known("ref-a", before)], &[after]);

    assert_eq!(result.matched, vec![matched("ref-a", 0)]);
    assert!(result.new.is_empty());
}

#[test]
fn serial_less_device_at_new_location_is_missing_and_new() {
    let before = fingerprint("Alpha", None, Some("usb:1"));
    let after = fingerprint("Alpha", None, Some("usb:7"));

    let result = reconcile(&[known("ref-a", before)], &[after]);

    assert!(result.matched.is_empty());
    assert_eq!(result.missing, vec![DeviceRef::new("ref-a")]);
    assert_eq!(result.new.len(), 1);
    assert_eq!(result.new[0].current, 0);
    assert_ne!(result.new[0].reference, DeviceRef::new("ref-a"));
}

#[test]
fn different_device_at_same_position_and_location_is_not_matched() {
    let before = fingerprint("Alpha", None, Some("usb:1"));
    let replacement = fingerprint("Gamma", None, Some("usb:1"));

    let result = reconcile(&[known("ref-a", before)], &[replacement]);

    assert!(result.matched.is_empty());
    assert_eq!(result.missing, vec![DeviceRef::new("ref-a")]);
    assert_eq!(result.new.len(), 1);
}

#[test]
fn changed_zone_shape_prevents_match() {
    let before = fingerprint("Alpha", Some("SN-A"), Some("usb:1"));
    let mut after = before.clone();
    after.zones[0].leds_max = 20;

    let result = reconcile(&[known("ref-a", before)], &[after]);

    assert!(result.matched.is_empty());
    assert_eq!(result.missing, vec![DeviceRef::new("ref-a")]);
    assert_eq!(result.new.len(), 1);
}

#[test]
fn indistinguishable_current_devices_stay_unresolved() {
    let twin = fingerprint("Alpha", None, Some("hid:0"));

    let result = reconcile(&[known("ref-a", twin.clone())], &[twin.clone(), twin]);

    assert!(result.matched.is_empty());
    assert!(result.new.is_empty());
    assert!(result.missing.is_empty());
    assert_eq!(
        result.ambiguous,
        vec![AmbiguousGroup {
            candidates: vec![DeviceRef::new("ref-a")],
            current: vec![0, 1],
        }]
    );
}

#[test]
fn indistinguishable_new_devices_get_no_reference() {
    let twin = fingerprint("Alpha", None, None);

    let result = reconcile(&[], &[twin.clone(), twin]);

    assert!(result.new.is_empty());
    assert_eq!(
        result.ambiguous,
        vec![AmbiguousGroup {
            candidates: vec![],
            current: vec![0, 1],
        }]
    );
}

#[test]
fn indistinguishable_known_devices_stay_unresolved() {
    let twin = fingerprint("Alpha", Some("SN-SHARED"), Some("usb:1"));
    let known_devices = [known("ref-1", twin.clone()), known("ref-2", twin.clone())];

    let result = reconcile(&known_devices, &[twin]);

    assert!(result.matched.is_empty());
    assert_eq!(
        result.ambiguous,
        vec![AmbiguousGroup {
            candidates: vec![DeviceRef::new("ref-1"), DeviceRef::new("ref-2")],
            current: vec![0],
        }]
    );
}

#[test]
fn shared_serial_at_new_locations_stays_unresolved() {
    let first = fingerprint("Alpha", Some("SN-SHARED"), Some("usb:1"));
    let second = fingerprint("Alpha", Some("SN-SHARED"), Some("usb:2"));
    let known_devices = [known("ref-1", first), known("ref-2", second)];

    let result = reconcile(
        &known_devices,
        &[
            fingerprint("Alpha", Some("SN-SHARED"), Some("usb:3")),
            fingerprint("Alpha", Some("SN-SHARED"), Some("usb:4")),
        ],
    );

    assert!(result.matched.is_empty());
    assert_eq!(
        result.ambiguous,
        vec![AmbiguousGroup {
            candidates: vec![DeviceRef::new("ref-1"), DeviceRef::new("ref-2")],
            current: vec![0, 1],
        }]
    );
}

#[test]
fn new_references_are_deterministic_and_unique() {
    let a = fingerprint("Alpha", Some("SN-A"), None);
    let b = fingerprint("Beta", Some("SN-B"), None);
    let current = [a, b];

    let first = reconcile(&[], &current);
    let second = reconcile(&[], &current);

    assert_eq!(first.new, second.new);
    assert_eq!(first.new.len(), 2);
    assert_ne!(first.new[0].reference, first.new[1].reference);
}

#[test]
fn new_reference_never_reuses_a_known_reference() {
    let gone = fingerprint("Alpha", None, Some("usb:1"));
    let arrived = fingerprint("Alpha", None, Some("usb:2"));
    let minted = reconcile(&[], std::slice::from_ref(&arrived)).new[0]
        .reference
        .clone();

    let result = reconcile(
        &[KnownDevice {
            reference: minted.clone(),
            fingerprint: gone,
        }],
        &[arrived],
    );

    assert_eq!(result.missing, vec![minted.clone()]);
    assert_ne!(result.new[0].reference, minted);
}
