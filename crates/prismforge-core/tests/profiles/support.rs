use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use prismforge_core::profile::{
    ExternalControl, Extra, ModeSelection, Profile, ProfileAssignment, ProfileDevice, StoredEffect,
    StoredIdentity, StoredZone,
};
use prismforge_core::store::{FileOps, StdFileOps};
use prismforge_core::{ApplyMode, DeviceKind, DeviceRef, Rgb, ZoneKind};
use prismforge_effects::{ControlValue, Direction, EffectInstance};
use serde_json::{Value, json};

/// A fresh folder under the system temporary folder, removed when dropped.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            env::temp_dir().join(format!("prismforge-test-{}-{label}-{count}", process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temporary folder");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn file(&self, name: &str) -> PathBuf {
        name.split('/')
            .fold(self.0.clone(), |path, part| path.join(part))
    }

    pub fn read(&self, name: &str) -> Vec<u8> {
        fs::read(self.file(name)).unwrap_or_else(|error| panic!("read {name}: {error}"))
    }

    pub fn write(&self, name: &str, bytes: &[u8]) {
        let path = self.file(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("folder");
        fs::write(path, bytes).expect("write fixture file");
    }

    /// Every file name below the folder, relative and with `/`.
    pub fn files(&self) -> Vec<String> {
        let mut names = Vec::new();
        collect(&self.0, "", &mut names);
        names.sort();
        names
    }
}

fn collect(dir: &Path, prefix: &str, names: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("list") {
        let entry = entry.expect("entry");
        let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if entry.file_type().expect("type").is_dir() {
            collect(&entry.path(), &format!("{name}/"), names);
        } else {
            names.push(name);
        }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Write,
    Sync,
    Replace,
    Remove,
}

#[derive(Clone)]
struct Fault {
    step: Step,
    suffix: String,
}

/// Real file operations that fail at chosen steps; a failed write leaves half the bytes behind.
#[derive(Clone, Default)]
pub struct FaultyOps {
    faults: Arc<Mutex<Vec<Fault>>>,
}

impl FaultyOps {
    pub fn fail(&self, step: Step, path_suffix: &str) {
        self.faults.lock().expect("faults").push(Fault {
            step,
            suffix: path_suffix.into(),
        });
    }

    pub fn heal(&self) {
        self.faults.lock().expect("faults").clear();
    }

    fn check(&self, step: Step, path: &Path) -> io::Result<()> {
        let path = path.to_string_lossy().replace('\\', "/");
        let failing = self
            .faults
            .lock()
            .expect("faults")
            .iter()
            .any(|fault| fault.step == step && path.ends_with(&fault.suffix));
        if failing {
            Err(io::Error::other(format!("injected {step:?} failure")))
        } else {
            Ok(())
        }
    }
}

impl FileOps for FaultyOps {
    fn read(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        StdFileOps.read(path, limit)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        if let Err(error) = self.check(Step::Write, path) {
            StdFileOps.write(path, &bytes[..bytes.len() / 2])?;
            return Err(error);
        }
        StdFileOps.write(path, bytes)
    }

    fn sync(&self, path: &Path) -> io::Result<()> {
        self.check(Step::Sync, path)?;
        StdFileOps.sync(path)
    }

    fn replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.check(Step::Replace, to)?;
        StdFileOps.replace(from, to)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        self.check(Step::Remove, path)?;
        StdFileOps.remove(path)
    }

    fn exists(&self, path: &Path) -> bool {
        StdFileOps.exists(path)
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        StdFileOps.list(dir)
    }

    fn create_dir_all(&self, dir: &Path) -> io::Result<()> {
        StdFileOps.create_dir_all(dir)
    }
}

pub fn golden(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("profiles")
        .join("golden")
        .join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|error| panic!("golden {name}: {error}"));
    text.replace("\r\n", "\n").into_bytes()
}

pub fn extra(pairs: &[(&str, Value)]) -> Extra {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect()
}

pub fn zone(name: &str, kind: ZoneKind, leds: u32) -> StoredZone {
    StoredZone {
        name: name.into(),
        kind,
        leds_min: leds,
        leds_max: leds,
        extra: Extra::new(),
    }
}

pub fn device(
    reference: &str,
    model: &str,
    serial: Option<&str>,
    location: Option<&str>,
    zones: Vec<StoredZone>,
) -> ProfileDevice {
    ProfileDevice {
        reference: DeviceRef::new(reference),
        label: model.into(),
        identity: StoredIdentity {
            kind: DeviceKind::LedStrip,
            vendor: "Fixture Vendor".into(),
            model: model.into(),
            serial: serial.map(Into::into),
            location: location.map(Into::into),
            zones,
        },
        extra: Extra::new(),
    }
}

