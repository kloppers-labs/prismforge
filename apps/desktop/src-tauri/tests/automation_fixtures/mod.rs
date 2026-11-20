use std::time::Duration;

use prismforge_core::automation::{RuleAction, Trigger};
use prismforge_core::profile::ProfileId;
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_desktop::{
    Automation, AutomationView, EventSupport, Shell, ShellConfig, ShellView, Stores, ViewSink,
};
use prismforge_effects::EffectInstance;
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice, FixtureMode};
use serde_json::Value;

use crate::Module;
use crate::support::temp::TempDir;

struct Silent;

impl ViewSink for Silent {
    fn deliver(&self, _: &ShellView) {}
}

/// Automation views with stable profile ids and run times.
pub struct AutomationModules {
    pub views: Module,
    ids: Vec<ProfileId>,
}

impl AutomationModules {
    pub fn new() -> Self {
        Self {
            views: Module::new("AutomationView"),
            ids: Vec::new(),
        }
    }

    fn add(&mut self, name: &str, view: &AutomationView) {
        let named = view.rules.iter().filter_map(|rule| match &rule.action {
            RuleAction::ApplyProfile { profile } => Some(profile),
            _ => None,
        });
        for id in view.profiles.iter().map(|choice| &choice.id).chain(named) {
            if !self.ids.contains(id) {
                self.ids.push(id.clone());
            }
        }
        let mut json = serde_json::to_string(view).expect("serialise fixture");
        for (index, id) in self.ids.iter().enumerate() {
            json = json.replace(id.as_str(), &format!("p-{:016x}", index + 0x100));
        }
        let mut value: Value = serde_json::from_str(&json).expect("fixture json");
        crate::fixed_run_times(&mut value);
        for notice in value["notices"].as_array_mut().into_iter().flatten() {
            if let Some(failure) = notice.pointer_mut("/event/WriteFailed/failure") {
                failure["step"] = "Replace".into();
                failure["detail"] = "Access is denied. (os error 5)".into();
            }
        }
        self.views.add(name, &value);
    }
}

fn shell(dir: &TempDir) -> Shell<Silent> {
    Shell::new(
        Silent,
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    )
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(500)).await;
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn apply(profile: &ProfileId) -> RuleAction {
    RuleAction::ApplyProfile {
        profile: profile.clone(),
    }
}

/// A strip with Direct Mode and a keyboard whose modes all save to its memory, as on the owner's PC.
fn desk() -> FakeServer {
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    server.add_device(
        1,
        FixtureDevice::new("keys", "Fixture Keyboard")
            .serial("SN-keys")
            .location("usb:keys")
            .zone("Keys", 12)
            .modes(vec![
                FixtureMode::static_color().saving_automatically(),
                FixtureMode::breathing().saving_automatically(),
            ]),
    );
    server
}

