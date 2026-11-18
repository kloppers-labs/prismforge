use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected { reason: Option<StateReason> },
    Connecting,
    Connected,
    Retrying { attempt: u32, reason: StateReason },
    Degraded { reasons: Vec<StateReason> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateReason {
    ServerUnreachable,
    UnsupportedProtocol { server_version: Option<u32> },
    ConnectionLost { cause: LossCause },
    InventoryStale,
    ControllerDataRejected { controllers: usize },
    AmbiguousDevices { devices: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossCause {
    Timeout,
    ServerClosed,
    Transport,
    ProtocolViolation,
    CallAbandoned,
}
