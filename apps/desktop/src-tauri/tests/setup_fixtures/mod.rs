use std::fs;
use std::path::Path;
use std::time::Duration;

use prismforge_core::profile::Extra;
use prismforge_core::session::SessionSnapshot;
use prismforge_core::setup::{
    Corner, FirstLed, Geometry, GridPoint, Line, Matrix, Perimeter, Placement, Ring, Runs, Setup,
    Single, Strip, Turn, plan_across,
};
use prismforge_core::{ConnectionState, DeviceRef, OutputTarget, RgbBackend};
use prismforge_desktop::{AcrossPreviewRequest, Shell, ShellConfig, Stores, preview_across};
use prismforge_effects::{EffectInstance, Flow};
use prismforge_openrgb::{ClientConfig, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice, FixtureMode};
use serde_json::{Value, json};

use crate::generated::Generated;
use crate::support::Recorder;
use crate::support::setups::{desk, fan, matrix, output, pc, place, setup, strip, view};
use crate::support::temp::TempDir;
use crate::{BLINK, Module, breathing_strip, reference};

const BOARD: &str = "demo-motherboard";
const GPU: &str = "demo-graphics-card";
const MEMORY_1: &str = "demo-memory-1";
const MEMORY_2: &str = "demo-memory-2";
const LIGHT_BAR: &str = "demo-light-bar";
const DEVICE_TYPE_GPU: i32 = 2;
const DEVICE_TYPE_KEYBOARD: i32 = 5;
const ALONG: Flow = Flow::Along { angle: 0 };

pub struct SetupModules {
    pub setups: Module,
    pub reports: Module,
    pub previews: Module,
}

