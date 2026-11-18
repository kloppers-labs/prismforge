use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::connection::StateReason;
use crate::device::DeviceRef;
use crate::error::BackendError;
use crate::session::identify::IdentifyOutcome;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Diagnostics {
    pub last_error: Option<ErrorRecord>,
    pub counters: Counters,
    /// Most recent events, oldest first; older events are dropped once the ring is full.
    pub events: Vec<SessionEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ErrorRecord {
    pub at_ms: u64,
    pub operation: Operation,
    pub error: BackendError,
    /// Set when this error ended the connection.
    pub ended_connection: Option<StateReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Operation {
    Connect,
    Refresh,
    Identify,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Counters {
    pub connect_attempts: u64,
    pub connect_failures: u64,
    pub inventory_refreshes: u64,
    pub refresh_failures: u64,
    pub commands_refused_busy: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SessionEvent {
    pub sequence: u64,
    pub at_ms: u64,
    pub kind: SessionEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SessionEventKind {
    ConnectAttempt,
    Connected,
    ConnectFailed {
        error: BackendError,
    },
    ConnectionLost {
        reason: StateReason,
    },
    RetryScheduled {
        attempt: u32,
        delay_ms: u64,
    },
    InventoryChanged,
    InventoryRefreshed {
        generation: u64,
        devices: usize,
    },
    RefreshFailed {
        error: BackendError,
    },
    Disconnected,
    IdentifyStarted {
        device: DeviceRef,
        zone: Option<usize>,
    },
    IdentifyFinished {
        outcome: IdentifyOutcome,
    },
}

pub(crate) struct EventRing {
    capacity: usize,
    next_sequence: u64,
    events: VecDeque<SessionEvent>,
}

impl EventRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            next_sequence: 0,
            events: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, at_ms: u64, kind: SessionEventKind) {
        if self.capacity == 0 {
            return;
        }
        if self.events.len() == self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(SessionEvent {
            sequence: self.next_sequence,
            at_ms,
            kind,
        });
        self.next_sequence += 1;
    }

    pub fn to_vec(&self) -> Vec<SessionEvent> {
        self.events.iter().cloned().collect()
    }
}
