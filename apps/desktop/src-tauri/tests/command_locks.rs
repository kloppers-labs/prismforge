mod support;

use std::collections::BTreeSet;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use prismforge_core::automation::{RuleAction, Trigger};
use prismforge_core::profile::ProfileId;
use prismforge_core::setup::{Setup, ViewId};
use prismforge_core::store::{CloseAction, GroupId, NoticeEvent, StoreNotice};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_desktop::{
    AcrossPreviewRequest, AppSettingChange, Automation, EventSupport, ExitGuard, Lifecycle,
    OnConnect, Shell, ShellConfig, Stores, SystemEvent, TrayState, end_of_session, marks,
    preview_across, preview_effect, quit_with_rule, stop_then_exit,
};
use prismforge_effects::{EffectInstance, Flow, PreviewRequest};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice};

use support::lifecycle::{FakeEntry, FakeExit, FakeWindow, LifecycleRecorder};
use support::setups::{fan, output, pc, place, setup, view};
use support::temp::TempDir;
use support::{ConnectsRule, Recorder, is_connected};

const PATIENCE: Duration = Duration::from_secs(600);
const GPU: &str = "demo-graphics-card";

type Run<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

/// A connected demo shell with one saved, exported profile marked to apply on connect.
struct Fixture {
    shell: Shell<Recorder>,
    lifecycle: Lifecycle<LifecycleRecorder>,
    automation: Automation,
    window: FakeWindow,
    exit: FakeExit,
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

fn memory() -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new("demo-memory-1"),
        zone: None,
    }
}

fn unknown_group() -> GroupId {
    GroupId::parse("g-0000000000000000").expect("group id")
}

