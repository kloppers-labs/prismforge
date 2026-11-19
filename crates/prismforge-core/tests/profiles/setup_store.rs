use prismforge_core::DeviceRef;
use prismforge_core::session::SessionDevice;
use prismforge_core::setup::{
    Component, ComponentId, FirstLed, Geometry, OutputId, Placement, Ring, Setup, SetupOutput,
    SetupProblem, Strip, Turn, View, ViewId,
};
use prismforge_core::store::{
    FileProblem, NoticeEvent, ProfileStore, Profiles, SETUP_FORMAT, StdFileOps, StoreError,
    StoreNotice,
};
use serde_json::{Map, Value, json};

use crate::session::demo;
use crate::support::TempDir;

const FILE: &str = "setup.json";
const BOARD: &str = "demo-motherboard";
const HEADER: usize = 1;

async fn devices() -> Vec<SessionDevice> {
    let (handle, _) = demo().await;
    handle.snapshot().devices
}

fn component_id(value: &str) -> ComponentId {
    ComponentId::parse(value).expect("component id")
}

fn ring(id: &str, name: &str) -> Component {
    Component {
        id: component_id(id),
        name: name.into(),
        geometry: Geometry::Ring(Ring {
            leds: 12,
            first_angle: 0,
            turn: Turn::Clockwise,
            extra: Map::new(),
        }),
        extra: Map::new(),
    }
}

fn placed(id: &str, x: i32) -> Placement {
    Placement {
        component: component_id(id),
        x,
        y: 0,
        size: 40,
        rotation: 0,
        flipped: false,
        extra: Map::new(),
    }
}

/// Two 12-LED fans chained on the demo board's first addressable header, placed in a PC view.
fn fans() -> Setup {
    Setup {
        components: vec![ring("c-front", "Front fan"), ring("c-rear", "Rear fan")],
        outputs: vec![SetupOutput {
            id: OutputId::parse("o-header").expect("output id"),
            device: DeviceRef::new(BOARD),
            zone: Some(HEADER),
            label: "Demo Motherboard".into(),
            zone_name: Some("Addressable Header 1".into()),
            first_led: 0,
            branches: vec![vec![component_id("c-front"), component_id("c-rear")]],
            extra: Map::new(),
        }],
        views: vec![View {
            id: ViewId::parse("v-pc").expect("view id"),
            name: "PC".into(),
            placements: vec![placed("c-front", 0), placed("c-rear", 60)],
            extra: Map::new(),
        }],
        extra: Map::new(),
    }
}

fn renamed(mut setup: Setup, name: &str) -> Setup {
    setup.components[0].name = name.into();
    setup
}

fn stored(dir: &TempDir) -> Value {
    serde_json::from_slice(&dir.read(FILE)).expect("json")
}

#[tokio::test(start_paused = true)]
async fn a_setup_round_trips_with_its_format_and_version() {
    let dir = TempDir::new("setup-round-trip");
    let devices = devices().await;
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert_eq!(store.setup(), Ok(&Setup::default()));

    store.save_setup(fans(), &devices).expect("save");
    let again = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");

    assert_eq!(again.setup(), Ok(&fans()));
    assert!(again.notices().is_empty());
    let file = stored(&dir);
    assert_eq!(file["format"], json!(SETUP_FORMAT));
    assert_eq!(file["schema_version"], json!({ "major": 1, "minor": 0 }));
    assert_eq!(
        file["components"][0]["geometry"],
        json!({ "Ring": { "leds": 12, "first_angle": 0, "turn": "Clockwise" } })
    );
}

#[tokio::test(start_paused = true)]
async fn names_are_trimmed_and_an_unchanged_setup_writes_nothing() {
    let dir = TempDir::new("setup-unchanged");
    let devices = devices().await;
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");

    store
        .save_setup(renamed(fans(), "  Front fan "), &devices)
        .expect("save");
    assert_eq!(store.setup(), Ok(&fans()));
    store.save_setup(fans(), &devices).expect("same again");

    assert_eq!(store.setup(), Ok(&fans()));
    assert!(
        !dir.files().contains(&format!("{FILE}.bak")),
        "{:?}",
        dir.files()
    );
}

