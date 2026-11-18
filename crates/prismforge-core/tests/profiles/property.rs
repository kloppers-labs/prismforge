use std::collections::BTreeSet;

use prismforge_core::profile::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, ProfileId,
    StoredEffect, StoredIdentity,
};
use prismforge_core::session::{SessionHandle, SessionSnapshot};
use prismforge_core::store::{ApplyOutcome, Profiles, StdFileOps};
use prismforge_core::{DeviceRef, OutputOwner, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, EffectInstance};

use crate::session::demo;
use crate::support::TempDir;

const SEEDS: u64 = 40;
const STEPS: usize = 64;
const DEVICES: [(&str, usize); 4] = [
    ("demo-motherboard", 3),
    ("demo-graphics-card", 1),
    ("demo-memory-1", 1),
    ("demo-light-bar", 1),
];

struct Rng(u64);

impl Rng {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}

fn solid(rng: &mut Rng) -> EffectInstance {
    let channel = |rng: &mut Rng| rng.below(256) as u8;
    let color = Rgb::new(channel(rng), channel(rng), channel(rng));
    EffectInstance {
        effect: "solid".into(),
        values: [("color".to_owned(), ControlValue::Color(color))].into(),
    }
}

enum Touched {
    Device(DeviceRef),
    FrameRate,
}

/// One random user command; `None` when the session refused it.
async fn random_command(handle: &SessionHandle, rng: &mut Rng) -> Option<Touched> {
    let (name, zones) = DEVICES[rng.below(DEVICES.len() + 3).saturating_sub(3)];
    let device = DeviceRef::new(name);
    let target = |rng: &mut Rng, whole: bool| OutputTarget {
        device: device.clone(),
        zone: (!whole).then(|| rng.below(zones)),
    };
    let done = match rng.below(13) {
        0..=2 => {
            let target = target(rng, false);
            handle.apply_effect(target, solid(rng)).await.is_ok()
        }
        3 | 4 => {
            let target = target(rng, true);
            handle.apply_effect(target, solid(rng)).await.is_ok()
        }
        5..=7 => handle.stop_effect(target(rng, false)).await.is_ok(),
        8 => handle.stop_effect(target(rng, true)).await.is_ok(),
        9 => handle
            .select_mode(device.clone(), rng.below(3))
            .await
            .is_ok(),
        10 => {
            let label = format!("Program {}", rng.below(3));
            handle.declare_external(device.clone(), label).await.is_ok()
        }
        11 => handle.clear_external(device.clone()).await.is_ok(),
        _ => {
            let rate = 10 + rng.below(51) as u32;
            return handle
                .set_frame_rate(rate)
                .await
                .ok()
                .map(|_| Touched::FrameRate);
        }
    };
    done.then_some(Touched::Device(device))
}

/// The stored profile with each touched device's settings replaced by what the session holds for it.
fn expected(
    stored: &Profile,
    snapshot: &SessionSnapshot,
    touched: &BTreeSet<DeviceRef>,
    rate: bool,
) -> Profile {
    let mut profile = stored.clone();
    if rate {
        profile.frame_rate = snapshot.output.frame_rate;
    }
    for device in touched {
        profile
            .assignments
            .retain(|assignment| &assignment.device != device);
        profile.modes.retain(|mode| &mode.device != device);
        profile
            .external
            .retain(|external| &external.device != device);
        let descriptor = &snapshot
            .devices
            .iter()
            .find(|entry| entry.identity.reference() == Some(device))
            .expect("demo device")
            .descriptor;
        let mut held = false;
        for assignment in &snapshot.output.scene.assignments {
            if &assignment.target.device == device {
                held = true;
                profile.assignments.push(ProfileAssignment {
                    device: device.clone(),
                    zone: assignment.target.zone,
                    effect: StoredEffect::from_instance(&assignment.effect),
                    extra: Extra::new(),
                });
            }
        }
        for output in snapshot
            .output
            .devices
            .iter()
            .filter(|output| &output.device == device)
        {
            match &output.owner {
                OutputOwner::HardwareMode { mode } => {
                    held = true;
                    profile.modes.push(ModeSelection {
                        device: device.clone(),
                        index: *mode,
                        name: descriptor.capabilities.modes[*mode].name.clone(),
                        extra: Extra::new(),
                    });
                }
                OutputOwner::External { label } => {
                    held = true;
                    profile.external.push(ExternalControl {
                        device: device.clone(),
                        label: label.clone(),
                        extra: Extra::new(),
                    });
                }
                OutputOwner::Effect => {}
            }
        }
        if held && profile.device(device).is_none() {
            profile.devices.push(ProfileDevice {
                reference: device.clone(),
                label: descriptor.name.clone(),
                identity: StoredIdentity::from_fingerprint(&descriptor.fingerprint()),
                extra: Extra::new(),
            });
        }
    }
    profile
}

fn normalized(mut profile: Profile) -> Profile {
    profile
        .devices
        .sort_by(|left, right| left.reference.cmp(&right.reference));
    profile
        .assignments
        .sort_by(|left, right| (&left.device, left.zone).cmp(&(&right.device, right.zone)));
    profile
        .modes
        .sort_by(|left, right| left.device.cmp(&right.device));
    profile
        .external
        .sort_by(|left, right| left.device.cmp(&right.device));
    profile
}

async fn check_update(
    profiles: &Profiles,
    handle: &SessionHandle,
    id: &ProfileId,
    stored: &Profile,
    touched: &BTreeSet<DeviceRef>,
    rate: bool,
    at: &str,
) -> Profile {
    let unsaved = profiles.summary(Some(handle)).await.unsaved_changes;
    let preview = profiles.preview_update(handle, id).await.expect("preview");
    let no_changes = preview.changes.is_empty() && preview.frame_rate.is_none();
    assert_eq!(
        unsaved == Some(0),
        no_changes,
        "{at}: {unsaved:?} {preview:?}"
    );
    let saved = profiles
        .update_from_session(handle, id)
        .await
        .expect("update");
    assert_eq!(saved, preview, "{at}");
    let after = profiles.profile(id).await.expect("saved");
    let wanted = expected(stored, &handle.snapshot(), touched, rate);
    assert_eq!(normalized(after.clone()), normalized(wanted), "{at}");
    after
}

#[tokio::test(start_paused = true)]
async fn an_update_holds_exactly_what_the_session_holds_for_every_touched_device() {
    for seed in 1..=SEEDS {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ seed);
        let dir = TempDir::new("property");
        let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
            .await
            .expect("open");
        let (handle, _) = demo().await;
        for _ in 0..6 {
            random_command(&handle, &mut rng).await;
        }
        let id = profiles
            .create_from_session(&handle, "Start")
            .await
            .expect("create")
            .id;
        match profiles.apply(&handle, &id).await.expect("apply") {
            ApplyOutcome::Applied(_) => {}
            other => panic!("{other:?}"),
        }
        let mut stored = profiles.profile(&id).await.expect("stored");
        let mut touched = BTreeSet::new();
        let mut rate = false;
        for step in 0..STEPS {
            if rng.below(8) == 0 {
                let at = format!("seed {seed} step {step}");
                stored = check_update(&profiles, &handle, &id, &stored, &touched, rate, &at).await;
                touched.clear();
                rate = false;
                continue;
            }
            match random_command(&handle, &mut rng).await {
                Some(Touched::Device(device)) => {
                    touched.insert(device);
                }
                Some(Touched::FrameRate) => rate = true,
                None => {}
            }
        }
        let at = format!("seed {seed} end");
        check_update(&profiles, &handle, &id, &stored, &touched, rate, &at).await;
    }
}
