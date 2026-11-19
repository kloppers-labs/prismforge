use std::io;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use prismforge_core::profile::{NameProblem, ProfileId};
use prismforge_core::session::SessionHandle;
use prismforge_core::store::{
    ConnectTrigger, FileOps, ProfileStore, Profiles, StdFileOps, StoreError,
};
use prismforge_core::{DeviceRef, Rgb};
use prismforge_effects::EffectInstance;
use prismforge_testkit::{DemoControls, DemoOutput, FakeServer};
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep};

use crate::session::{connected, demo, openrgb, reference, start, strip, whole};
use crate::support::{TempDir, profile};

const LIGHT_BAR: &str = "demo-light-bar";
const GPU: &str = "demo-graphics-card";
const GATE_LIMIT: Duration = Duration::from_secs(3);

async fn open(dir: &TempDir) -> Profiles {
    Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open store")
}

fn mode_calls(controls: &DemoControls) -> usize {
    controls
        .outputs()
        .iter()
        .filter(|output| matches!(output, DemoOutput::ModeReapplied { device, .. } if device.as_str() == LIGHT_BAR))
        .count()
}

/// What the desktop shell will run beside a session: apply the marked profile when the trigger says so.
fn watch_connects(profiles: Profiles, handle: SessionHandle) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut trigger = ConnectTrigger::default();
        let mut updates = handle.subscribe();
        loop {
            let snapshot = updates.borrow_and_update().clone();
            if trigger.due(&snapshot) {
                profiles.apply_marked(&handle).await.expect("apply marked");
            }
            if updates.changed().await.is_err() {
                return;
            }
        }
    })
}

#[tokio::test(start_paused = true)]
async fn apply_on_connect_is_off_by_default_and_applies_once_per_connect() {
    let dir = TempDir::new("on-connect");
    let profiles = open(&dir).await;
    let (setup, _) = demo().await;
    setup
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("breathing");
    let id = profiles
        .create_from_session(&setup, "Bar")
        .await
        .expect("capture")
        .id;
    drop(setup);

    let (unmarked, controls) = demo().await;
    let watcher = watch_connects(profiles.clone(), unmarked.clone());
    unmarked.refresh().await.expect("refresh");
    sleep(Duration::from_millis(500)).await;
    watcher.abort();
    assert_eq!(profiles.summary(None).await.apply_on_connect, None);
    assert_eq!(mode_calls(&controls), 0);
    drop(unmarked);

    profiles.set_apply_on_connect(Some(id)).await.expect("mark");
    let backend = prismforge_testkit::DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend);
    let watcher = watch_connects(profiles.clone(), handle.clone());
    handle.connect().await.expect("connect");
    for _ in 0..3 {
        handle.refresh().await.expect("refresh");
        sleep(Duration::from_millis(200)).await;
    }
    let after_first = mode_calls(&controls);
    handle.disconnect().await.expect("disconnect");
    sleep(Duration::from_millis(200)).await;
    handle.connect().await.expect("connect again");
    handle.refresh().await.expect("refresh");
    sleep(Duration::from_millis(500)).await;
    watcher.abort();

    assert_eq!(after_first, 1);
    assert_eq!(mode_calls(&controls), 2);
}

#[derive(Default)]
struct Gate {
    state: Mutex<(bool, bool)>,
    changed: Condvar,
}

/// Real file operations whose writes wait until the test opens the gate, or at most [`GATE_LIMIT`].
#[derive(Clone, Default)]
struct GatedOps(Arc<Gate>);

impl GatedOps {
    fn close(&self) {
        self.0.state.lock().expect("gate").0 = true;
    }

    fn entered(&self) -> bool {
        self.0.state.lock().expect("gate").1
    }

    fn open(&self) {
        self.0.state.lock().expect("gate").0 = false;
        self.0.changed.notify_all();
    }
}