async fn openrgb(dir: &TempDir, server: &FakeServer) -> (Shell<Silent>, ProfileId) {
    let shell = shell(dir);
    shell
        .start(OpenRgbBackend::new(
            server.connector(),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    settle().await;
    let strip = shell.view().session.expect("session").devices[0]
        .identity
        .reference()
        .cloned()
        .expect("resolved");
    shell
        .apply_effect(
            OutputTarget {
                device: strip,
                zone: None,
            },
            rainbow(),
        )
        .await
        .expect("effect");
    let id = shell.create_profile("Evening").await.expect("save").id;
    (shell, id)
}

pub async fn generate(modules: &mut AutomationModules) {
    let available = Automation::new(EventSupport::Available);

    let dir = TempDir::new("automation-demo");
    let demo = shell(&dir);
    demo.start(DemoBackend::new()).await.expect("demo");
    settle().await;
    let target = OutputTarget {
        device: DeviceRef::new("demo-graphics-card"),
        zone: None,
    };
    demo.apply_effect(target, rainbow()).await.expect("effect");
    demo.create_profile("Demo glow").await.expect("save");
    modules.add("demoNoRules", &available.view(&demo).await.expect("view"));
    let unsupported = Automation::new(EventSupport::NotOnThisSystem);
    modules.add(
        "demoNotOnThisSystem",
        &unsupported.view(&demo).await.expect("view"),
    );
    let failed = Automation::new(EventSupport::Failed {
        detail: "the event window could not be created: Access is denied.".into(),
    });
    modules.add("demoEventsFailed", &failed.view(&demo).await.expect("view"));

    let dir = TempDir::new("automation-sleep-failed");
    let server = desk();
    let (shell, _) = openrgb(&dir, &server).await;
    available
        .set_rule(&shell, Trigger::Sleeps, true, RuleAction::LightsOff)
        .await
        .expect("sleep");
    server.silence(prismforge_testkit::wire::SET_CUSTOM_MODE);
    available.at_sleep(&shell).await;
    modules.add(
        "openRgbSleepFailed",
        &available.view(&shell).await.expect("view"),
    );

    let dir = TempDir::new("automation-run-unwritten");
    let server = desk();
    let (shell, _) = openrgb(&dir, &server).await;
    available
        .set_rule(&shell, Trigger::Quits, true, RuleAction::LightsOff)
        .await
        .expect("quit");
    std::fs::create_dir_all(
        dir.path()
            .join("store")
            .join("rule-runs.json")
            .join("blocker"),
    )
    .expect("a folder in the file's place");
    available.at_quit(&shell).await;
    modules.add(
        "openRgbRunNotWritten",
        &available.view(&shell).await.expect("view"),
    );

    let dir = TempDir::new("automation-sleep-wake");
    let server = desk();
    let (shell, evening) = openrgb(&dir, &server).await;
    available
        .set_rule(&shell, Trigger::Sleeps, true, RuleAction::LightsOff)
        .await
        .expect("sleep");
    let both = available
        .set_rule(&shell, Trigger::Wakes, true, apply(&evening))
        .await
        .expect("wake");
    modules.add("openRgbSleepAndWake", &both);
    available.at_sleep(&shell).await;
    available.at_wake(&shell).await;
    modules.add(
        "openRgbAfterSleepAndWake",
        &available.view(&shell).await.expect("view"),
    );

    let dir = TempDir::new("automation-deleted");
    let server = desk();
    let (shell, evening) = openrgb(&dir, &server).await;
    available
        .set_rule(&shell, Trigger::Wakes, true, apply(&evening))
        .await
        .expect("wake");
    shell.delete_profile(&evening).await.expect("delete");
    modules.add(
        "openRgbDeletedProfile",
        &available.view(&shell).await.expect("view"),
    );

    let dir = TempDir::new("automation-openrgb-also");
    let server = desk();
    server.set_settings(
        "ProfileManager",
        r#"{"suspend_profile":{"enabled":true,"name":"Dark"},"resume_profile":{"enabled":true,"name":"Day"}}"#,
    );
    let (shell, evening) = openrgb(&dir, &server).await;
    available
        .set_rule(&shell, Trigger::Sleeps, true, RuleAction::LightsOff)
        .await
        .expect("sleep");
    let also = available
        .set_rule(&shell, Trigger::Wakes, true, apply(&evening))
        .await
        .expect("wake");
    modules.add("openRgbAlsoActs", &also);

    let dir = TempDir::new("automation-migrated");
    let server = desk();
    let (shell, evening) = openrgb(&dir, &server).await;
    shell.stop().await;
    drop(shell);
    let settings = serde_json::json!({
        "format": "prismforge.settings",
        "schema_version": {"major": 1, "minor": 0},
        "apply_on_connect": evening.as_str(),
    });
    std::fs::write(
        dir.path().join("store").join("settings.json"),
        serde_json::to_vec(&settings).expect("json"),
    )
    .expect("write");
    let shell = self::shell(&dir);
    shell
        .start(OpenRgbBackend::new(
            server.connector(),
            ClientConfig::default(),
        ))
        .await
        .expect("start");
    settle().await;
    modules.add(
        "openRgbMigratedConnects",
        &available.view(&shell).await.expect("view"),
    );
}
