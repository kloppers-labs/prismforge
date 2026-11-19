use std::fs;
use std::time::Duration;

use prismforge_core::profile::ProfileId;
use prismforge_core::session::SessionConfig;
use prismforge_core::{DeviceRef, OutputTarget, Rgb};
use prismforge_desktop::{
    CommandError, Shell, ShellConfig, ShellView, StoreView, Stores, ViewSink,
};
use prismforge_effects::{ControlValue, EffectInstance};
use prismforge_testkit::{DemoBackend, DemoControls, FakeServer, FixtureDevice};
use serde::Serialize;
use serde_json::Value;
use tokio::time::sleep;

use crate::Module;
use crate::support::temp::TempDir;

const GPU: &str = "demo-graphics-card";
const MOTHERBOARD: &str = "demo-motherboard";
const MEMORY_1: &str = "demo-memory-1";
const MEMORY_2: &str = "demo-memory-2";
const LIGHT_BAR: &str = "demo-light-bar";
const PLUGIN: &str = "OpenRGB Effects plugin";
const FOLDER: &str = "store-folder";

struct Silent;

impl ViewSink for Silent {
    fn deliver(&self, _: &ShellView) {}
}

/// Profile fixtures with stable ids and no local folder, so they stay the same on every machine.
pub struct ProfileModules {
    pub stores: Module,
    pub details: Module,
    pub saves: Module,
    pub applies: Module,
    pub imports: Module,
    pub views: Module,
    ids: Vec<ProfileId>,
}

impl ProfileModules {
    pub fn new() -> Self {
        Self {
            stores: Module::new("StoreView"),
            details: Module::new("ProfileDetails"),
            saves: Module::new("Saved"),
            applies: Module::new("ApplyOutcome"),
            imports: Module::new("ImportReport"),
            views: Module::new("ShellView"),
            ids: Vec::new(),
        }
    }

    fn know(&mut self, id: &ProfileId) {
        if !self.ids.contains(id) {
            self.ids.push(id.clone());
        }
    }

    fn stable(&self, value: &impl Serialize) -> Value {
        let mut json = serde_json::to_string(value).expect("serialise fixture");
        for (index, id) in self.ids.iter().enumerate() {
            json = json.replace(id.as_str(), &format!("p-{:016x}", index + 1));
        }
        serde_json::from_str(&json).expect("fixture json")
    }

    fn add(&mut self, module: fn(&mut Self) -> &mut Module, name: &str, value: &impl Serialize) {
        let value = self.stable(value);
        module(self).add(name, &value);
    }

    fn store(&mut self, name: &str, mut view: StoreView) {
        let kind = view.folder.rsplit(['/', '\\']).next().unwrap_or_default();
        view.folder = format!("{FOLDER}/{kind}");
        self.add(|modules| &mut modules.stores, name, &view);
    }
}

fn whole(device: &str) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone: None,
    }
}

fn effect(id: &str, values: &[(&str, ControlValue)]) -> EffectInstance {
    EffectInstance {
        effect: id.into(),
        values: values
            .iter()
            .map(|(control, value)| ((*control).to_owned(), value.clone()))
            .collect(),
    }
}

fn shell(dir: &TempDir) -> Shell<Silent> {
    shell_with(dir, SessionConfig::default())
}

fn shell_with(dir: &TempDir, session: SessionConfig) -> Shell<Silent> {
    Shell::new(
        Silent,
        ShellConfig {
            session,
            ..ShellConfig::default()
        },
        Stores::new(dir.path().to_path_buf()),
    )
}

async fn demo(shell: &Shell<Silent>) -> DemoControls {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    shell.start(backend).await.expect("start demo");
    settle().await;
    controls
}

async fn settle() {
    sleep(Duration::from_millis(300)).await;
}

fn error<T: std::fmt::Debug>(result: Result<T, CommandError>) -> CommandError {
    result.expect_err("an error")
}

