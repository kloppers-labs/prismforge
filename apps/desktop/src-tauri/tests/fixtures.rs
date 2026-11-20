mod automation_fixtures;
mod effect_fixtures;
mod generated;
mod group_fixtures;
mod lifecycle_fixtures;
mod profile_fixtures;
mod setup_fixtures;
mod support;

use std::io;
use std::time::Duration;

use prismforge_core::session::{
    IdentifyOutcome, Session, SessionConfig, SessionError, SessionHandle, SessionSnapshot,
};
use prismforge_core::{ConnectionState, DeviceRef, LossCause, RgbBackend};
use prismforge_openrgb::{ClientConfig, Connector, OpenRgbBackend};
use prismforge_testkit::{DemoBackend, FakeServer, FixtureDevice, FixtureMode, wire};
use serde::Serialize;
use tokio::io::DuplexStream;

use generated::Generated;

const ADDRESS: &str = "127.0.0.1:6742";
pub(crate) const BLINK: Duration = Duration::from_secs(3);
const GRAPHICS_CARD: &str = "demo-graphics-card";
const DEVICE_TYPE_GPU: i32 = 2;
const DEVICE_TYPE_KEYBOARD: i32 = 5;

struct Local(FakeServer);

impl Connector for Local {
    type Stream = DuplexStream;

    async fn connect(&self) -> io::Result<DuplexStream> {
        Ok(self.0.connect())
    }

    fn address(&self) -> Option<String> {
        Some(ADDRESS.into())
    }
}

struct Refusing;

impl Connector for Refusing {
    type Stream = DuplexStream;

    async fn connect(&self) -> io::Result<DuplexStream> {
        Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "connection refused",
        ))
    }

    fn address(&self) -> Option<String> {
        Some(ADDRESS.into())
    }
}

struct Module {
    types: &'static str,
    annotation: String,
    exports: Vec<String>,
}

impl Module {
    fn new(types: &'static str) -> Self {
        Self {
            types,
            annotation: types.into(),
            exports: Vec::new(),
        }
    }

    fn nullable(types: &'static str) -> Self {
        Self {
            annotation: format!("{types} | null"),
            ..Self::new(types)
        }
    }

    fn add(&mut self, name: &str, value: &impl Serialize) {
        let json = serde_json::to_string_pretty(value).expect("serialise fixture");
        self.exports.push(format!(
            "export const {name}: {} = {json};\n",
            self.annotation
        ));
    }

    fn render(&self) -> String {
        format!(
            "// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.\nimport type {{ {types} }} from \"../../lib/api/bindings/{types}\";\n\n{}",
            self.exports.join("\n"),
            types = self.types,
        )
    }
}

fn start<B: RgbBackend + Send + 'static>(backend: B) -> SessionHandle {
    let (handle, session) = Session::new(backend, SessionConfig::default());
    tokio::spawn(session.run());
    handle
}

pub(crate) async fn wait_until(
    handle: &SessionHandle,
    condition: impl FnMut(&SessionSnapshot) -> bool,
) -> SessionSnapshot {
    handle
        .subscribe()
        .wait_for(condition)
        .await
        .expect("session running")
        .clone()
}

pub(crate) fn openrgb(server: &FakeServer) -> SessionHandle {
    start(OpenRgbBackend::new(
        Local(server.clone()),
        ClientConfig::default(),
    ))
}

pub(crate) fn breathing_strip(label: &str, name: &str) -> FixtureDevice {
    FixtureDevice::new(label, name)
        .serial(&format!("SN-{label}"))
        .location(&format!("usb:{label}"))
        .resizable_zone("Header", 0, 60, 12)
        .modes(vec![
            FixtureMode::direct(),
            FixtureMode::static_color(),
            FixtureMode::breathing(),
        ])
        .active_mode(2)
}

fn graphics_card() -> FixtureDevice {
    let mut card = FixtureDevice::new("card", "Fixture Graphics Card")
        .serial("SN-card")
        .location("pci:1")
        .zone("Logo", 4)
        .without_direct_mode();
    card.device_type = DEVICE_TYPE_GPU;
    card
}

fn keyboard() -> FixtureDevice {
    let mut keyboard = FixtureDevice::new("keys", "Fixture Keyboard")
        .serial("SN-keys")
        .location("usb:keys")
        .matrix_zone("Keys", 2, 4);
    keyboard.device_type = DEVICE_TYPE_KEYBOARD;
    keyboard
}

fn twin(label: &str) -> FixtureDevice {
    FixtureDevice::new(label, "Fixture Fan").zone("Ring", 8)
}

pub(crate) async fn demo_connected() -> (SessionHandle, prismforge_testkit::DemoControls) {
    let backend = DemoBackend::new();
    let controls = backend.controls();
    let handle = start(backend);
    handle.connect().await.expect("demo connects");
    (handle, controls)
}

