mod support;

use std::collections::BTreeSet;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use prismforge_core::profile::ProfileId;
use prismforge_core::store::{NoticeEvent, StoreNotice};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_desktop::{OnConnect, Shell, ShellConfig, Stores, preview_effect, stop_then_exit};
use prismforge_effects::{EffectInstance, PreviewRequest};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice};

use support::temp::TempDir;
use support::{Recorder, is_connected};

const PATIENCE: Duration = Duration::from_secs(600);
const GPU: &str = "demo-graphics-card";

type Run<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

/// A connected demo shell with one saved, exported profile marked to apply on connect.
struct Fixture {
    shell: Shell<Recorder>,
    id: ProfileId,
    unknown: ProfileId,
    exported: PathBuf,
    destination: PathBuf,
    server: FakeServer,
    _dir: TempDir,
    outside: TempDir,
}

/// One registered command, driven through the shell method under it.
struct Case {
    command: &'static str,
    variant: &'static str,
    run: for<'a> fn(&'a Fixture) -> Run<'a>,
}

macro_rules! case {
    ($command:literal, $variant:literal, |$fixture:ident| $body:expr) => {
        Case {
            command: $command,
            variant: $variant,
            run: {
                fn run<'a>($fixture: &'a Fixture) -> Run<'a> {
                    Box::pin(async move {
                        let _ = $body;
                    })
                }
                run
            },
        }
    };
}

fn gpu() -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(GPU),
        zone: None,
    }
}

fn effect(name: &str) -> EffectInstance {
    EffectInstance {
        effect: name.into(),
        values: Default::default(),
    }
}

fn preview(name: &str) -> PreviewRequest {
    PreviewRequest {
        instance: effect(name),
        led_count: 12,
        start_ms: 0,
        frame_count: 30,
        interval_ms: 33,
    }
}

fn notice() -> StoreNotice {
    StoreNotice {
        file: "profiles/none.json".into(),
        event: NoticeEvent::StartedEmpty {
            main: None,
            backup: None,
            set_aside: Vec::new(),
        },
    }
}

fn openrgb(server: &FakeServer) -> OpenRgbBackend<impl Connector + use<>> {
    OpenRgbBackend::new(server.connector(), ClientConfig::default())
}