/// Changes the exported file so its import has a changed mode list and an effect this version does not have.
fn altered(export: &str) -> String {
    let mut file: Value = serde_json::from_str(export).expect("export json");
    let devices = file["devices"].as_array().expect("devices").clone();
    let key_of = |model: &str| {
        devices
            .iter()
            .find(|device| device["model"] == model)
            .and_then(|device| device["reference"].as_str())
            .map(str::to_owned)
            .expect("device in export")
    };
    let (bar, card) = (key_of("Demo Light Bar"), key_of("Demo Graphics Card"));
    for mode in file["modes"].as_array_mut().expect("modes") {
        if mode["device"] == bar.as_str() {
            mode["name"] = "Pulse".into();
        }
    }
    for assignment in file["assignments"].as_array_mut().expect("assignments") {
        if assignment["device"] == card.as_str() {
            assignment["effect"]["effect"] = "aurora".into();
        }
    }
    serde_json::to_string_pretty(&file).expect("altered export")
}

pub async fn generate(modules: &mut ProfileModules, errors: &mut Module) {
    demo_store(modules, errors).await;
    offline_store(modules).await;
    recovered_store(modules).await;
    busy_store(errors).await;
}

async fn demo_store(modules: &mut ProfileModules, errors: &mut Module) {
    let dir = TempDir::new("fixtures-demo");
    let outside = TempDir::new("fixtures-outside");
    let shell = shell(&dir);
    let controls = demo(&shell).await;
    modules.store("storeEmpty", shell.store_view().await.expect("store"));
    errors.add("invalidName", &error(shell.create_profile("   ").await));

    let rainbow = effect("rainbow-wave", &[]);
    for device in [GPU, MEMORY_1] {
        shell
            .apply_effect(whole(device), rainbow.clone())
            .await
            .expect("effect");
    }
    shell
        .select_mode(DeviceRef::new(LIGHT_BAR), 1)
        .await
        .expect("mode");
    shell
        .declare_external(DeviceRef::new(MEMORY_2), PLUGIN.into())
        .await
        .expect("declare");
    let created = shell.create_profile("Evening").await.expect("save");
    let evening = created.id.clone();
    modules.know(&evening);
    modules.add(|modules| &mut modules.saves, "savedNew", &created);
    modules.store("storeSaved", shell.store_view().await.expect("store"));

    shell
        .apply_effect(
            OutputTarget {
                device: DeviceRef::new(MOTHERBOARD),
                zone: Some(1),
            },
            effect(
                "solid",
                &[("color", ControlValue::Color(Rgb::new(0, 200, 120)))],
            ),
        )
        .await
        .expect("zone effect");
    shell.stop_effect(whole(MEMORY_1)).await.expect("stop");
    shell
        .apply_effect(whole(GPU), effect("moving-highlight", &[]))
        .await
        .expect("replace");
    shell.set_frame_rate(45).await.expect("frame rate");
    settle().await;
    let preview = shell.preview_update(&evening).await.expect("preview");
    modules.add(|modules| &mut modules.saves, "previewChanges", &preview);
    modules.store("storeUnsaved", shell.store_view().await.expect("store"));
    let updated = shell.update_profile(&evening).await.expect("update");
    modules.add(|modules| &mut modules.saves, "savedChanges", &updated);
    let none = shell.preview_update(&evening).await.expect("preview");
    modules.add(|modules| &mut modules.saves, "previewNone", &none);
    let details = shell.profile_details(&evening).await.expect("details");
    modules.add(|modules| &mut modules.details, "detailsEvening", &details);

    shell
        .set_device_name(DeviceRef::new(GPU), Some("Desk card".into()))
        .await
        .expect("device name");
    shell
        .set_zone_name(DeviceRef::new(MOTHERBOARD), 1, Some("Front fans".into()))
        .await
        .expect("zone name");
    modules.store("storeNamed", shell.store_view().await.expect("store"));

    let exported = outside.path().join("evening.json");
    shell
        .export_profile(&evening, exported.clone())
        .await
        .expect("export");
    let changed = outside.path().join("changed.json");
    fs::write(
        &changed,
        altered(&fs::read_to_string(&exported).expect("read export")),
    )
    .expect("write changed");
    let imported = shell.import_profile(changed).await.expect("import");
    let older = imported.details.id.clone();
    modules.know(&older);
    modules.add(|modules| &mut modules.imports, "importedRenamed", &imported);
    let details = shell.profile_details(&older).await.expect("details");
    modules.add(|modules| &mut modules.details, "detailsImported", &details);
    let partly = shell.apply_profile(&older).await.expect("apply");
    modules.add(|modules| &mut modules.applies, "appliedPartly", &partly);
    let fully = shell.apply_profile(&evening).await.expect("apply");
    modules.add(|modules| &mut modules.applies, "appliedFully", &fully);
    errors.add(
        "notActive",
        &modules.stable(&error(shell.update_profile(&older).await)),
    );
    errors.add(
        "nameTaken",
        &modules.stable(&error(shell.rename_profile(&older, "Evening").await)),
    );
    modules.store("storeTwo", shell.store_view().await.expect("store"));

    controls.set_call_delay(Duration::from_secs(1));
    let (_, (view, refused)) = tokio::join!(shell.apply_profile(&evening), async {
        while shell.view().applying.is_none() {
            sleep(Duration::from_millis(1)).await;
        }
        let view = shell.view();
        (view, shell.apply_effect(whole(GPU), rainbow.clone()).await)
    });
    controls.set_call_delay(Duration::ZERO);
    modules.add(|modules| &mut modules.views, "viewApplying", &view);
    errors.add("profileApplying", &error(refused));

    shell
        .set_apply_on_connect(Some(evening.clone()))
        .await
        .expect("mark");
    demo(&shell).await;
    modules.add(
        |modules| &mut modules.views,
        "viewConnectApplied",
        &shell.view(),
    );
    let backend = DemoBackend::new();
    backend.controls().fail_connect(true);
    shell.start(backend).await.expect("start");
    settle().await;
    let waiting = shell.apply_profile(&evening).await.expect("waiting");
    modules.add(
        |modules| &mut modules.applies,
        "waitingForConnection",
        &waiting,
    );
    shell.stop().await;
}