pub fn effect(id: &str, values: &[(&str, ControlValue)]) -> StoredEffect {
    StoredEffect::from_instance(&EffectInstance {
        effect: id.into(),
        values: values
            .iter()
            .map(|(control, value)| ((*control).to_owned(), value.clone()))
            .collect(),
    })
}

pub fn assignment(device: &str, zone: Option<usize>, effect: StoredEffect) -> ProfileAssignment {
    ProfileAssignment {
        device: DeviceRef::new(device),
        zone,
        effect,
        mode: ApplyMode::EachDevice,
        extra: Extra::new(),
    }
}

pub fn profile(name: &str, devices: Vec<ProfileDevice>) -> Profile {
    Profile {
        name: name.into(),
        frame_rate: 30,
        devices,
        assignments: Vec::new(),
        modes: Vec::new(),
        external: Vec::new(),
        extra: Extra::new(),
    }
}

/// Every kind of stored content, including fields a newer minor version added.
pub fn full_profile() -> Profile {
    let mut desk = device(
        "dev-desk",
        "Strip desk",
        Some("SN-desk"),
        Some("usb:desk"),
        vec![
            zone("Front", ZoneKind::Linear, 8),
            zone("Back", ZoneKind::LinearLoop, 4),
        ],
    );
    desk.label = "Desk".into();
    desk.identity.kind = DeviceKind::Motherboard;
    desk.extra = extra(&[("future_device_field", json!("kept"))]);
    desk.identity.zones[1].extra = extra(&[("future_zone_field", json!([1, 2]))]);
    let mut panel = device(
        "dev-panel",
        "Panel",
        None,
        Some("hid:panel"),
        vec![
            zone("Grid", ZoneKind::Matrix, 16),
            zone("Edge", ZoneKind::Other(9), 2),
        ],
    );
    panel.identity.kind = DeviceKind::Keyboard;
    let away = device(
        "dev-away",
        "Strip away",
        Some("SN-away"),
        Some("usb:away"),
        vec![zone("Strip", ZoneKind::Linear, 5)],
    );
    let card = device(
        "dev-card",
        "Graphics",
        Some("SN-card"),
        None,
        vec![zone("Logo", ZoneKind::Single, 1)],
    );
    let plugin = device(
        "dev-plugin",
        "Plugin strip",
        None,
        None,
        vec![zone("Strip", ZoneKind::Linear, 3)],
    );
    let lamp = device(
        "dev-lamp",
        "Lamp",
        Some("SN-lamp"),
        Some("usb:lamp"),
        vec![zone("Bar", ZoneKind::Segmented, 10)],
    );
    let mut wave = assignment(
        "dev-desk",
        Some(0),
        effect(
            "rainbow-wave",
            &[
                ("brightness", ControlValue::Number(80)),
                ("direction", ControlValue::Direction(Direction::Backward)),
                ("speed", ControlValue::Number(12)),
            ],
        ),
    );
    wave.extra = extra(&[("future_assignment_field", json!({"a": 1, "b": null}))]);
    let mut gradient = effect(
        "gradient",
        &[
            ("brightness", ControlValue::Number(100)),
            ("direction", ControlValue::Direction(Direction::Forward)),
            (
                "palette",
                ControlValue::Palette(vec![Rgb::new(255, 0, 0), Rgb::new(0, 0, 255)]),
            ),
        ],
    );
    gradient.extra = extra(&[("future_effect_field", json!(7))]);
    let mut highlight = effect(
        "moving-highlight",
        &[
            ("background", ControlValue::Color(Rgb::new(0, 0, 0))),
            ("color", ControlValue::Color(Rgb::new(255, 255, 255))),
            ("width", ControlValue::Number(25)),
        ],
    );
    highlight
        .values
        .insert("future_control".into(), json!({"Gradient": [1, 2]}));
    Profile {
        name: "Evening".into(),
        frame_rate: 45,
        devices: vec![desk, panel, away, card, plugin, lamp],
        assignments: vec![
            wave,
            assignment("dev-desk", Some(1), gradient),
            assignment(
                "dev-panel",
                None,
                effect(
                    "solid",
                    &[("color", ControlValue::Color(Rgb::new(12, 34, 56)))],
                ),
            ),
            assignment("dev-away", None, highlight),
            assignment(
                "dev-card",
                Some(0),
                effect("fire", &[("heat", ControlValue::Number(3))]),
            ),
        ],
        modes: vec![ModeSelection {
            device: DeviceRef::new("dev-lamp"),
            index: 2,
            name: "Breathing".into(),
            extra: extra(&[("future_mode_field", json!(true))]),
        }],
        external: vec![ExternalControl {
            device: DeviceRef::new("dev-plugin"),
            label: "OpenRGB Effects plugin".into(),
            extra: extra(&[("future_external_field", json!("x"))]),
        }],
        extra: extra(&[("future_profile_field", json!({"layout": [1, 2, 3]}))]),
    }
}