impl SetupModules {
    pub fn new() -> Self {
        Self {
            setups: Module::new("SetupStoreView"),
            reports: Module::new("AcrossReport"),
            previews: Module::new("AcrossPreview"),
        }
    }
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn whole(device: &DeviceRef) -> OutputTarget {
    OutputTarget {
        device: device.clone(),
        zone: None,
    }
}

/// A PC with four fans on a hub behind one header, a cooler whose two strips are chained on a second header,
/// memory, the graphics card and the board's own LEDs; the desk view holds a light bar and the card again.
fn demo_tower() -> Setup {
    let board = DeviceRef::new(BOARD);
    let gpu = DeviceRef::new(GPU);
    setup(
        vec![
            fan("c-front-fan-1", "Front fan 1", 8),
            fan("c-front-fan-2", "Front fan 2", 8),
            fan("c-front-fan-3", "Front fan 3", 8),
            fan("c-rear-fan", "Rear fan", 8),
            strip("c-cooler-left", "Cooler left strip", 8),
            strip("c-cooler-right", "Cooler right strip", 8),
            strip("c-memory-1", "Memory module 1", 8),
            strip("c-memory-2", "Memory module 2", 8),
            strip("c-graphics", "Graphics card", 12),
            strip("c-board-edge", "Board edge", 6),
            strip("c-desk-bar", "Desk light bar", 10),
        ],
        vec![
            output(
                "o-fan-hub",
                &board,
                Some((1, "Addressable Header 1")),
                "Demo Motherboard",
                &[
                    &["c-front-fan-1"],
                    &["c-front-fan-2"],
                    &["c-front-fan-3"],
                    &["c-rear-fan"],
                ],
            ),
            output(
                "o-cooler",
                &board,
                Some((2, "Addressable Header 2")),
                "Demo Motherboard",
                &[&["c-cooler-left", "c-cooler-right"]],
            ),
            output(
                "o-board-edge",
                &board,
                Some((0, "Onboard")),
                "Demo Motherboard",
                &[&["c-board-edge"]],
            ),
            output(
                "o-memory-1",
                &DeviceRef::new(MEMORY_1),
                None,
                "Demo Memory Module",
                &[&["c-memory-1"]],
            ),
            output(
                "o-memory-2",
                &DeviceRef::new(MEMORY_2),
                None,
                "Demo Memory Module",
                &[&["c-memory-2"]],
            ),
            output(
                "o-graphics",
                &gpu,
                None,
                "Demo Graphics Card",
                &[&["c-graphics"]],
            ),
            output(
                "o-desk-bar",
                &DeviceRef::new(LIGHT_BAR),
                None,
                "Demo Light Bar",
                &[&["c-desk-bar"]],
            ),
        ],
        vec![
            view(
                pc(),
                "PC",
                vec![
                    place("c-front-fan-1", 160, 130, 100, 0),
                    place("c-front-fan-2", 160, 300, 100, 0),
                    place("c-front-fan-3", 160, 470, 100, 0),
                    place("c-rear-fan", 910, 130, 100, 0),
                    place("c-cooler-left", 440, 270, 120, 90),
                    place("c-cooler-right", 560, 270, 120, 90),
                    place("c-memory-1", 690, 270, 130, 90),
                    place("c-memory-2", 800, 270, 130, 90),
                    place("c-graphics", 560, 470, 320, 0),
                    place("c-board-edge", 530, 610, 200, 0),
                ],
            ),
            view(
                desk(),
                "Desk",
                vec![
                    place("c-desk-bar", 500, 120, 520, 0),
                    place("c-graphics", 820, 470, 160, 0),
                ],
            ),
        ],
    )
}

fn fixture_board() -> FixtureDevice {
    FixtureDevice::new("board", "Fixture Motherboard")
        .serial("SN-board")
        .location("pci:board")
        .resizable_zone("ARGB Header 1", 0, 1024, 36)
        .resizable_zone("ARGB Header 2", 0, 1024, 16)
        .zone("Onboard", 6)
}

fn fixture_card() -> FixtureDevice {
    let mut card = FixtureDevice::new("card", "Fixture Graphics Card")
        .serial("SN-card")
        .location("pci:1")
        .zone("Light bar", 12)
        .modes(vec![FixtureMode::direct(), FixtureMode::static_color()]);
    card.device_type = DEVICE_TYPE_GPU;
    card
}

fn fixture_keyboard() -> FixtureDevice {
    let mut keyboard = FixtureDevice::new("keys", "Fixture Keyboard")
        .serial("SN-keys")
        .location("usb:keys")
        .matrix_zone("Keys", 4, 12);
    keyboard.device_type = DEVICE_TYPE_KEYBOARD;
    keyboard
}

fn real_server() -> FakeServer {
    let server = FakeServer::new();
    server.add_device(0, fixture_board());
    server.add_device(1, fixture_card());
    server.add_device(2, fixture_keyboard());
    server.add_device(3, breathing_strip("desk", "Fixture Desk Strip"));
    server
}

fn real_tower(snapshot: &SessionSnapshot) -> Setup {
    let board = reference(snapshot, "SN-board");
    let card = reference(snapshot, "SN-card");
    let keys = reference(snapshot, "SN-keys");
    let desk_strip = reference(snapshot, "SN-desk");
    setup(
        vec![
            fan("c-front-fan-1", "Front fan 1", 8),
            fan("c-front-fan-2", "Front fan 2", 8),
            fan("c-front-fan-3", "Front fan 3", 8),
            fan("c-rear-fan", "Rear fan", 8),
            strip("c-cooler-left", "Cooler left strip", 8),
            strip("c-cooler-right", "Cooler right strip", 8),
            strip("c-graphics", "Graphics card", 12),
            matrix("c-keyboard", "Keyboard", 12, 4),
            strip("c-desk-edge", "Desk edge strip", 12),
        ],
        vec![
            output(
                "o-fan-hub",
                &board,
                Some((0, "ARGB Header 1")),
                "Fixture Motherboard",
                &[
                    &["c-front-fan-1"],
                    &["c-front-fan-2"],
                    &["c-front-fan-3"],
                    &["c-rear-fan"],
                ],
            ),
            output(
                "o-cooler",
                &board,
                Some((1, "ARGB Header 2")),
                "Fixture Motherboard",
                &[&["c-cooler-left", "c-cooler-right"]],
            ),
            output(
                "o-graphics",
                &card,
                None,
                "Fixture Graphics Card",
                &[&["c-graphics"]],
            ),
            output(
                "o-keyboard",
                &keys,
                None,
                "Fixture Keyboard",
                &[&["c-keyboard"]],
            ),
            output(
                "o-desk-edge",
                &desk_strip,
                None,
                "Fixture Desk Strip",
                &[&["c-desk-edge"]],
            ),
        ],
        vec![
            view(
                pc(),
                "PC",
                vec![
                    place("c-front-fan-1", 160, 130, 100, 0),
                    place("c-front-fan-2", 160, 300, 100, 0),
                    place("c-front-fan-3", 160, 470, 100, 0),
                    place("c-rear-fan", 910, 130, 100, 0),
                    place("c-cooler-left", 440, 270, 120, 90),
                    place("c-cooler-right", 560, 270, 120, 90),
                    place("c-graphics", 560, 470, 320, 0),
                ],
            ),
            view(
                desk(),
                "Desk",
                vec![
                    place("c-keyboard", 430, 470, 360, 0),
                    place("c-desk-edge", 500, 640, 760, 0),
                    place("c-graphics", 840, 300, 160, 90),
                ],
            ),
        ],
    )
}

async fn started<B: RgbBackend + Send + 'static>(
    shell: &Shell<Recorder>,
    backend: B,
) -> SessionSnapshot {
    shell.start(backend).await.expect("start");
    loop {
        if let Some(snapshot) = shell.snapshot().await
            && snapshot.connection == ConnectionState::Connected
            && !snapshot.devices.is_empty()
        {
            return snapshot;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn settled(shell: &Shell<Recorder>) -> SessionSnapshot {
    tokio::time::sleep(Duration::from_millis(500)).await;
    shell.snapshot().await.expect("session")
}

fn change_zone_name(folder: &Path, from: &str, to: &str) {
    let file = folder.join("setup.json");
    let stored = fs::read_to_string(&file).expect("setup file");
    fs::write(&file, stored.replace(from, to)).expect("change the zone name");
}

/// Setup views, reports, previews, sessions and live output from real shells on demo and real stores.
pub async fn generate(
    modules: &mut SetupModules,
    sessions: &mut Module,
    live: &mut Module,
    errors: &mut Module,
) {
    demo(modules, sessions, errors).await;
    demo_skips(modules, sessions).await;
    real(modules, sessions, live).await;
    previews(modules);
}

async fn demo(modules: &mut SetupModules, sessions: &mut Module, errors: &mut Module) {
    let dir = TempDir::new("fixtures-setup-demo");
    let recorder = Recorder::default();
    let shell = Shell::new(
        recorder,
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    started(&shell, DemoBackend::new()).await;
    modules
        .setups
        .add("setupDemoEmpty", &shell.setup_view().await.expect("empty"));
    let mut too_long = demo_tower();
    too_long.components[4] = strip("c-cooler-left", "Cooler left strip", 9);
    errors.add(
        "errorSetupExceedsOutput",
        &shell
            .save_setup(too_long)
            .await
            .expect_err("chain too long"),
    );
    let mut too_many = demo_tower();
    too_many.components[0] = fan("c-front-fan-1", "Front fan 1", 121);
    errors.add(
        "errorSetupLedBounds",
        &shell.save_setup(too_many).await.expect_err("bounds"),
    );
    let saved = shell.save_setup(demo_tower()).await.expect("save");
    modules.setups.add("setupDemoTower", &saved);
    errors.add(
        "errorPlanUnknownView",
        &shell
            .apply_across(
                prismforge_core::setup::ViewId::parse("v-gone").expect("view id"),
                ALONG,
                rainbow(),
                None,
            )
            .await
            .expect_err("unknown view"),
    );
    let report = shell
        .apply_across(pc(), ALONG, rainbow(), None)
        .await
        .expect("apply across");
    modules.reports.add("acrossDemoTower", &report);
    sessions.add("demoAcrossRunning", &settled(&shell).await);
    shell
        .apply_effect(whole(&DeviceRef::new(MEMORY_2)), rainbow())
        .await
        .expect("each device");
    sessions.add("demoAcrossBesideEachDevice", &settled(&shell).await);
    shell.stop().await;
}

/// An apply across where one output's zone changed, a device has no Direct Mode and one is busy identifying.
async fn demo_skips(modules: &mut SetupModules, sessions: &mut Module) {
    let dir = TempDir::new("fixtures-setup-skips");
    let stores = || Stores::new(dir.path().to_path_buf());
    let shell = Shell::new(Recorder::default(), ShellConfig::default(), stores());
    started(&shell, DemoBackend::new()).await;
    let mut tower = demo_tower();
    tower.views[0]
        .placements
        .push(place("c-desk-bar", 860, 600, 200, 0));
    shell.save_setup(tower).await.expect("save");
    shell.stop().await;
    drop(shell);
    change_zone_name(
        &dir.path().join("demo-store"),
        "\"Onboard\"",
        "\"Board logo\"",
    );
    let shell = Shell::new(Recorder::default(), ShellConfig::default(), stores());
    started(&shell, DemoBackend::new()).await;
    modules.setups.add(
        "setupDemoZoneChanged",
        &shell.setup_view().await.expect("changed setup"),
    );
    let (_, report) = tokio::join!(shell.identify(DeviceRef::new(GPU), None), async {
        loop {
            if shell
                .snapshot()
                .await
                .is_some_and(|snapshot| snapshot.identify.is_some())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let report = shell
            .apply_across(pc(), ALONG, rainbow(), None)
            .await
            .expect("apply across");
        sessions.add(
            "demoAcrossSkipped",
            &shell.snapshot().await.expect("session"),
        );
        report
    });
    modules.reports.add("acrossDemoSkipped", &report);
    tokio::time::sleep(BLINK).await;
    shell.stop().await;
}

async fn real(modules: &mut SetupModules, sessions: &mut Module, live: &mut Module) {
    let dir = TempDir::new("fixtures-setup-real");
    let recorder = Recorder::default();
    let shell = Shell::new(
        recorder.clone(),
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    let server = real_server();
    let snapshot = started(
        &shell,
        OpenRgbBackend::new(server.connector(), ClientConfig::default()),
    )
    .await;
    sessions.add("openRgbSetupDevices", &snapshot);
    modules
        .setups
        .add("setupRealEmpty", &shell.setup_view().await.expect("empty"));
    let tower = real_tower(&snapshot);
    let saved = shell.save_setup(tower.clone()).await.expect("save");
    modules.setups.add("setupRealTower", &saved);
    let report = shell
        .apply_across(pc(), ALONG, rainbow(), None)
        .await
        .expect("apply across");
    modules.reports.add("acrossRealTower", &report);
    let targets: Vec<OutputTarget> = tower.outputs.iter().map(|each| each.target()).collect();
    shell.watch_live(targets).await.expect("watch");
    let running = settled(&shell).await;
    sessions.add("openRgbAcrossRunning", &running);
    live.add(
        "liveRealAcross",
        recorder.live().last().expect("live output"),
    );
    shell.watch_live(Vec::new()).await.expect("stop watching");
    shell.stop().await;
}

fn previews(modules: &mut SetupModules) {
    let preview = preview_across(AcrossPreviewRequest {
        setup: demo_tower(),
        view: pc(),
        flow: ALONG,
        instance: rainbow(),
        start_ms: 0,
        frame_count: 2,
        interval_ms: 40,
    })
    .expect("preview");
    modules.previews.add("previewDemoTower", &preview);
}

fn sample(geometry: Geometry, placement: Placement) -> Value {
    let leds: Vec<Value> = geometry
        .leds()
        .into_iter()
        .map(|led| {
            let at = placement.locate(led);
            json!({ "x": at.x, "y": at.y })
        })
        .collect();
    json!({ "geometry": geometry, "placement": placement, "leds": leds })
}

fn turned(rotation: u16, flipped: bool) -> Placement {
    let mut placement = place("c-sample", 200, 150, 100, rotation);
    placement.flipped = flipped;
    placement
}

/// LED places for each shape and transform, and the parts and mirrors of the demo setup, from the Rust model.
pub fn geometry(generated: &Generated) {
    let points = |list: &[(i32, i32)]| -> Vec<GridPoint> {
        list.iter()
            .map(|&(x, y)| GridPoint {
                x,
                y,
                extra: Extra::new(),
            })
            .collect()
    };
    let samples = vec![
        sample(
            Geometry::Strip(Strip {
                leds: 5,
                first: FirstLed::AtStart,
                extra: Extra::new(),
            }),
            turned(0, false),
        ),
        sample(
            Geometry::Strip(Strip {
                leds: 5,
                first: FirstLed::AtEnd,
                extra: Extra::new(),
            }),
            turned(90, true),
        ),
        sample(
            Geometry::Ring(Ring {
                leds: 8,
                first_angle: 45,
                turn: Turn::CounterClockwise,
                extra: Extra::new(),
            }),
            turned(30, false),
        ),
        sample(
            Geometry::Matrix(Matrix {
                columns: 4,
                rows: 3,
                first: Corner::BottomRight,
                runs: Runs::AlongColumns,
                serpentine: true,
                extra: Extra::new(),
            }),
            turned(15, true),
        ),
        sample(
            Geometry::Matrix(Matrix {
                columns: 5,
                rows: 1,
                first: Corner::TopLeft,
                runs: Runs::AlongRows,
                serpentine: false,
                extra: Extra::new(),
            }),
            turned(0, false),
        ),
        sample(
            Geometry::Perimeter(Perimeter {
                top: 4,
                right: 2,
                bottom: 4,
                left: 2,
                width: 16,
                height: 9,
                first: Corner::TopRight,
                turn: Turn::CounterClockwise,
                extra: Extra::new(),
            }),
            turned(345, false),
        ),
        sample(Geometry::Point(Single::default()), turned(0, false)),
        sample(
            Geometry::Line(Line {
                leds: 6,
                points: points(&[(0, 0), (10, 0), (10, 5)]),
                first: FirstLed::AtEnd,
                extra: Extra::new(),
            }),
            turned(180, false),
        ),
        sample(
            Geometry::Ring(Ring {
                leds: 8,
                first_angle: 400,
                turn: Turn::Clockwise,
                extra: Extra::new(),
            }),
            turned(0, false),
        ),
    ];
    let tower = demo_tower();
    let parts: Vec<Value> = tower
        .outputs
        .iter()
        .flat_map(|output| tower.parts(output))
        .map(|part| {
            json!({
                "component": part.component,
                "branch": part.branch,
                "start": part.leds.start,
                "end": part.leds.end,
            })
        })
        .collect();
    let shared: Vec<Value> = tower
        .components
        .iter()
        .map(|component| {
            let control = tower.control(&component.id);
            json!({
                "component": component.id,
                "shared_with": control.map(|control| control.shared_with).unwrap_or_default(),
            })
        })
        .collect();
    let plan = plan_across(&tower, &pc(), ALONG, &[]).expect("plan");
    let planned: Vec<Value> = plan
        .targets
        .iter()
        .map(|placed| json!({ "target": placed.target, "leds": placed.points.len() }))
        .collect();
    let file = format!(
        "// Generated from the Rust setup model by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.\n\
import type {{ ComponentId }} from \"../../lib/api/bindings/ComponentId\";\n\
import type {{ Geometry }} from \"../../lib/api/bindings/Geometry\";\n\
import type {{ OutputTarget }} from \"../../lib/api/bindings/OutputTarget\";\n\
import type {{ Placement }} from \"../../lib/api/bindings/Placement\";\n\
import type {{ Setup }} from \"../../lib/api/bindings/Setup\";\n\n\
export const ledSamples: {{ geometry: Geometry; placement: Placement; leds: {{ x: number; y: number }}[] }}[] = {};\n\n\
export const demoTower: Setup = {};\n\n\
export const demoTowerParts: {{ component: ComponentId; branch: number; start: number; end: number }}[] = {};\n\n\
export const demoTowerShared: {{ component: ComponentId; shared_with: ComponentId[] }}[] = {};\n\n\
export const demoTowerPlanned: {{ target: OutputTarget; leds: number }}[] = {};\n",
        pretty(&samples),
        pretty(&tower),
        pretty(&parts),
        pretty(&shared),
        pretty(&planned),
    );
    generated.write("setupModel.ts", &file);
}

fn pretty(value: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(value).expect("serialise fixture")
}
