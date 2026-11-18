//! Deterministic OpenRGB SDK fixtures with an encoder written independently of the client.

mod fixture;
mod recording;
mod server;
pub mod wire;

pub use fixture::{ControllerBlock, FixtureDevice, FixtureMode, FixtureZone};
pub use recording::{ColorWrite, ModeUpdate, ReceivedPacket};
pub use server::FakeServer;
