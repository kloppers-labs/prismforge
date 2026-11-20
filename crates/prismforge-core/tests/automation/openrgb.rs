use prismforge_core::automation::{
    EventProfile, EventProfiles, OpenRgbAct, OpenRgbSetting, Trigger,
};

fn enabled(name: &str) -> EventProfile {
    EventProfile {
        enabled: true,
        name: name.into(),
    }
}

fn every_setting_on() -> EventProfiles {
    EventProfiles {
        open: enabled("Open"),
        exit: enabled("Exit"),
        resume: enabled("Resume"),
        suspend: enabled("Suspend"),
        service_startup: enabled("Service start"),
        service_shutdown: enabled("Service stop"),
    }
}

fn act(setting: OpenRgbSetting, profile: &str) -> OpenRgbAct {
    OpenRgbAct {
        setting,
        profile: profile.into(),
    }
}

#[test]
fn each_event_maps_to_the_openrgb_settings_that_act_on_it() {
    let profiles = every_setting_on();
    assert_eq!(
        profiles.acting_on(Trigger::Sleeps),
        [act(OpenRgbSetting::Suspend, "Suspend")]
    );
    assert_eq!(
        profiles.acting_on(Trigger::Wakes),
        [act(OpenRgbSetting::Resume, "Resume")]
    );
    let closing = [
        act(OpenRgbSetting::Exit, "Exit"),
        act(OpenRgbSetting::ServiceShutdown, "Service stop"),
    ];
    assert_eq!(profiles.acting_on(Trigger::SessionEnds), closing);
    assert_eq!(profiles.acting_on(Trigger::Quits), closing);
    assert_eq!(
        profiles.acting_on(Trigger::Starts),
        [
            act(OpenRgbSetting::Open, "Open"),
            act(OpenRgbSetting::ServiceStartup, "Service start"),
        ]
    );
    assert!(profiles.acting_on(Trigger::Connects).is_empty());
}

#[test]
fn a_setting_acts_only_when_it_is_on_and_names_a_profile() {
    let mut profiles = EventProfiles::default();
    assert!(profiles.acting_on(Trigger::Sleeps).is_empty());
    profiles.suspend = EventProfile {
        enabled: true,
        name: "  ".into(),
    };
    assert!(profiles.acting_on(Trigger::Sleeps).is_empty());
    profiles.suspend = EventProfile {
        enabled: false,
        name: "Dark".into(),
    };
    assert!(profiles.acting_on(Trigger::Sleeps).is_empty());
}