impl FileOps for GatedOps {
    fn read(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        StdFileOps.read(path, limit)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut state = self.0.state.lock().expect("gate");
        if state.0 {
            state.1 = true;
            state = self
                .0
                .changed
                .wait_timeout_while(state, GATE_LIMIT, |state| state.0)
                .expect("gate")
                .0;
        }
        drop(state);
        StdFileOps.write(path, bytes)
    }

    fn sync(&self, path: &Path) -> io::Result<()> {
        StdFileOps.sync(path)
    }

    fn replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        StdFileOps.replace(from, to)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
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

fn gpu_frames(controls: &DemoControls) -> Vec<Instant> {
    controls
        .timed_outputs()
        .into_iter()
        .filter(|(_, output)| matches!(output, DemoOutput::DeviceFrame { device, .. } if device.as_str() == GPU))
        .map(|(at, _)| at)
        .collect()
}

#[tokio::test]
async fn file_io_does_not_stall_frame_output() {
    let dir = TempDir::new("stall");
    let ops = GatedOps::default();
    let profiles = Profiles::open(dir.path().to_path_buf(), ops.clone())
        .await
        .expect("open");
    let (handle, controls) = demo().await;
    let rainbow = EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    };
    handle
        .apply_effect(whole(&DeviceRef::new(GPU)), rainbow)
        .await
        .expect("apply");
    sleep(Duration::from_millis(200)).await;
    ops.close();

    let saving = {
        let (profiles, handle) = (profiles.clone(), handle.clone());
        tokio::spawn(async move { profiles.create_from_session(&handle, "Slow disk").await })
    };
    while !ops.entered() {
        sleep(Duration::from_millis(1)).await;
    }
    let before = gpu_frames(&controls).len();
    let mut command_latency = Duration::ZERO;
    for _ in 0..10 {
        let asked = Instant::now();
        handle.set_frame_rate(30).await.expect("command");
        command_latency = command_latency.max(asked.elapsed());
        sleep(Duration::from_millis(100)).await;
    }
    let during: Vec<Instant> = gpu_frames(&controls)[before..].to_vec();
    assert!(!saving.is_finished());
    ops.open();
    let saved = saving.await.expect("task").expect("saved");

    assert!(
        during.len() >= 20,
        "{} frames while the write waited",
        during.len()
    );
    let longest = during
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .max()
        .expect("several frames");
    assert!(longest <= Duration::from_millis(250), "{longest:?}");
    assert!(
        command_latency <= Duration::from_millis(250),
        "{command_latency:?}"
    );
    assert_eq!(
        profiles.profile(&saved.id).await.expect("stored").name,
        "Slow disk"
    );
}

#[tokio::test(start_paused = true)]
async fn registry_keeps_a_reference_across_an_app_restart_when_a_serial_device_moves() {
    let dir = TempDir::new("registry-restart");
    let server = FakeServer::new();
    server.add_device(0, strip("alpha", 3));
    let profiles = open(&dir).await;
    let handle = start(openrgb(&server, profiles.known_devices().await));
    let keeper = {
        let (profiles, known) = (profiles.clone(), handle.known_devices());
        tokio::spawn(async move { profiles.keep_registry(known).await })
    };
    handle.connect().await.expect("connect");
    let alpha = reference(&handle.snapshot(), "SN-alpha");
    drop(handle);
    keeper.await.expect("keeper ends with the session");
    drop(profiles);

    let moved = FakeServer::new();
    moved.add_device(0, strip("alpha", 3).location("usb:moved"));
    let unseeded = connected(&moved, Vec::new()).await;
    let profiles = open(&dir).await;
    let known = profiles.known_devices().await;
    let restarted = connected(&moved, known.clone()).await;

    assert_eq!(known.len(), 1);
    assert_eq!(known[0].reference, alpha);
    assert_ne!(reference(&unseeded.snapshot(), "SN-alpha"), alpha);
    assert_eq!(reference(&restarted.snapshot(), "SN-alpha"), alpha);
}

