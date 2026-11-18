//! Deterministic OpenRGB SDK fixtures (encoder written independently of the client) and a simulated demo backend.

mod demo;
mod fixture;
mod recording;
mod server;
pub mod wire;

pub use demo::{DemoBackend, DemoControls, DemoOutput};
pub use fixture::{ControllerBlock, FixtureDevice, FixtureMode, FixtureZone};
pub use recording::{ColorWrite, ModeUpdate, ReceivedPacket};
pub use server::FakeServer;