#[tokio::test(start_paused = true)]
async fn a_damaged_setup_is_restored_from_its_backup() {
    let dir = TempDir::new("setup-recovery");
    let devices = devices().await;
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.save_setup(fans(), &devices).expect("first");
    store
        .save_setup(renamed(fans(), "Intake"), &devices)
        .expect("second");
    drop(store);
    dir.write(FILE, b"{\"format\": \"prismforge.setup\", \"schema_vers");

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");

    assert_eq!(store.setup(), Ok(&fans()));
    assert_eq!(
        store.notices(),
        [StoreNotice {
            file: FILE.into(),
            event: NoticeEvent::RestoredFromBackup {
                problem: Some(FileProblem::NotJson {
                    detail: "EOF while parsing a string at line 1 column 43".into(),
                }),
                set_aside: Some(format!("{FILE}.unreadable-1")),
                rewritten: true,
            },
        }]
    );
    assert!(dir.files().contains(&format!("{FILE}.unreadable-1")));
}

#[tokio::test(start_paused = true)]
async fn a_setup_from_a_newer_prismforge_is_refused_and_left_untouched() {
    let dir = TempDir::new("setup-newer");
    let devices = devices().await;
    let newer = serde_json::to_vec_pretty(&json!({
        "format": SETUP_FORMAT,
        "schema_version": { "major": 2, "minor": 0 },
        "rooms": []
    }))
    .expect("encode");
    dir.write(FILE, &newer);

    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let problem = FileProblem::NewerVersion {
        found: 2,
        supported: 1,
    };

    assert_eq!(
        store.setup(),
        Err(StoreError::NewerFile {
            file: FILE.into(),
            problem: problem.clone(),
        })
    );
    assert_eq!(
        store.notices(),
        [StoreNotice {
            file: FILE.into(),
            event: NoticeEvent::LeftUntouched {
                problem: problem.clone()
            },
        }]
    );
    assert_eq!(
        store.save_setup(fans(), &devices),
        Err(StoreError::NewerFile {
            file: FILE.into(),
            problem,
        })
    );
    assert_eq!(dir.read(FILE), newer);
    assert_eq!(dir.files(), [FILE]);
}

#[tokio::test(start_paused = true)]
async fn unknown_fields_of_a_newer_minor_version_are_kept() {
    let dir = TempDir::new("setup-unknown");
    let devices = devices().await;
    let mut file = serde_json::to_value(fans()).expect("encode");
    file["format"] = json!(SETUP_FORMAT);
    file["schema_version"] = json!({ "major": 1, "minor": 3 });
    file["future_setup"] = json!(1);
    file["components"][0]["future_component"] = json!("kept");
    file["components"][0]["geometry"]["Ring"]["future_geometry"] = json!([2]);
    file["outputs"][0]["future_output"] = json!({ "a": null });
    file["views"][0]["future_view"] = json!(true);
    file["views"][0]["placements"][1]["future_placement"] = json!(3);
    dir.write(FILE, &serde_json::to_vec(&file).expect("encode"));

    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let mut setup = store.setup().expect("readable").clone();
    setup.components[1].name = "Exhaust".into();
    store.save_setup(setup, &devices).expect("save");

    let written = stored(&dir);
    assert_eq!(written["schema_version"], json!({ "major": 1, "minor": 0 }));
    assert_eq!(written["future_setup"], json!(1));
    assert_eq!(written["components"][0]["future_component"], json!("kept"));
    assert_eq!(
        written["components"][0]["geometry"]["Ring"]["future_geometry"],
        json!([2])
    );
    assert_eq!(written["outputs"][0]["future_output"], json!({ "a": null }));
    assert_eq!(written["views"][0]["future_view"], json!(true));
    assert_eq!(
        written["views"][0]["placements"][1]["future_placement"],
        json!(3)
    );
    assert_eq!(written["components"][1]["name"], json!("Exhaust"));
}