#[test]
fn store_operations_keep_names_distinct_and_the_marker_honest() {
    let dir = TempDir::new("crud");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let evening = store
        .create(profile("Evening", Vec::new()))
        .expect("create");
    let copy = store.duplicate(&evening).expect("duplicate");
    let other = store
        .create(profile("  Evening  ", Vec::new()))
        .expect("create");
    store
        .set_apply_on_connect(Some(copy.clone()))
        .expect("mark");

    assert_eq!(store.profile(&copy).expect("copy").name, "Evening (2)");
    assert_eq!(store.profile(&other).expect("other").name, "Evening (3)");
    assert_eq!(
        store.rename(&other, "Evening"),
        Err(StoreError::NameTaken {
            name: "Evening".into()
        })
    );
    assert_eq!(
        store.rename(&other, &"x".repeat(61)),
        Err(StoreError::InvalidName {
            problem: NameProblem::TooLong { max: 60 }
        })
    );
    assert_eq!(
        store.rename(&other, "Line\nbreak"),
        Err(StoreError::InvalidName {
            problem: NameProblem::ControlCharacter
        })
    );
    store.rename(&other, "Night").expect("rename");
    store.delete(&copy).expect("delete");
    let missing = ProfileId::parse("p-missing").expect("id");
    assert_eq!(
        store.delete(&missing),
        Err(StoreError::UnknownProfile { id: missing })
    );

    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    let names: Vec<String> = reopened
        .summary(None)
        .profiles
        .into_iter()
        .map(|summary| summary.name)
        .collect();
    let mut expected = vec!["Evening".to_owned(), "Night".to_owned()];
    expected.sort();
    let mut names = names;
    names.sort();
    assert_eq!(names, expected);
    assert_eq!(reopened.apply_on_connect(), None);
    assert!(!dir.files().iter().any(|name| name.contains(copy.as_str())));
}

#[tokio::test(start_paused = true)]
async fn friendly_names_are_kept_and_label_captured_devices() {
    let dir = TempDir::new("names");
    let profiles = open(&dir).await;
    let (handle, _) = demo().await;
    let gpu = DeviceRef::new(GPU);
    let unknown = profiles.set_device_name(&gpu, Some("Card".into())).await;
    profiles
        .record_known_devices(vec![prismforge_core::KnownDevice {
            reference: gpu.clone(),
            fingerprint: handle
                .snapshot()
                .devices
                .iter()
                .find(|device| device.identity.reference() == Some(&gpu))
                .expect("gpu")
                .descriptor
                .fingerprint(),
        }])
        .await
        .expect("record");
    profiles
        .set_device_name(&gpu, Some("  Big card  ".into()))
        .await
        .expect("name");
    profiles
        .set_zone_name(&gpu, 0, Some("Logo light".into()))
        .await
        .expect("zone");
    let wrong_zone = profiles.set_zone_name(&gpu, 4, Some("Nope".into())).await;
    handle
        .apply_effect(
            whole(&gpu),
            EffectInstance {
                effect: "solid".into(),
                values: [(
                    "color".to_owned(),
                    prismforge_effects::ControlValue::Color(Rgb::new(1, 1, 1)),
                )]
                .into(),
            },
        )
        .await
        .expect("apply");
    let id = profiles
        .create_from_session(&handle, "Named")
        .await
        .expect("capture")
        .id;
    drop(profiles);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");

    assert_eq!(
        unknown,
        Err(StoreError::UnknownDevice {
            device: gpu.clone()
        })
    );
    assert_eq!(
        wrong_zone,
        Err(StoreError::UnknownZone {
            device: gpu.clone(),
            zone: 4
        })
    );
    let entry = reopened.registry().device(&gpu).expect("registered");
    assert_eq!(entry.name.as_deref(), Some("Big card"));
    assert_eq!(entry.zone_name(0), Some("Logo light"));
    assert_eq!(
        reopened.profile(&id).expect("profile").devices[0].label,
        "Big card"
    );
}