async fn demo_sessions(sessions: &mut Module, outcomes: &mut Module) {
    let (handle, _) = demo_connected().await;
    sessions.add("demoConnected", &handle.snapshot());
    let running = tokio::spawn(handle.identify(DeviceRef::new("demo-motherboard"), Some(1), BLINK));
    let identifying = wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    sessions.add("demoIdentifying", &identifying);
    outcomes.add("restored", &outcome(running.await.expect("task")));
}

async fn demo_outcomes(outcomes: &mut Module) {
    let (handle, controls) = demo_connected().await;
    controls.change_inventory();
    let stale = handle
        .identify(DeviceRef::new(GRAPHICS_CARD), None, BLINK)
        .await;
    outcomes.add("unchangedStaleInventory", &outcome(stale));

    let (handle, controls) = demo_connected().await;
    controls.hide_active_mode(&DeviceRef::new(GRAPHICS_CARD));
    handle.refresh().await.expect("refresh");
    let unknown = handle
        .identify(DeviceRef::new(GRAPHICS_CARD), None, BLINK)
        .await;
    outcomes.add("partlyRestoredPreviousStateUnknown", &outcome(unknown));

    let (handle, controls) = demo_connected().await;
    let running = tokio::spawn(handle.identify(DeviceRef::new(GRAPHICS_CARD), None, BLINK));
    wait_until(&handle, |snapshot| snapshot.identify.is_some()).await;
    controls.lose_connection(LossCause::Timeout);
    outcomes.add(
        "notRestoredConnectionLost",
        &outcome(running.await.expect("task")),
    );
}

async fn openrgb_outcomes(outcomes: &mut Module) {
    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha", "Fixture Strip"));
    let handle = openrgb(&server);
    handle.connect().await.expect("connect");
    let alpha = reference(&handle.snapshot(), "SN-alpha");

    server.reject(wire::UPDATE_MODE);
    let refused = handle.identify(alpha.clone(), None, BLINK).await;
    outcomes.add("notRestoredRefused", &outcome(refused));

    server.reject(wire::SET_CUSTOM_MODE);
    let light_refused = handle.identify(alpha, None, BLINK).await;
    outcomes.add("unchangedLightRefused", &outcome(light_refused));
}

async fn openrgb_sessions(sessions: &mut Module) {
    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha", "Fixture Strip"));
    server.add_device(1, graphics_card());
    server.add_device(2, breathing_strip("gamma", "Fixture Light Bar"));
    let handle = openrgb(&server);
    handle.connect().await.expect("connect");
    sessions.add("openRgbConnected", &handle.snapshot());

    let mut uneven = breathing_strip("uneven", "Fixture Uneven Strip");
    uneven.led_count_override = Some(3);
    server.restart(vec![
        (0, breathing_strip("alpha", "Fixture Strip").into()),
        (1, graphics_card().into()),
        (2, keyboard().into()),
        (3, twin("left").into()),
        (4, twin("right").into()),
        (5, uneven.into()),
    ]);
    wait_until(&handle, |snapshot| {
        matches!(snapshot.connection, ConnectionState::Retrying { .. })
    })
    .await;
    let degraded = wait_until(&handle, |snapshot| {
        matches!(snapshot.connection, ConnectionState::Degraded { .. })
    })
    .await;
    sessions.add("openRgbDegraded", &degraded);
}

async fn openrgb_losses(sessions: &mut Module) {
    let server = FakeServer::new();
    server.add_device(0, breathing_strip("alpha", "Fixture Strip"));
    server.add_device(1, graphics_card());
    let handle = openrgb(&server);
    handle.connect().await.expect("connect");
    server.silence(wire::REQUEST_PROTOCOL_VERSION);
    server.disconnect_all();
    let lost = wait_until(&handle, |snapshot| {
        matches!(snapshot.connection, ConnectionState::Retrying { .. })
    })
    .await;
    sessions.add("openRgbLostWhileConnected", &lost);
    let retry_failed = wait_until(&handle, |snapshot| {
        matches!(
            snapshot.connection,
            ConnectionState::Retrying { attempt: 2, .. }
        )
    })
    .await;
    sessions.add("openRgbLostRetryFailed", &retry_failed);

    let rejecting = FakeServer::new();
    let mut uneven = breathing_strip("uneven", "Fixture Uneven Strip");
    uneven.led_count_override = Some(3);
    rejecting.add_device(0, uneven);
    let handle = openrgb(&rejecting);
    handle.connect().await.expect("connect");
    sessions.add("openRgbAllRejected", &handle.snapshot());
}