/// Every command registered with Tauri; the guard test keeps this list complete.
fn cases() -> Vec<Case> {
    vec![
        case!("view", "current", |f| f.shell.view()),
        case!("connect_openrgb", "fake server", |f| f
            .shell
            .start(openrgb(&f.server))
            .await),
        case!("start_demo", "demo", |f| f
            .shell
            .start(DemoBackend::new())
            .await),
        case!("disconnect", "stop", |f| f.shell.stop().await),
        case!("refresh", "session", |f| f.shell.refresh().await),
        case!("identify", "device", |f| f
            .shell
            .identify(DeviceRef::new("demo-motherboard"), None)
            .await),
        case!("identify", "unknown device", |f| f
            .shell
            .identify(DeviceRef::new("no-such-device"), Some(0))
            .await),
        case!("effect_catalog", "session", |f| f.shell.catalog().await),
        case!("apply_effect", "rainbow", |f| f
            .shell
            .apply_effect(gpu(), effect("rainbow-wave"))
            .await),
        case!("apply_effect", "unknown effect", |f| f
            .shell
            .apply_effect(gpu(), effect("no-such-effect"))
            .await),
        case!("stop_effect", "device", |f| f
            .shell
            .stop_effect(gpu())
            .await),
        case!("select_mode", "light bar", |f| f
            .shell
            .select_mode(DeviceRef::new("demo-light-bar"), 1)
            .await),
        case!("select_mode", "no such mode", |f| f
            .shell
            .select_mode(DeviceRef::new("demo-light-bar"), 99)
            .await),
        case!("declare_external", "device", |f| f
            .shell
            .declare_external(DeviceRef::new(GPU), "Other".into())
            .await),
        case!("clear_external", "device", |f| f
            .shell
            .clear_external(DeviceRef::new(GPU))
            .await),
        case!("set_frame_rate", "20", |f| f.shell.set_frame_rate(20).await),
        case!("set_frame_rate", "0", |f| f.shell.set_frame_rate(0).await),
        case!("preview_effect", "rainbow", |_f| preview_effect(preview(
            "rainbow-wave"
        ))),
        case!("preview_effect", "unknown effect", |_f| preview_effect(
            preview("no-such-effect")
        )),
        case!("profile_store", "store", |f| f.shell.store_view().await),
        case!("profile_details", "profile", |f| f
            .shell
            .profile_details(&f.id)
            .await),
        case!("profile_details", "unknown", |f| f
            .shell
            .profile_details(&f.unknown)
            .await),
        case!("create_profile", "name", |f| f
            .shell
            .create_profile("During")
            .await),
        case!("create_profile", "empty name", |f| f
            .shell
            .create_profile("")
            .await),
        case!("preview_profile_update", "profile", |f| f
            .shell
            .preview_update(&f.id)
            .await),
        case!("preview_profile_update", "unknown", |f| f
            .shell
            .preview_update(&f.unknown)
            .await),
        case!("update_profile", "profile", |f| f
            .shell
            .update_profile(&f.id)
            .await),
        case!("update_profile", "unknown", |f| f
            .shell
            .update_profile(&f.unknown)
            .await),
        case!("rename_profile", "profile", |f| f
            .shell
            .rename_profile(&f.id, "Renamed")
            .await),
        case!("rename_profile", "unknown", |f| f
            .shell
            .rename_profile(&f.unknown, "Renamed")
            .await),
        case!("duplicate_profile", "profile", |f| f
            .shell
            .duplicate_profile(&f.id)
            .await),
        case!("duplicate_profile", "unknown", |f| f
            .shell
            .duplicate_profile(&f.unknown)
            .await),
        case!("delete_profile", "profile", |f| f
            .shell
            .delete_profile(&f.id)
            .await),
        case!("delete_profile", "unknown", |f| f
            .shell
            .delete_profile(&f.unknown)
            .await),
        case!("apply_profile", "profile", |f| f
            .shell
            .apply_profile(&f.id)
            .await),
        case!("apply_profile", "unknown", |f| f
            .shell
            .apply_profile(&f.unknown)
            .await),
        case!("set_apply_on_connect", "mark", |f| f
            .shell
            .set_apply_on_connect(Some(f.id.clone()))
            .await),
        case!("set_apply_on_connect", "unmark", |f| f
            .shell
            .set_apply_on_connect(None)
            .await),
        case!("set_apply_on_connect", "unknown", |f| f
            .shell
            .set_apply_on_connect(Some(f.unknown.clone()))
            .await),
        case!("export_profile", "file name and file", |f| {
            let _ = f.shell.export_file_name(&f.id).await;
            f.shell.export_profile(&f.id, f.destination.clone()).await
        }),
        case!("export_profile", "relative path", |f| f
            .shell
            .export_profile(&f.id, PathBuf::from("glow.json"))
            .await),
        case!("import_profile", "file", |f| f
            .shell
            .import_profile(f.exported.clone())
            .await),
        case!("import_profile", "missing file", |f| f
            .shell
            .import_profile(f.outside.path().join("missing.json"))
            .await),
        case!("set_device_name", "name", |f| f
            .shell
            .set_device_name(DeviceRef::new(GPU), Some("Card".into()))
            .await),
        case!("set_device_name", "clear", |f| f
            .shell
            .set_device_name(DeviceRef::new(GPU), None)
            .await),
        case!("set_zone_name", "name", |f| f
            .shell
            .set_zone_name(DeviceRef::new(GPU), 0, Some("Logo light".into()))
            .await),
        case!("set_zone_name", "no such zone", |f| f
            .shell
            .set_zone_name(DeviceRef::new(GPU), 9, Some("Nowhere".into()))
            .await),
        case!("dismiss_store_notice", "absent notice", |f| f
            .shell
            .dismiss_notice(notice())
            .await),
    ]
}

