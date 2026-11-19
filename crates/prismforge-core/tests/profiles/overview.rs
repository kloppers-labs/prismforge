use prismforge_core::profile::{EffectProblem, ProfileItem, SkipReason};
use prismforge_core::store::{
    DeviceNames, NoticeEvent, Profiles, Readiness, StdFileOps, StoredSetting, ZoneLabel,
};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::EffectInstance;

use crate::session::demo;
use crate::support::TempDir;

const GPU: &str = "demo-graphics-card";
const MEMORY_1: &str = "demo-memory-1";
const MEMORY_2: &str = "demo-memory-2";
const MOTHERBOARD: &str = "demo-motherboard";
const LIGHT_BAR: &str = "demo-light-bar";

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

async fn open(dir: &TempDir) -> Profiles {
    Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open store")
}

fn readiness_of(settings: &[StoredSetting], device: &str) -> Readiness {
    settings
        .iter()
        .find(|setting| setting.device.as_str() == device)
        .unwrap_or_else(|| panic!("no setting for {device}: {settings:?}"))
        .today
        .clone()
}

fn skipped(reason: SkipReason) -> Readiness {
    Readiness::Skipped { reason }
}

#[tokio::test(start_paused = true)]
async fn the_overview_counts_settings_and_checks_each_one_against_today() {
    let dir = TempDir::new("overview");
    let profiles = open(&dir).await;
    let (setup, _) = demo().await;
    for device in [GPU, MEMORY_1] {
        setup
            .apply_effect(whole(device), rainbow())
            .await
            .expect("effect");
    }
    setup
        .select_mode(DeviceRef::new(MOTHERBOARD), 2)
        .await
        .expect("rainbow wave mode");
    setup
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("breathing mode");
    setup
        .declare_external(DeviceRef::new(MEMORY_2), "Plugin".into())
        .await
        .expect("declare");
    let id = profiles
        .create_from_session(&setup, "Evening")
        .await
        .expect("capture")
        .id;
    drop(setup);
    let mut stored = profiles.profile(&id).await.expect("stored");
    stored
        .modes
        .retain(|mode| mode.device.as_str() != LIGHT_BAR);
    for mode in &mut stored.modes {
        mode.name = "Old Wave".into();
    }
    let mut on_bar = stored.assignments[0].clone();
    on_bar.device = DeviceRef::new(LIGHT_BAR);
    stored.assignments.push(on_bar);
    for assignment in &mut stored.assignments {
        if assignment.device.as_str() == GPU {
            assignment.effect.effect = "aurora".into();
            assignment.zone = Some(0);
        }
    }
    profiles.replace(&id, stored).await.expect("replace");

    let (today, _) = demo().await;
    today
        .declare_external(DeviceRef::new(MEMORY_1), "Other app".into())
        .await
        .expect("declare");
    let summary = profiles.summary(Some(&today)).await;
    let details = profiles.details(&id, Some(&today)).await.expect("details");
    let offline = profiles.details(&id, None).await.expect("offline details");

    let listed = &summary.profiles[0];
    assert_eq!((listed.effects, listed.modes, listed.external), (3, 1, 1));
    assert!(details.checked);
    assert!(!offline.checked);
    let settings = &details.settings;
    assert_eq!(settings.len(), 5);
    assert_eq!(readiness_of(settings, MEMORY_2), Readiness::Ready);
    assert_eq!(
        readiness_of(settings, MOTHERBOARD),
        skipped(SkipReason::ModeListChanged {
            found: Some("Rainbow Wave".into())
        })
    );
    assert_eq!(
        readiness_of(settings, GPU),
        skipped(SkipReason::NotAvailable {
            problem: EffectProblem::EffectNotAvailable {
                effect: "aurora".into()
            }
        })
    );
    assert_eq!(
        readiness_of(settings, MEMORY_1),
        skipped(SkipReason::DeclaredExternal {
            label: "Other app".into()
        })
    );
    assert_eq!(
        readiness_of(settings, LIGHT_BAR),
        skipped(SkipReason::NoDirectMode)
    );
    let gpu = settings
        .iter()
        .find(|setting| setting.device.as_str() == GPU)
        .expect("gpu");
    assert_eq!(gpu.zone_name.as_deref(), Some("Logo"));
    assert_eq!(gpu.label, "Demo Graphics Card");
    assert_eq!(
        gpu.item,
        ProfileItem::Effect {
            zone: Some(0),
            effect: "aurora".into()
        }
    );
    assert_eq!(gpu.effect_label, None);
    let memory = settings
        .iter()
        .find(|setting| setting.device.as_str() == MEMORY_1)
        .expect("memory");
    assert_eq!(memory.effect_label.as_deref(), Some("Rainbow wave"));
}

#[tokio::test(start_paused = true)]
async fn names_are_listed_set_and_cleared_in_the_registry() {
    let dir = TempDir::new("names");
    let profiles = open(&dir).await;
    let (handle, _) = demo().await;
    let known = handle.known_devices().borrow().clone();
    profiles.record_known_devices(known).await.expect("record");

    profiles
        .set_device_name(&DeviceRef::new(GPU), Some("Desk card".into()))
        .await
        .expect("device name");
    profiles
        .set_zone_name(&DeviceRef::new(MOTHERBOARD), 1, Some("Front fans".into()))
        .await
        .expect("zone name");
    let named = profiles.names().await;
    profiles
        .set_device_name(&DeviceRef::new(GPU), None)
        .await
        .expect("clear");
    let cleared = profiles.names().await;

    assert_eq!(
        named,
        vec![
            DeviceNames {
                device: DeviceRef::new(MOTHERBOARD),
                name: None,
                zones: vec![ZoneLabel {
                    zone: 1,
                    name: "Front fans".into()
                }],
            },
            DeviceNames {
                device: DeviceRef::new(GPU),
                name: Some("Desk card".into()),
                zones: Vec::new(),
            },
        ]
    );
    assert_eq!(cleared.len(), 1);
    assert_eq!(cleared[0].device.as_str(), MOTHERBOARD);
}

#[tokio::test]
async fn a_dismissed_notice_stays_away_and_an_unknown_one_changes_nothing() {
    let dir = TempDir::new("dismiss");
    dir.write("devices.json", b"not json");
    let profiles = open(&dir).await;
    let notices = profiles.summary(None).await.notices;
    let notice = notices.first().cloned().expect("a notice");
    assert!(matches!(notice.event, NoticeEvent::StartedEmpty { .. }));

    let first = profiles.dismiss_notice(notice.clone()).await;
    let again = profiles.dismiss_notice(notice).await;

    assert!(first);
    assert!(!again);
    assert!(profiles.summary(None).await.notices.is_empty());
}