async fn openrgb_failures(sessions: &mut Module) {
    let refusing = start(OpenRgbBackend::new(Refusing, ClientConfig::default()));
    let _ = refusing.connect().await;
    let third = wait_until(&refusing, |snapshot| {
        matches!(
            snapshot.connection,
            ConnectionState::Retrying { attempt: 3, .. }
        )
    })
    .await;
    sessions.add("openRgbRefused", &third);

    let silent = FakeServer::new();
    silent.silence(wire::REQUEST_PROTOCOL_VERSION);
    let hung = openrgb(&silent);
    let connecting = hung.connect();
    sessions.add(
        "openRgbConnecting",
        &wait_until(&hung, |snapshot| {
            snapshot.connection == ConnectionState::Connecting
        })
        .await,
    );
    let _ = connecting.await;
    sessions.add("openRgbNoVersionAnswer", &hung.snapshot());

    let old = FakeServer::new();
    old.set_protocol_version(Some(5));
    let refused = openrgb(&old);
    let _ = refused.connect().await;
    sessions.add("openRgbOldServer", &refused.snapshot());
}

/// The time of every rule run, fixed so the fixtures stay the same on every run.
pub(crate) const RUN_TIME_MS: u64 = 1_795_000_000_000;

pub(crate) fn fixed_run_times(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            if fields.contains_key("trigger") && fields.contains_key("outcome") {
                fields.insert("at_ms".into(), RUN_TIME_MS.into());
            }
            fields.values_mut().for_each(fixed_run_times);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(fixed_run_times),
        _ => {}
    }
}

pub(crate) fn reference(snapshot: &SessionSnapshot, serial: &str) -> DeviceRef {
    snapshot
        .devices
        .iter()
        .find(|device| device.descriptor.serial == serial)
        .and_then(|device| device.identity.reference())
        .cloned()
        .expect("resolved device")
}

fn outcome(result: Result<IdentifyOutcome, SessionError>) -> IdentifyOutcome {
    result.expect("identify outcome")
}

#[tokio::test(start_paused = true)]
async fn committed_fixtures_match_real_sessions() {
    let generated = Generated::new("src/test/fixtures");
    let mut sessions = Module::new("SessionSnapshot");
    let mut outcomes = Module::new("IdentifyOutcome");

    demo_sessions(&mut sessions, &mut outcomes).await;
    demo_outcomes(&mut outcomes).await;
    openrgb_sessions(&mut sessions).await;
    openrgb_losses(&mut sessions).await;
    openrgb_failures(&mut sessions).await;
    openrgb_outcomes(&mut outcomes).await;
    let mut catalog = Module::new("EffectCatalog");
    let mut stops = Module::nullable("RestoreOutcome");
    let mut errors = Module::new("CommandError");
    let mut previews = Module::new("PreviewBatch");
    effect_fixtures::sessions(&mut sessions).await;
    effect_fixtures::catalog(&mut catalog).await;
    effect_fixtures::stops(&mut stops).await;
    effect_fixtures::errors(&mut errors).await;
    effect_fixtures::previews(&mut previews);
    let mut batches = Module::new("BatchReport");
    effect_fixtures::batches(&mut batches, &mut sessions).await;
    let mut groups = Module::new("GroupsView");
    group_fixtures::generate(&mut groups, &mut sessions).await;
    let mut live = Module::new("LiveOutput");
    effect_fixtures::live(&mut live, &mut sessions).await;
    let mut profiles = profile_fixtures::ProfileModules::new();
    profile_fixtures::generate(&mut profiles, &mut errors).await;
    let mut lifecycles = Module::new("LifecycleView");
    lifecycle_fixtures::generate(&mut lifecycles, &mut errors).await;
    let mut setups = setup_fixtures::SetupModules::new();
    setup_fixtures::generate(&mut setups, &mut sessions, &mut live, &mut errors).await;
    setup_fixtures::geometry(&generated);
    let mut automations = automation_fixtures::AutomationModules::new();
    automation_fixtures::generate(&mut automations).await;

    generated.write("sessions.ts", &sessions.render());
    generated.write("outcomes.ts", &outcomes.render());
    generated.write("catalog.ts", &catalog.render());
    generated.write("stops.ts", &stops.render());
    generated.write("errors.ts", &errors.render());
    generated.write("previews.ts", &previews.render());
    generated.write("batches.ts", &batches.render());
    generated.write("live.ts", &live.render());
    generated.write("groups.ts", &groups.render());
    generated.write("stores.ts", &profiles.stores.render());
    generated.write("profileDetails.ts", &profiles.details.render());
    generated.write("saves.ts", &profiles.saves.render());
    generated.write("applies.ts", &profiles.applies.render());
    generated.write("imports.ts", &profiles.imports.render());
    generated.write("views.ts", &profiles.views.render());
    generated.write("lifecycles.ts", &lifecycles.render());
    generated.write("setups.ts", &setups.setups.render());
    generated.write("acrossReports.ts", &setups.reports.render());
    generated.write("acrossPreviews.ts", &setups.previews.render());
    generated.write("automations.ts", &automations.views.render());
    generated.compare_or_update();
}