/// A store with one profile marked to apply on connect and an export of it, made once per test.
struct Template {
    dir: TempDir,
    outside: TempDir,
    id: ProfileId,
}

async fn template() -> Template {
    let dir = TempDir::new("table-template");
    let outside = TempDir::new("table-template-out");
    let recorder = Recorder::default();
    let shell = Shell::new(
        recorder.clone(),
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    shell.start(DemoBackend::new()).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell
        .apply_effect(gpu(), effect("rainbow-wave"))
        .await
        .expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    shell
        .set_apply_on_connect(Some(id.clone()))
        .await
        .expect("mark");
    shell
        .export_profile(&id, outside.path().join("glow.json"))
        .await
        .expect("export");
    shell.stop().await;
    drop(shell);
    Template { dir, outside, id }
}

fn copy_folder(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("folder");
    for entry in fs::read_dir(from).expect("read folder") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_folder(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy");
        }
    }
}

/// A fresh copy of the template's store, connected, after its apply on connect has run.
async fn fixture(template: &Template) -> Fixture {
    let dir = TempDir::new("table");
    copy_folder(template.dir.path(), dir.path());
    let outside = TempDir::new("table-out");
    let recorder = Recorder::default();
    let shell = Shell::new(
        recorder.clone(),
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    shell.start(DemoBackend::new()).await.expect("start");
    recorder
        .wait_view(|view| matches!(view.on_connect, OnConnect::Fired { .. }))
        .await;
    let server = FakeServer::new();
    server.add_device(
        0,
        FixtureDevice::new("desk", "Fixture Strip")
            .serial("SN-desk")
            .location("usb:desk")
            .zone("Ring", 8),
    );
    Fixture {
        shell,
        id: template.id.clone(),
        unknown: ProfileId::parse("p-0000000000000000").expect("id"),
        exported: template.outside.path().join("glow.json"),
        destination: outside.path().join("again.json"),
        server,
        _dir: dir,
        outside,
    }
}

#[derive(Clone, Copy)]
enum Partner {
    Disconnect,
    Switch,
    WindowClose,
}

impl Partner {
    fn run<'a>(self, f: &'a Fixture, exited: &'a AtomicBool) -> Run<'a> {
        match self {
            Self::Disconnect => Box::pin(async move {
                f.shell.stop().await;
                exited.store(true, Ordering::SeqCst);
            }),
            Self::Switch => Box::pin(async move {
                let _ = f.shell.start(openrgb(&f.server)).await;
                exited.store(true, Ordering::SeqCst);
            }),
            Self::WindowClose => Box::pin(stop_then_exit(&f.shell, || {
                exited.store(true, Ordering::SeqCst)
            })),
        }
    }
}

async fn responds(shell: &Shell<Recorder>) -> bool {
    tokio::time::timeout(PATIENCE, shell.stop()).await.is_ok()
        && tokio::time::timeout(PATIENCE, shell.start(DemoBackend::new()))
            .await
            .is_ok()
        && tokio::time::timeout(PATIENCE, shell.stop()).await.is_ok()
}

#[derive(Clone, Copy, Debug)]
enum Turn {
    Apply,
    Command,
    Partner,
}

const WITHOUT_APPLY: &[&[Turn]] = &[
    &[Turn::Command, Turn::Partner],
    &[Turn::Partner, Turn::Command],
];

const WITH_APPLY: &[&[Turn]] = &[
    &[Turn::Apply, Turn::Command, Turn::Partner],
    &[Turn::Apply, Turn::Partner, Turn::Command],
    &[Turn::Command, Turn::Apply, Turn::Partner],
    &[Turn::Command, Turn::Partner, Turn::Apply],
    &[Turn::Partner, Turn::Apply, Turn::Command],
    &[Turn::Partner, Turn::Command, Turn::Apply],
];

/// Joins `turns` in that poll order, every time the join is polled.
async fn joined(turns: &[Turn], case: &Case, f: &Fixture, partner: Partner, done: &AtomicBool) {
    let mut apply: Option<Run<'_>> = Some(Box::pin(async {
        let _ = f.shell.apply_profile(&f.id).await;
    }));
    let mut command = Some((case.run)(f));
    let mut other = Some(partner.run(f, done));
    let mut take = |turn: &Turn| {
        match turn {
            Turn::Apply => apply.take(),
            Turn::Command => command.take(),
            Turn::Partner => other.take(),
        }
        .expect("each turn once")
    };
    match turns {
        [first, second] => {
            tokio::join!(biased; take(first), take(second));
        }
        [first, second, third] => {
            tokio::join!(biased; take(first), take(second), take(third));
        }
        _ => unreachable!("two or three turns"),
    }
}

