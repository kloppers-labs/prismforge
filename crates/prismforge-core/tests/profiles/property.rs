use std::collections::BTreeSet;

use prismforge_core::profile::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, ProfileId,
    StoredEffect, StoredIdentity,
};
use prismforge_core::session::{BatchReport, SessionHandle, SessionSnapshot, TargetOutcome};
use prismforge_core::setup::{
    Component, ComponentId, FirstLed, Geometry, OutputId, Placement, Ring, Setup, SetupOutput,
    Strip, Turn, View, ViewId, plan_across,
};
use prismforge_core::store::{ApplyOutcome, Profiles, StdFileOps};
use prismforge_core::{ApplyMode, DeviceRef, OutputOwner, OutputTarget, Rgb};
use prismforge_effects::{ControlValue, EffectInstance, Flow};
use serde_json::Map;

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

fn strip(leds: u32) -> Geometry {
    Geometry::Strip(Strip {
        leds,
        first: FirstLed::AtStart,
        extra: Map::new(),
    })
}

fn ring(leds: u32) -> Geometry {
    Geometry::Ring(Ring {
        leds,
        first_angle: 0,
        turn: Turn::Clockwise,
        extra: Map::new(),
    })
}

/// An output id, its device, its zone and its branches of component ids.
type Wiring = (
    &'static str,
    &'static str,
    Option<usize>,
    &'static [&'static [&'static str]],
);

/// The demo devices as a setup: a chain on header 1, a mirrored pair on header 2, and two views.
fn setup() -> Setup {
    let components = [
        ("c-onboard", strip(6)),
        ("c-fan-1", ring(12)),
        ("c-fan-2", ring(12)),
        ("c-fan-3", ring(8)),
        ("c-fan-4", ring(8)),
        ("c-card", ring(12)),
        ("c-ram", strip(8)),
        ("c-bar", strip(10)),
    ];
    let wiring: [Wiring; 6] = [
        ("o-onboard", "demo-motherboard", Some(0), &[&["c-onboard"]]),
        (
            "o-chain",
            "demo-motherboard",
            Some(1),
            &[&["c-fan-1", "c-fan-2"]],
        ),
        (
            "o-hub",
            "demo-motherboard",
            Some(2),
            &[&["c-fan-3"], &["c-fan-4"]],
        ),
        ("o-card", "demo-graphics-card", None, &[&["c-card"]]),
        ("o-ram", "demo-memory-1", None, &[&["c-ram"]]),
        ("o-bar", "demo-light-bar", None, &[&["c-bar"]]),
    ];
    let zone_names = ["Onboard", "Addressable Header 1", "Addressable Header 2"];
    let id = |value: &str| ComponentId::parse(value).expect("component id");
    let placements = |ids: &[&str]| -> Vec<Placement> {
        ids.iter()
            .enumerate()
            .map(|(index, component)| Placement {
                component: id(component),
                x: 30 * i32::try_from(index).expect("small"),
                y: 10 * i32::try_from(index % 3).expect("small"),
                size: 24,
                rotation: u16::try_from(index * 45 % 360).expect("small"),
                flipped: index % 2 == 1,
                extra: Map::new(),
            })
            .collect()
    };
    let all: Vec<&str> = components.iter().map(|(name, _)| *name).collect();
    Setup {
        components: components
            .into_iter()
            .map(|(name, geometry)| Component {
                id: id(name),
                name: name.trim_start_matches("c-").into(),
                geometry,
                frame: None,
                extra: Map::new(),
            })
            .collect(),
        outputs: wiring
            .into_iter()
            .map(|(output, device, zone, branches)| SetupOutput {
                id: OutputId::parse(output).expect("output id"),
                device: DeviceRef::new(device),
                zone,
                label: device.into(),
                zone_name: zone.map(|zone| zone_names[zone].to_owned()),
                first_led: 0,
                branches: branches
                    .iter()
                    .map(|branch| branch.iter().map(|component| id(component)).collect())
                    .collect(),
                extra: Map::new(),
            })
            .collect(),
        views: vec![
            View {
                id: ViewId::parse("v-pc").expect("view id"),
                name: "PC".into(),
                placements: placements(&all),
                outlines: Vec::new(),
                extra: Map::new(),
            },
            View {
                id: ViewId::parse("v-desk").expect("view id"),
                name: "Desk".into(),
                placements: placements(&["c-card", "c-ram", "c-bar", "c-fan-1"]),
                outlines: Vec::new(),
                extra: Map::new(),
            },
        ],
        extra: Map::new(),
    }
}

/// Some of the targets of a random view, as the setup places them now.
fn across_targets(handle: &SessionHandle, rng: &mut Rng) -> prismforge_core::setup::AcrossPlan {
    let view = ViewId::parse(["v-pc", "v-desk"][rng.below(2)]).expect("view id");
    let flow = match rng.below(3) {
        0 => Flow::FromCentre {
            x: rng.below(101) as u8,
            y: rng.below(101) as u8,
        },
        _ => Flow::Along {
            angle: 45 * rng.below(8) as u16,
        },
    };
    let plan = plan_across(&setup(), &view, flow, &handle.snapshot().devices).expect("view");
    let mut targets = Vec::new();
    for placed in &plan.targets {
        if rng.below(3) != 0 {
            targets.push(placed.target.clone());
        }
    }
    plan.only(&targets)
}

enum Touched {
    Device(DeviceRef),
    FrameRate,
}

fn random_target(rng: &mut Rng) -> OutputTarget {
    let (name, zones) = DEVICES[rng.below(DEVICES.len() + 3).saturating_sub(3)];
    OutputTarget {
        device: DeviceRef::new(name),
        zone: (rng.below(3) == 0).then(|| rng.below(zones)),
    }
}

/// The devices of a batch's targets that the batch changed.
fn changed(report: &BatchReport) -> Vec<Touched> {
    report
        .results
        .iter()
        .filter(|result| {
            matches!(
                result.outcome,
                TargetOutcome::Applied | TargetOutcome::Stopped { .. }
            )
        })
        .map(|result| Touched::Device(result.target.device.clone()))
        .collect()
}

/// One random user command, single or batch; empty when the session changed nothing.
async fn random_command(handle: &SessionHandle, rng: &mut Rng) -> Vec<Touched> {
    let (name, zones) = DEVICES[rng.below(DEVICES.len() + 3).saturating_sub(3)];
    let device = DeviceRef::new(name);
    let target = |rng: &mut Rng, whole: bool| OutputTarget {
        device: device.clone(),
        zone: (!whole).then(|| rng.below(zones)),
    };
    let batch = |rng: &mut Rng| {
        let count = 1 + rng.below(4);
        (0..count).map(|_| random_target(rng)).collect::<Vec<_>>()
    };
    let done = match rng.below(20) {
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
        12 | 13 => {
            let targets = batch(rng);
            let effect = solid(rng);
            let report = handle.apply_batch(targets, effect).await.expect("batch");
            return changed(&report);
        }
        14 => {
            let report = handle.stop_batch(batch(rng)).await.expect("stop batch");
            return changed(&report);
        }
        15 => {
            let report = handle.stop_all_effects().await.expect("stop all");
            return changed(&report);
        }
        17 | 18 => {
            let plan = across_targets(handle, rng);
            let effect = solid(rng);
            let report = handle.apply_across(plan, effect).await.expect("across");
            return changed(&report);
        }
        19 => {
            let targets = across_targets(handle, rng)
                .targets
                .into_iter()
                .map(|placed| placed.target)
                .collect();
            let report = handle.stop_batch(targets).await.expect("stop across");
            return changed(&report);
        }
        _ => {
            let rate = 10 + rng.below(51) as u32;
            return handle
                .set_frame_rate(rate)
                .await
                .ok()
                .map(|_| Touched::FrameRate)
                .into_iter()
                .collect();
        }
    };
    if done {
        vec![Touched::Device(device)]
    } else {
        Vec::new()
    }
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
                    mode: assignment.mode.clone(),
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
    let mut across_saved = 0;
    for seed in 1..=SEEDS {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ seed);
        let dir = TempDir::new("property");
        let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
            .await
            .expect("open");
        let (handle, _) = demo().await;
        profiles
            .save_setup(setup(), Some(&handle))
            .await
            .expect("setup");
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
            for touch in random_command(&handle, &mut rng).await {
                match touch {
                    Touched::Device(device) => {
                        touched.insert(device);
                    }
                    Touched::FrameRate => rate = true,
                }
            }
        }
        let at = format!("seed {seed} end");
        let last = check_update(&profiles, &handle, &id, &stored, &touched, rate, &at).await;
        if last
            .assignments
            .iter()
            .any(|assignment| assignment.mode != ApplyMode::EachDevice)
        {
            across_saved += 1;
        }
    }
    assert!(
        across_saved >= SEEDS / 4,
        "effects across the setup reach a saved profile in {across_saved} of {SEEDS} runs"
    );
}
