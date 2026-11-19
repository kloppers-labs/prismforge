mod support;

use std::fs;

use prismforge_core::profile::{Extra, ItemOutcome};
use prismforge_core::session::TargetOutcome;
use prismforge_core::setup::{
    Component, ComponentId, FirstLed, Geometry, OutputId, Placement, PlanError, Ring, Setup,
    SetupOutput, SetupProblem, Strip, Turn, View, ViewId,
};
use prismforge_core::store::{ApplyOutcome, MemberState, StoreError};
use prismforge_core::{ApplyMode, DeviceRef, OutputTarget};
use prismforge_desktop::{
    AcrossPreviewRequest, CommandError, Shell, ShellConfig, Stores, preview_across,
};
use prismforge_effects::{EffectInstance, Flow};
use prismforge_testkit::DemoBackend;

use support::temp::TempDir;
use support::{Recorder, is_connected};

const BOARD: &str = "demo-motherboard";
const GPU: &str = "demo-graphics-card";
const MEMORY: &str = "demo-memory-1";
const HEADER_ONE: usize = 1;
const HEADER_TWO: usize = 2;
const ALONG: Flow = Flow::Along { angle: 0 };

fn shell_on(dir: &TempDir) -> (Shell<Recorder>, Recorder) {
    let recorder = Recorder::default();
    let stores = Stores::new(dir.path().to_path_buf());
    (
        Shell::new(recorder.clone(), ShellConfig::default(), stores),
        recorder,
    )
}

async fn demo(shell: &Shell<Recorder>, recorder: &Recorder) {
    shell.start(DemoBackend::new()).await.expect("start demo");
    recorder.wait_for(is_connected).await;
}

fn component(id: &str, geometry: Geometry) -> Component {
    Component {
        id: ComponentId::parse(id).expect("component id"),
        name: id.trim_start_matches("c-").replace('-', " "),
        geometry,
        extra: Extra::new(),
    }
}

fn fan(id: &str, leds: u32) -> Component {
    component(
        id,
        Geometry::Ring(Ring {
            leds,
            first_angle: 0,
            turn: Turn::Clockwise,
            extra: Extra::new(),
        }),
    )
}

fn strip(id: &str, leds: u32) -> Component {
    component(
        id,
        Geometry::Strip(Strip {
            leds,
            first: FirstLed::AtStart,
            extra: Extra::new(),
        }),
    )
}

fn ids(names: &[&str]) -> Vec<ComponentId> {
    names
        .iter()
        .map(|name| ComponentId::parse(name).expect("component id"))
        .collect()
}

fn output(id: &str, device: &str, zone: Option<usize>, branches: &[&[&str]]) -> SetupOutput {
    SetupOutput {
        id: OutputId::parse(id).expect("output id"),
        device: DeviceRef::new(device),
        zone,
        label: device.into(),
        zone_name: None,
        first_led: 0,
        branches: branches.iter().map(|branch| ids(branch)).collect(),
        extra: Extra::new(),
    }
}

fn place(id: &str, x: i32, y: i32, size: u32) -> Placement {
    Placement {
        component: ComponentId::parse(id).expect("component id"),
        x,
        y,
        size,
        rotation: 0,
        flipped: false,
        extra: Extra::new(),
    }
}

fn pc() -> ViewId {
    ViewId::parse("v-pc").expect("view id")
}

fn view(placements: Vec<Placement>) -> View {
    View {
        id: pc(),
        name: "PC".into(),
        placements,
        extra: Extra::new(),
    }
}

/// A hub of four mirrored fans on header 1, a two-part cooler chained on header 2 and the graphics card's logo.
fn tower() -> Setup {
    let mut zone_two = output(
        "o-cooler",
        BOARD,
        Some(HEADER_TWO),
        &[&["c-cooler-left", "c-cooler-right"]],
    );
    zone_two.zone_name = Some("Addressable Header 2".into());
    let mut zone_one = output(
        "o-fan-hub",
        BOARD,
        Some(HEADER_ONE),
        &[&["c-fan-1"], &["c-fan-2"], &["c-fan-3"], &["c-fan-4"]],
    );
    zone_one.zone_name = Some("Addressable Header 1".into());
    Setup {
        components: vec![
            fan("c-fan-1", 8),
            fan("c-fan-2", 8),
            fan("c-fan-3", 8),
            fan("c-fan-4", 8),
            strip("c-cooler-left", 8),
            strip("c-cooler-right", 8),
            fan("c-graphics-logo", 12),
        ],
        outputs: vec![
            zone_one,
            zone_two,
            output("o-graphics", GPU, None, &[&["c-graphics-logo"]]),
        ],
        views: vec![view(vec![
            place("c-fan-1", 100, 100, 80),
            place("c-fan-2", 100, 200, 80),
            place("c-fan-3", 100, 300, 80),
            place("c-fan-4", 100, 400, 80),
            place("c-cooler-left", 400, 200, 100),
            place("c-cooler-right", 520, 200, 100),
            place("c-graphics-logo", 400, 500, 120),
        ])],
        extra: Extra::new(),
    }
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

fn target(device: &str, zone: Option<usize>) -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new(device),
        zone,
    }
}