/// Runs every case in every order beside `partner`, and names every join that never ends.
async fn every_command_beside(partner: Partner, orders: &[&[Turn]]) {
    let template = template().await;
    let mut hung = Vec::new();
    for case in cases() {
        for turns in orders {
            let f = fixture(&template).await;
            let done = AtomicBool::new(false);
            let all = joined(turns, &case, &f, partner, &done);
            let ended = tokio::time::timeout(PATIENCE, all).await.is_ok();
            if !(ended && done.load(Ordering::SeqCst) && responds(&f.shell).await) {
                hung.push(format!("{} ({}) {turns:?}", case.command, case.variant));
            }
        }
    }
    assert!(hung.is_empty(), "never finished: {hung:#?}");
}

#[tokio::test(start_paused = true)]
async fn every_command_finishes_beside_a_disconnect() {
    every_command_beside(Partner::Disconnect, WITHOUT_APPLY).await;
}

#[tokio::test(start_paused = true)]
async fn every_command_finishes_beside_a_switch_to_openrgb() {
    every_command_beside(Partner::Switch, WITHOUT_APPLY).await;
}

#[tokio::test(start_paused = true)]
async fn every_command_finishes_beside_a_window_close() {
    every_command_beside(Partner::WindowClose, WITHOUT_APPLY).await;
}

#[tokio::test(start_paused = true)]
async fn every_command_finishes_beside_an_apply_and_a_disconnect() {
    every_command_beside(Partner::Disconnect, WITH_APPLY).await;
}

#[tokio::test(start_paused = true)]
async fn every_command_finishes_beside_an_apply_and_a_switch_to_openrgb() {
    every_command_beside(Partner::Switch, WITH_APPLY).await;
}

#[tokio::test(start_paused = true)]
async fn every_command_finishes_beside_an_apply_and_a_window_close() {
    every_command_beside(Partner::WindowClose, WITH_APPLY).await;
}

fn source(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)).expect(path)
}

fn registered() -> BTreeSet<String> {
    let lib = source("src/lib.rs");
    let (_, list) = lib
        .split_once("generate_handler![")
        .expect("generate_handler");
    let (list, _) = list.split_once(']').expect("end of the handler list");
    list.split(',')
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| path.rsplit("::").next().unwrap_or(path).to_owned())
        .collect()
}

fn permitted() -> BTreeSet<String> {
    let capability: serde_json::Value =
        serde_json::from_str(&source("capabilities/main.json")).expect("capability json");
    capability["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .filter_map(|permission| permission.as_str()?.strip_prefix("allow-"))
        .map(|command| command.replace('-', "_"))
        .collect()
}

#[test]
fn every_registered_command_is_in_the_lock_table() {
    let table: BTreeSet<String> = cases().iter().map(|case| case.command.to_owned()).collect();

    let registered = registered();

    let missing: Vec<_> = registered.difference(&table).collect();
    let unknown: Vec<_> = table.difference(&registered).collect();
    assert!(
        missing.is_empty(),
        "registered but not in the lock table: {missing:?}"
    );
    assert!(
        unknown.is_empty(),
        "in the lock table but not registered: {unknown:?}"
    );
    assert_eq!(permitted(), registered);
}
