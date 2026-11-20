use prismforge_core::automation::{
    DeviceLightsOff, LightsOffOutcome, LightsOffSkip, LightsOffStep, LightsOffWrite,
    plan_lights_off,
};
use prismforge_core::session::SessionDevice;
use prismforge_core::{
    BackendOrigin, DeviceCapabilities, DeviceDescriptor, DeviceIdentity, DeviceKind, DeviceRef,
    ModeDescriptor, ModePersistence,
};

fn mode(name: &str, persistence: ModePersistence) -> ModeDescriptor {
    ModeDescriptor {
        name: name.into(),
        speed: None,
        brightness: None,
        directions: Vec::new(),
        per_led_color: name == "Direct",
        mode_colors: None,
        random_color: false,
        applies_to_entire_device: true,
        persistence,
    }
}

fn device(reference: &str, name: &str, leds: usize, modes: Vec<ModeDescriptor>) -> SessionDevice {
    let direct_mode = modes.iter().position(|mode| mode.name == "Direct");
    SessionDevice {
        identity: DeviceIdentity::Matched(DeviceRef::new(reference)),
        descriptor: DeviceDescriptor {
            origin: BackendOrigin::OpenRgb,
            name: name.into(),
            vendor: "Vendor".into(),
            description: String::new(),
            version: String::new(),
            serial: String::new(),
            location: String::new(),
            kind: DeviceKind::Unknown,
            zones: Vec::new(),
            led_count: leds,
            colors: Vec::new(),
            capabilities: DeviceCapabilities {
                direct_mode,
                active_mode: Some(0),
                modes,
                resizable_zones: Vec::new(),
                saves_to_device_memory: false,
            },
        },
        available: true,
    }
}

fn skipped(reference: Option<&str>, name: &str, reason: LightsOffSkip) -> DeviceLightsOff {
    DeviceLightsOff {
        device: reference.map(DeviceRef::new),
        name: name.into(),
        outcome: LightsOffOutcome::Skipped { reason },
    }
}

#[test]
fn a_device_with_direct_mode_gets_black_on_every_led() {
    let strip = device(
        "strip",
        "Strip",
        12,
        vec![
            mode("Direct", ModePersistence::None),
            mode("Off", ModePersistence::None),
        ],
    );
    let plan = plan_lights_off(&[strip]);
    assert_eq!(
        plan.steps,
        [LightsOffStep {
            device: DeviceRef::new("strip"),
            name: "Strip".into(),
            write: LightsOffWrite::Black { leds: 12 },
        }]
    );
    assert!(plan.skipped.is_empty());
}

#[test]
fn a_device_without_direct_mode_gets_its_off_mode_when_that_mode_does_not_save() {
    let card = device(
        "card",
        "Graphics Card",
        4,
        vec![
            mode("Static", ModePersistence::ManualSave),
            mode("off", ModePersistence::ManualSave),
        ],
    );
    let plan = plan_lights_off(&[card]);
    assert_eq!(
        plan.steps,
        [LightsOffStep {
            device: DeviceRef::new("card"),
            name: "Graphics Card".into(),
            write: LightsOffWrite::OffMode {
                mode: 1,
                name: "off".into()
            },
        }]
    );
}

#[test]
fn an_off_mode_that_saves_to_device_memory_is_skipped_with_the_reason() {
    let keyboard = device(
        "keys",
        "Keyboard",
        104,
        vec![
            mode("Static", ModePersistence::AutomaticSave),
            mode("Off", ModePersistence::AutomaticSave),
        ],
    );
    let plan = plan_lights_off(&[keyboard]);
    assert!(plan.steps.is_empty());
    assert_eq!(
        plan.skipped,
        [skipped(
            Some("keys"),
            "Keyboard",
            LightsOffSkip::OffModeSavesToDevice
        )]
    );
}

#[test]
fn the_owner_keyboard_without_direct_or_off_mode_is_never_written() {
    let keyboard = device(
        "keys",
        "Keyboard",
        104,
        vec![
            mode("Static", ModePersistence::AutomaticSave),
            mode("Breathing", ModePersistence::AutomaticSave),
        ],
    );
    let plan = plan_lights_off(&[keyboard]);
    assert!(plan.steps.is_empty());
    assert_eq!(
        plan.skipped,
        [skipped(
            Some("keys"),
            "Keyboard",
            LightsOffSkip::NoDirectOrOffMode
        )]
    );
}

#[test]
fn unresolved_and_missing_devices_are_skipped() {
    let mut twin = device("a", "Fan", 8, vec![mode("Direct", ModePersistence::None)]);
    twin.identity = DeviceIdentity::Ambiguous {
        candidates: vec![DeviceRef::new("a"), DeviceRef::new("b")],
    };
    let mut gone = device("g", "Gone", 8, vec![mode("Direct", ModePersistence::None)]);
    gone.available = false;
    let plan = plan_lights_off(&[twin, gone]);
    assert!(plan.steps.is_empty());
    assert_eq!(
        plan.skipped,
        [
            skipped(None, "Fan", LightsOffSkip::NotResolved),
            skipped(Some("g"), "Gone", LightsOffSkip::NotAvailable),
        ]
    );
}