#[tokio::test(start_paused = true)]
async fn the_setup_is_saved_and_read_back_with_the_state_of_each_output() {
    let dir = TempDir::new("setup-round-trip");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;

    let empty = shell.setup_view().await.expect("empty setup");
    let saved = shell.save_setup(tower()).await.expect("save");
    let again = shell.setup_view().await.expect("read again");

    assert_eq!(empty.setup, Setup::default());
    assert_eq!(saved.setup, tower());
    assert_eq!(again, saved);
    assert_eq!(
        again
            .outputs
            .iter()
            .map(|status| status.state)
            .collect::<Vec<_>>(),
        vec![MemberState::Resolved; 3]
    );
    assert_eq!(again.limits.component_leds, 4096);
}

#[tokio::test(start_paused = true)]
async fn saving_refuses_a_component_with_more_leds_than_its_zone_allows() {
    let dir = TempDir::new("setup-bounds");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    let mut setup = tower();
    setup.components[4] = strip("c-cooler-left", 17);

    let refused = shell.save_setup(setup).await;

    assert_eq!(
        refused,
        Err(CommandError::Store(StoreError::InvalidSetup {
            problem: SetupProblem::LedCountOutsideBounds {
                component: ComponentId::parse("c-cooler-left").expect("id"),
                leds: 17,
                min: 1,
                max: 16,
            }
        }))
    );
    assert_eq!(
        shell.setup_view().await.expect("view").setup,
        Setup::default()
    );
}

#[tokio::test(start_paused = true)]
async fn apply_across_runs_one_picture_and_reports_what_gets_nothing() {
    let dir = TempDir::new("setup-apply");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    let mut setup = tower();
    setup.components.push(strip("c-memory", 8));
    let mut memory = output("o-memory", MEMORY, Some(0), &[&["c-memory"]]);
    memory.zone_name = Some("Module".into());
    setup.outputs.push(memory);
    setup.views[0]
        .placements
        .push(place("c-memory", 600, 100, 80));
    shell.save_setup(setup).await.expect("save");
    shell.stop().await;
    drop(shell);
    let file = dir.path().join("demo-store").join("setup.json");
    let stored = fs::read_to_string(&file).expect("setup file");
    fs::write(
        &file,
        stored.replace("\"Module\"", "\"A zone that is gone\""),
    )
    .expect("change the zone");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;

    let report = shell
        .apply_across(pc(), ALONG, rainbow(), None)
        .await
        .expect("apply across");

    let applied: Vec<_> = report
        .report
        .results
        .iter()
        .map(|result| (result.target.clone(), result.outcome.clone()))
        .collect();
    assert_eq!(
        applied,
        vec![
            (target(BOARD, Some(HEADER_ONE)), TargetOutcome::Applied),
            (target(BOARD, Some(HEADER_TWO)), TargetOutcome::Applied),
            (target(GPU, None), TargetOutcome::Applied),
        ]
    );
    assert_eq!(report.excluded.len(), 1);
    assert_eq!(report.excluded[0].target, target(MEMORY, Some(0)));
    assert_eq!(report.excluded[0].state, MemberState::Changed);
    assert_eq!(report.not_placed.len(), 1);
    assert_eq!(report.not_placed[0].target, target(BOARD, Some(HEADER_ONE)));
    assert_eq!(
        (report.not_placed[0].leds, report.not_placed[0].of),
        (22, 30)
    );
    let scene = shell.snapshot().await.expect("session").output.scene;
    assert!(scene.assignments.iter().all(|assignment| assignment.mode
        == ApplyMode::AcrossSetup {
            view: pc(),
            flow: ALONG,
        }));
}

#[tokio::test(start_paused = true)]
async fn apply_across_to_some_targets_sends_only_those() {
    let dir = TempDir::new("setup-retry");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    shell.save_setup(tower()).await.expect("save");

    let report = shell
        .apply_across(pc(), ALONG, rainbow(), Some(vec![target(GPU, None)]))
        .await
        .expect("apply across");

    assert_eq!(report.report.results.len(), 1);
    assert_eq!(report.report.results[0].target, target(GPU, None));
    assert!(report.excluded.is_empty());
}

#[tokio::test(start_paused = true)]
async fn apply_across_a_view_that_is_not_there_is_a_plan_error() {
    let dir = TempDir::new("setup-plan-error");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    let desk = ViewId::parse("v-desk").expect("view id");

    let refused = shell
        .apply_across(desk.clone(), ALONG, rainbow(), None)
        .await;
    let flow = shell
        .apply_across(pc(), Flow::Along { angle: 360 }, rainbow(), None)
        .await;

    assert_eq!(
        refused,
        Err(CommandError::Plan(PlanError::UnknownView { view: desk }))
    );
    assert_eq!(
        flow,
        Err(CommandError::Plan(PlanError::InvalidFlow {
            flow: Flow::Along { angle: 360 }
        }))
    );
}