/// A group made in this join, so the command meets whichever session the partner left.
async fn made_group(f: &Fixture) -> GroupId {
    f.shell
        .create_group("Made", vec![gpu()])
        .await
        .ok()
        .and_then(|view| view.groups.into_iter().find(|group| group.name == "Made"))
        .map_or_else(unknown_group, |group| group.id)
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

fn logo_setup() -> Setup {
    setup(
        vec![fan("c-logo", "Logo", 12)],
        vec![output(
            "o-logo",
            &DeviceRef::new(GPU),
            None,
            "Demo Graphics Card",
            &[&["c-logo"]],
        )],
        vec![view(pc(), "PC", vec![place("c-logo", 100, 100, 100, 0)])],
    )
}

fn across_preview() -> AcrossPreviewRequest {
    AcrossPreviewRequest {
        setup: logo_setup(),
        view: pc(),
        flow: Flow::Along { angle: 0 },
        instance: effect("rainbow-wave"),
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
        case!("connect_openrgb", "fake server", |f| {
            marks::connect_openrgb(
                &f.shell,
                &f.lifecycle,
                "127.0.0.1",
                6742,
                openrgb(&f.server),
            )
            .await
        }),
        case!("start_demo", "demo", |f| marks::start(
            &f.shell,
            &f.lifecycle,
            DemoBackend::new()
        )
        .await),
        case!("disconnect", "stop", |f| marks::stop(
            &f.shell,
            &f.lifecycle
        )
        .await),
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
        case!("apply_effect_batch", "two targets", |f| f
            .shell
            .apply_batch(vec![gpu(), memory()], effect("rainbow-wave"))
            .await),
        case!("apply_effect_batch", "unknown effect", |f| f
            .shell
            .apply_batch(vec![gpu()], effect("no-such-effect"))
            .await),
        case!("stop_effect_batch", "two targets", |f| f
            .shell
            .stop_batch(vec![gpu(), memory()])
            .await),
        case!("watch_live_output", "two targets", |f| f
            .shell
            .watch_live(vec![gpu(), memory()])
            .await),
        case!("watch_live_output", "none", |f| f
            .shell
            .watch_live(Vec::new())
            .await),
        case!("stop_all_effects", "session", |f| f
            .shell
            .stop_all_effects()
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
        case!("groups", "view", |f| f.shell.groups_view().await),
        case!("create_group", "two targets", |f| f
            .shell
            .create_group("Desk", vec![gpu(), memory()])
            .await),
        case!("create_group", "unknown device", |f| f
            .shell
            .create_group(
                "Nowhere",
                vec![OutputTarget {
                    device: DeviceRef::new("no-such-device"),
                    zone: None,
                }]
            )
            .await),
        case!("rename_group", "made here", |f| {
            let id = made_group(f).await;
            f.shell.rename_group(&id, "Renamed").await
        }),
        case!("rename_group", "unknown", |f| f
            .shell
            .rename_group(&unknown_group(), "Renamed")
            .await),
        case!("add_group_members", "made here", |f| {
            let id = made_group(f).await;
            f.shell.add_group_members(&id, vec![memory()]).await
        }),
        case!("remove_group_member", "made here", |f| {
            let id = made_group(f).await;
            f.shell.remove_group_member(&id, gpu()).await
        }),
        case!("delete_group", "made here", |f| {
            let id = made_group(f).await;
            f.shell.delete_group(&id).await
        }),
        case!("delete_group", "unknown", |f| f
            .shell
            .delete_group(&unknown_group())
            .await),
        case!("setup", "view", |f| f.shell.setup_view().await),
        case!("save_setup", "logo", |f| f
            .shell
            .save_setup(logo_setup())
            .await),
        case!("save_setup", "not valid", |f| {
            let mut setup = logo_setup();
            setup.views[0].placements[0].size = 0;
            f.shell.save_setup(setup).await
        }),
        case!("apply_across", "saved here", |f| {
            let _ = f.shell.save_setup(logo_setup()).await;
            f.shell
                .apply_across(pc(), Flow::Along { angle: 0 }, effect("rainbow-wave"), None)
                .await
        }),
        case!("apply_across", "unknown view", |f| f
            .shell
            .apply_across(
                ViewId::parse("v-nowhere").expect("view id"),
                Flow::Along { angle: 0 },
                effect("rainbow-wave"),
                Some(vec![gpu()])
            )
            .await),
        case!("preview_across", "logo", |_f| preview_across(
            across_preview()
        )),
        case!("preview_across", "unknown effect", |_f| {
            let mut request = across_preview();
            request.instance = effect("no-such-effect");
            preview_across(request)
        }),
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
        case!("rename_profile", "profile", |f| marks::rename_profile(
            &f.shell,
            &f.lifecycle,
            &f.id,
            "Renamed"
        )
        .await),
        case!("rename_profile", "unknown", |f| marks::rename_profile(
            &f.shell,
            &f.lifecycle,
            &f.unknown,
            "Renamed"
        )
        .await),
        case!("duplicate_profile", "profile", |f| f
            .shell
            .duplicate_profile(&f.id)
            .await),
        case!("duplicate_profile", "unknown", |f| f
            .shell
            .duplicate_profile(&f.unknown)
            .await),
        case!("delete_profile", "profile", |f| marks::delete_profile(
            &f.shell,
            &f.lifecycle,
            &f.id
        )
        .await),
        case!("delete_profile", "unknown", |f| marks::delete_profile(
            &f.shell,
            &f.lifecycle,
            &f.unknown
        )
        .await),
        case!("apply_profile", "profile", |f| f
            .shell
            .apply_profile(&f.id)
            .await),
        case!("apply_profile", "unknown", |f| f
            .shell
            .apply_profile(&f.unknown)
            .await),
        case!("automation", "view", |f| f.automation.view(&f.shell).await),
        case!("set_rule", "connects applies the profile", |f| {
            let action = RuleAction::ApplyProfile {
                profile: f.id.clone(),
            };
            marks::set_rule(
                &f.shell,
                &f.lifecycle,
                &f.automation,
                Trigger::Connects,
                true,
                action,
            )
            .await
        }),
        case!("set_rule", "connects off", |f| {
            let action = RuleAction::Nothing;
            marks::set_rule(
                &f.shell,
                &f.lifecycle,
                &f.automation,
                Trigger::Connects,
                false,
                action,
            )
            .await
        }),
        case!("set_rule", "lights off at sleep", |f| {
            let action = RuleAction::LightsOff;
            marks::set_rule(
                &f.shell,
                &f.lifecycle,
                &f.automation,
                Trigger::Sleeps,
                true,
                action,
            )
            .await
        }),
        case!("set_rule", "unknown profile at wake", |f| {
            let action = RuleAction::ApplyProfile {
                profile: f.unknown.clone(),
            };
            marks::set_rule(
                &f.shell,
                &f.lifecycle,
                &f.automation,
                Trigger::Wakes,
                true,
                action,
            )
            .await
        }),
        case!("export_profile", "file name and file", |f| {
            let _ = f.shell.export_file_name(&f.id).await;
            f.shell.export_profile(&f.id, f.destination.clone()).await
        }),
        case!("export_profile", "relative path", |f| f
            .shell
            .export_profile(&f.id, PathBuf::from("glow.json"))
            .await),
        case!("import_profile", "file", |f| marks::import_profile(
            &f.shell,
            &f.lifecycle,
            f.exported.clone()
        )
        .await),
        case!("import_profile", "missing file", |f| marks::import_profile(
            &f.shell,
            &f.lifecycle,
            f.outside.path().join("missing.json")
        )
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
        case!("lifecycle", "publish", |f| f
            .lifecycle
            .publish(&f.shell)
            .await),
        case!("change_setting", "background", |f| f
            .lifecycle
            .change(&f.shell, AppSettingChange::OnClose(CloseAction::Background))
            .await),
        case!("change_setting", "start with the system", |f| f
            .lifecycle
            .change(&f.shell, AppSettingChange::StartWithSystem(true))
            .await),
        case!("change_setting", "start hidden", |f| f
            .lifecycle
            .change(&f.shell, AppSettingChange::StartHidden(true))
            .await),
        case!("change_setting", "connect at start", |f| f
            .lifecycle
            .change(&f.shell, AppSettingChange::ConnectAtStart(true))
            .await),
        case!("answer_close", "keep running", |f| {
            f.lifecycle
                .close_requested(&f.shell, &f.window, &f.exit)
                .await;
            f.lifecycle
                .answer_close(&f.shell, Some(CloseAction::Background), &f.window, &f.exit)
                .await
        }),
        case!("answer_close", "cancel", |f| f
            .lifecycle
            .answer_close(&f.shell, None, &f.window, &f.exit)
            .await),
        case!("quit", "exit handler", |f| {
            f.lifecycle.quit(&f.shell, &f.window, &f.exit).await;
            stop_then_exit(&f.shell, || {}).await
        }),
    ]
}

/// The glue's actions (`tray::Action`), the tray model and start-up: not commands, but the same paths.
fn system_actions() -> Vec<Case> {
    vec![
        case!("Show", "tray click", |f| f
            .lifecycle
            .show(&f.shell, &f.window)
            .await),
        case!("Toggle", "tray item", |f| f
            .lifecycle
            .toggle(&f.shell, &f.window)
            .await),
        case!("StopAll", "tray item", |f| f
            .lifecycle
            .stop_all(&f.shell)
            .await),
        case!("Quit", "tray item, app menu or keyboard", |f| {
            f.lifecycle.quit(&f.shell, &f.window, &f.exit).await;
            stop_then_exit(&f.shell, || {}).await
        }),
        case!("SecondInstance", "second start", |f| f
            .lifecycle
            .second_instance(&f.shell, &f.window)
            .await),
        case!("CloseRequested", "window close", |f| f
            .lifecycle
            .close_requested(&f.shell, &f.window, &f.exit)
            .await),
        case!("Sleep", "sleep rule", |f| f
            .automation
            .on_event(&f.shell, SystemEvent::Sleep)
            .await),
        case!("Wake", "wake sequence and rule", |f| f
            .automation
            .on_event(&f.shell, SystemEvent::Wake)
            .await),
        case!("SessionEnd", "session-end rule, then the stop", |f| {
            f.automation
                .on_event(&f.shell, SystemEvent::SessionEnd)
                .await;
            end_of_session(&f.shell, &f.automation, &ExitGuard::default()).await
        }),
        case!("quit rule", "quit rule, then the stop", |f| {
            quit_with_rule(&f.shell, &f.automation, &ExitGuard::default(), || {}).await
        }),
        case!("tray model", "follow", |f| f
            .lifecycle
            .tray_model(&f.shell)
            .await),
        case!("start up", "connect at start armed", |f| {
            let _ = f
                .lifecycle
                .remember_address(&f.shell, "127.0.0.1", 6742)
                .await;
            let _ = f
                .lifecycle
                .change(&f.shell, AppSettingChange::ConnectAtStart(true))
                .await;
            let connects = std::cell::Cell::new(0);
            f.lifecycle
                .begin(&f.shell, false, &f.window, |_| async {
                    connects.set(connects.get() + 1);
                    let _ = f.shell.start(openrgb(&f.server)).await;
                })
                .await;
            assert_eq!(connects.get(), 1, "connect at start did not run");
        }),
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
    let automation = Automation::new(EventSupport::Available);
    let rules = [
        (Trigger::Sleeps, RuleAction::LightsOff),
        (
            Trigger::Wakes,
            RuleAction::ApplyProfile {
                profile: id.clone(),
            },
        ),
        (Trigger::SessionEnds, RuleAction::LightsOff),
        (
            Trigger::Quits,
            RuleAction::ApplyProfile {
                profile: id.clone(),
            },
        ),
    ];
    for (trigger, action) in rules {
        automation
            .set_rule(&shell, trigger, true, action)
            .await
            .expect("rule");
    }
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
    let lifecycle = Lifecycle::new(LifecycleRecorder::default(), FakeEntry::default());
    lifecycle.set_tray(TrayState::Available);
    Fixture {
        shell,
        lifecycle,
        automation: Automation::new(EventSupport::Available),
        window: FakeWindow::default(),
        exit: FakeExit::default(),
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
    for case in cases().into_iter().chain(system_actions()) {
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

fn glue_actions() -> BTreeSet<String> {
    let tray = source("src/tray.rs");
    let (_, body) = tray
        .split_once("pub enum Action {")
        .expect("the glue's action enum");
    let (body, _) = body.split_once('}').expect("end of the action enum");
    body.split(',')
        .map(str::trim)
        .filter(|variant| !variant.is_empty())
        .map(str::to_owned)
        .collect()
}

fn system_events() -> BTreeSet<String> {
    let events = source("src/system_events.rs");
    let (_, body) = events
        .split_once("pub enum SystemEvent {")
        .expect("the system event enum");
    let (body, _) = body.split_once('}').expect("end of the event enum");
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("///"))
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect()
}

#[test]
fn every_system_event_is_in_the_lock_table() {
    let table: BTreeSet<String> = system_actions()
        .iter()
        .map(|case| case.command.to_owned())
        .collect();

    let missing: Vec<_> = system_events().difference(&table).cloned().collect();

    assert_eq!(system_events().len(), 3);
    assert!(
        missing.is_empty(),
        "system events not in the lock table: {missing:?}"
    );
}

#[test]
fn every_glue_action_is_in_the_lock_table() {
    let table: BTreeSet<String> = system_actions()
        .iter()
        .map(|case| case.command.to_owned())
        .collect();

    let missing: Vec<_> = glue_actions().difference(&table).cloned().collect();

    assert!(
        missing.is_empty(),
        "glue actions not in the lock table: {missing:?}"
    );
}