async fn offline_store(modules: &mut ProfileModules) {
    let dir = TempDir::new("fixtures-offline");
    let shell = shell(&dir);
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    shell
        .start(prismforge_openrgb::OpenRgbBackend::new(
            server.connector(),
            prismforge_openrgb::ClientConfig::default(),
        ))
        .await
        .expect("start");
    settle().await;
    let desk = shell.view().session.expect("session").devices[0]
        .identity
        .reference()
        .cloned()
        .expect("reference");
    shell
        .apply_effect(
            OutputTarget {
                device: desk,
                zone: None,
            },
            effect("rainbow-wave", &[]),
        )
        .await
        .expect("effect");
    let desk = shell.create_profile("Desk").await.expect("save").id;
    modules.know(&desk);
    shell
        .set_apply_on_connect(Some(desk.clone()))
        .await
        .expect("mark");
    shell.stop().await;
    modules.store("storeOffline", shell.store_view().await.expect("store"));
    let details = shell.profile_details(&desk).await.expect("details");
    modules.add(|modules| &mut modules.details, "detailsOffline", &details);
}

async fn recovered_store(modules: &mut ProfileModules) {
    let dir = TempDir::new("fixtures-recovered");
    let shell = shell(&dir);
    demo(&shell).await;
    shell
        .apply_effect(whole(GPU), effect("rainbow-wave", &[]))
        .await
        .expect("effect");
    let night = shell.create_profile("Night").await.expect("save").id;
    shell
        .rename_profile(&night, "Late night")
        .await
        .expect("rename");
    modules.know(&night);
    shell.stop().await;
    drop(shell);
    let store = dir.path().join("demo-store");
    fs::write(
        store.join("profiles").join(format!("{night}.json")),
        b"damaged",
    )
    .expect("damage profile");
    fs::write(store.join("devices.json"), b"damaged").expect("damage registry");
    let _ = fs::remove_file(store.join("devices.json.bak"));

    let shell = self::shell(&dir);
    demo(&shell).await;
    modules.store("storeRecovered", shell.store_view().await.expect("store"));
    shell.stop().await;
}

async fn busy_store(errors: &mut Module) {
    let dir = TempDir::new("fixtures-busy");
    let shell = shell_with(
        &dir,
        SessionConfig {
            command_capacity: 1,
            ..SessionConfig::default()
        },
    );
    let backend = DemoBackend::new();
    let controls = backend.controls();
    controls.set_call_delay(Duration::from_secs(10));
    shell.start(backend).await.expect("start");
    let (_, busy) = tokio::join!(shell.refresh(), shell.create_profile("Busy"));
    controls.set_call_delay(Duration::ZERO);
    errors.add("sessionBusy", &error(busy));
    shell.stop().await;
}