#[test]
fn the_draft_preview_renders_the_unsaved_setup_without_a_session() {
    let preview = preview_across(AcrossPreviewRequest {
        setup: tower(),
        view: pc(),
        flow: ALONG,
        instance: rainbow(),
        start_ms: 0,
        frame_count: 3,
        interval_ms: 40,
    })
    .expect("preview");

    let shapes: Vec<_> = preview
        .targets
        .iter()
        .map(|each| (each.target.clone(), each.frames.len(), each.frames[0].len()))
        .collect();
    assert_eq!(
        shapes,
        vec![
            (target(BOARD, Some(HEADER_ONE)), 3, 8),
            (target(BOARD, Some(HEADER_TWO)), 3, 16),
            (target(GPU, None), 3, 12),
        ]
    );
    let hub = &preview.targets[0].frames[0];
    assert_ne!(
        hub[2], hub[6],
        "a rainbow from left to right differs on the two sides of a fan"
    );
}

#[test]
fn the_draft_preview_refuses_a_setup_too_large_to_draw() {
    let mut setup = tower();
    setup.components = (0..8)
        .map(|index| strip(&format!("c-long-{index}"), 4096))
        .collect();
    setup.outputs = vec![output(
        "o-long",
        BOARD,
        Some(HEADER_ONE),
        &[&[
            "c-long-0", "c-long-1", "c-long-2", "c-long-3", "c-long-4", "c-long-5", "c-long-6",
            "c-long-7",
        ]],
    )];
    setup.views = vec![view(
        (0..8)
            .map(|index| place(&format!("c-long-{index}"), 100, 100 * index, 50))
            .collect(),
    )];

    let refused = preview_across(AcrossPreviewRequest {
        setup,
        view: pc(),
        flow: ALONG,
        instance: rainbow(),
        start_ms: 0,
        frame_count: 3,
        interval_ms: 40,
    });

    assert!(
        matches!(refused, Err(CommandError::PreviewTooLarge { .. })),
        "{refused:?}"
    );
}

/// From a zone the demo backend reports to a placed fan, an effect across the setup and a profile that brings it back.
#[tokio::test(start_paused = true)]
async fn a_detected_zone_becomes_a_placed_fan_that_runs_across_the_setup_and_returns_with_a_profile()
 {
    let dir = TempDir::new("setup-journey");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    let snapshot = shell.snapshot().await.expect("session");
    let board = snapshot
        .devices
        .iter()
        .find(|device| device.descriptor.name == "Demo Motherboard")
        .expect("board");
    let reference = board.identity.reference().cloned().expect("resolved");
    let zone = &board.descriptor.zones[HEADER_ONE];
    let mut fans = output(
        "o-header",
        reference.as_str(),
        Some(HEADER_ONE),
        &[&["c-front-fan"]],
    );
    fans.label = board.descriptor.name.clone();
    fans.zone_name = Some(zone.name.clone());
    let setup = Setup {
        components: vec![fan("c-front-fan", 12)],
        outputs: vec![fans],
        views: vec![view(vec![place("c-front-fan", 200, 200, 120)])],
        extra: Extra::new(),
    };

    shell.save_setup(setup).await.expect("save");
    shell
        .apply_across(pc(), Flow::FromCentre { x: 50, y: 50 }, rainbow(), None)
        .await
        .expect("apply across");
    let id = shell.create_profile("Across").await.expect("profile").id;
    shell.stop_all_effects().await.expect("stop");
    let applied = shell.apply_profile(&id).await.expect("apply profile");

    assert!(matches!(applied, ApplyOutcome::Applied(_)), "{applied:?}");
    let scene = shell.snapshot().await.expect("session").output.scene;
    assert_eq!(scene.assignments.len(), 1);
    assert_eq!(
        scene.assignments[0].target,
        target(reference.as_str(), Some(HEADER_ONE))
    );
    assert_eq!(
        scene.assignments[0].mode,
        ApplyMode::AcrossSetup {
            view: pc(),
            flow: Flow::FromCentre { x: 50, y: 50 },
        }
    );
}

/// Owner question still open: whether a profile applied after its view changed should warn. Today it applies without one.
#[tokio::test(start_paused = true)]
async fn a_profile_applied_after_its_view_changed_applies_without_a_warning() {
    let dir = TempDir::new("setup-view-changed");
    let (shell, recorder) = shell_on(&dir);
    demo(&shell, &recorder).await;
    let mut setup = tower();
    shell.save_setup(setup.clone()).await.expect("save");
    shell
        .apply_across(pc(), ALONG, rainbow(), None)
        .await
        .expect("apply across");
    let id = shell.create_profile("Tower").await.expect("profile").id;
    setup.views[0].placements[6].x = 900;
    setup.views[0].placements[6].rotation = 90;
    shell.save_setup(setup).await.expect("move the logo");
    shell.stop_all_effects().await.expect("stop");

    let applied = shell.apply_profile(&id).await.expect("apply profile");

    let ApplyOutcome::Applied(report) = applied else {
        panic!("{applied:?}");
    };
    assert!(
        report
            .items
            .iter()
            .all(|item| item.outcome == ItemOutcome::Applied),
        "{report:?}"
    );
    let scene = shell.snapshot().await.expect("session").output.scene;
    assert_eq!(scene.assignments.len(), 3);
}