#[tokio::test(start_paused = true)]
async fn a_setup_that_is_not_valid_is_refused_and_the_file_stays() {
    let dir = TempDir::new("setup-invalid");
    let devices = devices().await;
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.save_setup(fans(), &devices).expect("save");
    let before = dir.read(FILE);
    let mut twice = fans();
    twice.outputs[0]
        .branches
        .push(vec![component_id("c-front")]);
    let mut too_long = fans();
    too_long.components[0].geometry = Geometry::Strip(Strip {
        leds: 40,
        first: FirstLed::AtStart,
        extra: Map::new(),
    });

    assert_eq!(
        store.save_setup(twice, &devices),
        Err(StoreError::InvalidSetup {
            problem: SetupProblem::WiredTwice {
                component: component_id("c-front"),
            },
        })
    );
    assert_eq!(
        store.save_setup(too_long, &devices),
        Err(StoreError::InvalidSetup {
            problem: SetupProblem::ExceedsOutput {
                output: OutputId::parse("o-header").expect("output id"),
                needed: 52,
                available: 30,
            },
        })
    );
    assert_eq!(dir.read(FILE), before);
    assert_eq!(store.setup(), Ok(&fans()));
}

#[tokio::test(start_paused = true)]
async fn a_new_output_must_name_a_device_reported_now() {
    let dir = TempDir::new("setup-new-output");
    let devices = devices().await;
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    let mut elsewhere = fans();
    elsewhere.outputs[0].device = DeviceRef::new("no-such-device");

    assert_eq!(
        store.save_setup(elsewhere, &devices),
        Err(StoreError::InvalidSetup {
            problem: SetupProblem::UnknownTarget {
                output: OutputId::parse("o-header").expect("output id"),
            },
        })
    );
    assert!(dir.files().is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_stored_output_that_no_longer_fits_does_not_block_other_edits() {
    let dir = TempDir::new("setup-stale-output");
    let devices = devices().await;
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.save_setup(fans(), &devices).expect("save");
    let mut smaller = devices.clone();
    let board = smaller
        .iter_mut()
        .find(|device| device.identity.reference() == Some(&DeviceRef::new(BOARD)))
        .expect("board");
    board.descriptor.zones[HEADER].led_count = 20;

    store
        .save_setup(renamed(fans(), "Intake"), &smaller)
        .expect("an unrelated edit");
    let mut moved = renamed(fans(), "Intake");
    moved.outputs[0].first_led = 1;

    assert!(matches!(
        store.save_setup(moved, &smaller),
        Err(StoreError::InvalidSetup {
            problem: SetupProblem::ExceedsOutput { .. }
        })
    ));
}

#[tokio::test(start_paused = true)]
async fn the_async_store_saves_the_setup_against_the_session_devices() {
    let dir = TempDir::new("setup-async");
    let (handle, _) = demo().await;
    let profiles = Profiles::open(dir.path().to_path_buf(), StdFileOps)
        .await
        .expect("open");

    profiles
        .save_setup(fans(), Some(&handle))
        .await
        .expect("save");

    assert_eq!(profiles.setup().await, Ok(fans()));
    profiles
        .save_setup(renamed(fans(), "Other"), None)
        .await
        .expect("a name needs no device");
    let mut moved = fans();
    moved.outputs[0].first_led = 2;
    assert_eq!(
        profiles.save_setup(moved, None).await,
        Err(StoreError::InvalidSetup {
            problem: SetupProblem::UnknownTarget {
                output: OutputId::parse("o-header").expect("output id"),
            },
        })
    );
}
