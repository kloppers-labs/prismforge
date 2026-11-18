//! OpenRGB SDK protocol 6 client and PrismForge backend adapter.

mod adapter;
mod client;
mod connector;
pub mod protocol;
mod reader;
mod translate;

pub use adapter::OpenRgbBackend;
pub use client::{
    CLIENT_PROTOCOL_VERSION, ClientConfig, ClientError, MIN_PROTOCOL_VERSION, Notification,
    OpenRgbClient,
};
pub use connector::{Connector, TcpConnector};
