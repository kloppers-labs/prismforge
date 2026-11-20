use serde::{Deserialize, Serialize};

use crate::automation::Trigger;

/// OpenRGB's own "load profile" settings for events, as its settings manager reports them.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EventProfiles {
    pub open: EventProfile,
    pub exit: EventProfile,
    pub resume: EventProfile,
    pub suspend: EventProfile,
    pub service_startup: EventProfile,
    pub service_shutdown: EventProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EventProfile {
    pub enabled: bool,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum OpenRgbSetting {
    Open,
    Exit,
    Resume,
    Suspend,
    ServiceStartup,
    ServiceShutdown,
}

/// OpenRGB will also load `profile` around this event, so two programs write and the last one wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct OpenRgbAct {
    pub setting: OpenRgbSetting,
    pub profile: String,
}

impl EventProfiles {
    /// The settings that act on the same event; OpenRGB loads a profile only when it is on and named.
    pub fn acting_on(&self, trigger: Trigger) -> Vec<OpenRgbAct> {
        let settings: &[OpenRgbSetting] = match trigger {
            Trigger::Starts => &[OpenRgbSetting::Open, OpenRgbSetting::ServiceStartup],
            Trigger::Connects => &[],
            Trigger::Sleeps => &[OpenRgbSetting::Suspend],
            Trigger::Wakes => &[OpenRgbSetting::Resume],
            Trigger::SessionEnds | Trigger::Quits => {
                &[OpenRgbSetting::Exit, OpenRgbSetting::ServiceShutdown]
            }
        };
        settings
            .iter()
            .filter_map(|setting| {
                let profile = self.profile(*setting);
                let name = profile.name.trim();
                (profile.enabled && !name.is_empty()).then(|| OpenRgbAct {
                    setting: *setting,
                    profile: name.to_owned(),
                })
            })
            .collect()
    }

    fn profile(&self, setting: OpenRgbSetting) -> &EventProfile {
        match setting {
            OpenRgbSetting::Open => &self.open,
            OpenRgbSetting::Exit => &self.exit,
            OpenRgbSetting::Resume => &self.resume,
            OpenRgbSetting::Suspend => &self.suspend,
            OpenRgbSetting::ServiceStartup => &self.service_startup,
            OpenRgbSetting::ServiceShutdown => &self.service_shutdown,
        }
    }
}
