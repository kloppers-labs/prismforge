use std::collections::{HashMap, HashSet};
use std::future::{Ready, ready};
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::io::{
    AsyncReadExt, AsyncWriteExt, DuplexStream, ReadHalf, WriteHalf, copy_bidirectional,
};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;

use crate::fixture::{ControllerBlock, FixtureDevice};
use crate::recording::{ColorWrite, ReceivedPacket};
use crate::wire::{self, Writer, u32_at};

const DUPLEX_BUFFER: usize = 64 * 1024;
const MAX_CLIENT_PACKET: u32 = 16 * 1024 * 1024;
const WAIT_ROUNDS: usize = 10_000;
const SERVER_NAME: &str = "PrismForge fake OpenRGB";

/// Scripted OpenRGB SDK server speaking the wire protocol over in-memory duplex streams.
#[derive(Clone)]
pub struct FakeServer {
    state: Arc<Mutex<State>>,
}

struct State {
    protocol_version: Option<u32>,
    controllers: Vec<Controller>,
    received: Vec<ReceivedPacket>,
    connections: Vec<Connection>,
    silent_packets: HashSet<u32>,
    reply_delays: HashMap<u32, Duration>,
    session: u32,
    next_connection: usize,
}

struct Controller {
    id: u32,
    label: String,
    block: ControllerBlock,
}

struct Connection {
    outgoing: UnboundedSender<Vec<u8>>,
    tasks: [JoinHandle<()>; 2],
}

impl Default for FakeServer {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeServer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                protocol_version: Some(6),
                controllers: Vec::new(),
                received: Vec::new(),
                connections: Vec::new(),
                silent_packets: HashSet::new(),
                reply_delays: HashMap::new(),
                session: 0,
                next_connection: 0,
            })),
        }
    }

    /// `None` makes the server ignore version requests, like a protocol 0 server.
    pub fn set_protocol_version(&self, version: Option<u32>) {
        self.lock().protocol_version = version;
    }

    pub fn add_device(&self, id: u32, device: FixtureDevice) {
        let label = device.label.clone();
        self.add_controller(id, &label, device.into());
    }

    pub fn add_controller(&self, id: u32, label: &str, block: ControllerBlock) {
        self.lock().controllers.push(Controller {
            id,
            label: label.into(),
            block,
        });
    }

    pub fn silence(&self, packet_id: u32) {
        self.lock().silent_packets.insert(packet_id);
    }

    pub fn delay_replies(&self, packet_id: u32, delay: Duration) {
        self.lock().reply_delays.insert(packet_id, delay);
    }

    pub fn connector(
        &self,
    ) -> impl Fn() -> Ready<io::Result<DuplexStream>> + Send + Sync + 'static {
        let server = self.clone();
        move || ready(Ok(server.connect()))
    }

    pub fn connect(&self) -> DuplexStream {
        let (client, server_side) = tokio::io::duplex(DUPLEX_BUFFER);
        let (reader, writer) = tokio::io::split(server_side);
        let (outgoing, queue) = unbounded_channel();
        let mut state = self.lock();
        let connection = state.next_connection;
        state.next_connection += 1;
        let session = state.session;
        let serve_task = tokio::spawn(serve(
            self.state.clone(),
            session,
            connection,
            reader,
            outgoing.clone(),
        ));
        let write_task = tokio::spawn(forward(queue, writer));
        state.connections.push(Connection {
            outgoing,
            tasks: [serve_task, write_task],
        });
        client
    }

    pub async fn serve_tcp(&self, listener: TcpListener) -> io::Result<()> {
        loop {
            let (mut socket, _) = listener.accept().await?;
            let mut connection = self.connect();
            tokio::spawn(async move {
                let _ = copy_bidirectional(&mut socket, &mut connection).await;
            });
        }
    }

    /// Drops every client and replaces the device list, as a restarted server would.
    pub fn restart(&self, controllers: Vec<(u32, ControllerBlock)>) {
        let mut state = self.lock();
        state.drop_connections();
        state.session += 1;
        state.controllers = controllers
            .into_iter()
            .map(|(id, block)| {
                let label = match &block {
                    ControllerBlock::Device(device) => device.label.clone(),
                    ControllerBlock::Raw(_) => format!("raw-{id}"),
                };
                Controller { id, label, block }
            })
            .collect();
    }

    pub fn disconnect_all(&self) {
        self.lock().drop_connections();
    }

    pub fn notify_device_list_updated(&self) {
        self.broadcast(wire::frame(0, wire::DEVICE_LIST_UPDATED, &[]));
    }

    pub fn broadcast(&self, bytes: Vec<u8>) {
        for connection in &self.lock().connections {
            let _ = connection.outgoing.send(bytes.clone());
        }
    }

    pub fn received(&self) -> Vec<ReceivedPacket> {
        self.lock().received.clone()
    }

    pub fn received_with_id(&self, packet_id: u32) -> Vec<ReceivedPacket> {
        self.received()
            .into_iter()
            .filter(|packet| packet.packet_id == packet_id)
            .collect()
    }

    pub fn color_writes(&self) -> Vec<ColorWrite> {
        self.received()
            .iter()
            .filter_map(ReceivedPacket::color_write)
            .collect()
    }

    /// Yields to the runtime until `count` colour packets have arrived; panics if they never do.
    pub async fn wait_for_color_writes(&self, count: usize) -> Vec<ColorWrite> {
        for _ in 0..WAIT_ROUNDS {
            let writes = self.color_writes();
            if writes.len() >= count {
                return writes;
            }
            tokio::task::yield_now().await;
        }
        panic!(
            "expected {count} colour writes, got {:?}",
            self.color_writes()
        );
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("fake server state poisoned")
    }
}

impl State {
    fn drop_connections(&mut self) {
        for connection in self.connections.drain(..) {
            for task in connection.tasks {
                task.abort();
            }
        }
    }

    fn controller(&self, id: u32) -> Option<&Controller> {
        self.controllers
            .iter()
            .find(|controller| controller.id == id)
    }

    fn respond(
        &self,
        negotiated: &mut u32,
        device_id: u32,
        packet_id: u32,
        payload: &[u8],
    ) -> Vec<Vec<u8>> {
        if self.silent_packets.contains(&packet_id) {
            return Vec::new();
        }
        let mut replies = Vec::new();
        let mut status = wire::STATUS_OK;
        match packet_id {
            wire::REQUEST_PROTOCOL_VERSION => {
                let Some(server_version) = self.protocol_version else {
                    return Vec::new();
                };
                let requested = u32_at(payload, 0).unwrap_or(0);
                *negotiated = requested.min(server_version);
                replies.push(wire::frame(0, packet_id, &server_version.to_le_bytes()));
                if *negotiated >= 6 {
                    let mut name = SERVER_NAME.as_bytes().to_vec();
                    name.push(0);
                    replies.push(wire::frame(0, wire::SET_SERVER_NAME, &name));
                }
            }
            wire::REQUEST_CONTROLLER_COUNT => {
                let mut body = Writer::default();
                body.u32(self.controllers.len() as u32);
                for controller in &self.controllers {
                    body.u32(controller.id);
                }
                replies.push(wire::frame(0, packet_id, &body.into_bytes()));
            }
            wire::REQUEST_CONTROLLER_DATA => {
                if let Some(controller) = self.controller(device_id) {
                    replies.push(wire::frame(
                        device_id,
                        packet_id,
                        &controller.block.encode(),
                    ));
                }
            }
            wire::UPDATE_LEDS | wire::UPDATE_ZONE_LEDS => match self.controller(device_id) {
                None => status = wire::STATUS_INVALID_ID,
                Some(_) if u32_at(payload, 0) != Some(payload.len() as u32) => {
                    status = wire::STATUS_INVALID_DATA;
                }
                Some(_) if packet_id == wire::UPDATE_LEDS => {
                    let mut echo = Writer::default();
                    echo.u32((payload.len() + 4) as u32).u32(0);
                    let mut body = echo.into_bytes();
                    body.extend_from_slice(&payload[4..]);
                    replies.push(wire::frame(device_id, wire::SIGNAL_UPDATE, &body));
                }
                Some(_) => {}
            },
            wire::SET_CUSTOM_MODE if self.controller(device_id).is_none() => {
                status = wire::STATUS_INVALID_ID;
            }
            _ => {}
        }
        if *negotiated >= 6 {
            replies.push(wire::ack(device_id, packet_id, status));
        }
        replies
    }
}

async fn serve(
    state: Arc<Mutex<State>>,
    session: u32,
    connection: usize,
    mut reader: ReadHalf<DuplexStream>,
    outgoing: UnboundedSender<Vec<u8>>,
) {
    let mut negotiated = 0;
    loop {
        let mut header = [0u8; wire::HEADER_LEN];
        if reader.read_exact(&mut header).await.is_err() || &header[..4] != wire::MAGIC {
            return;
        }
        let device_id = u32_at(&header, 4).unwrap_or_default();
        let packet_id = u32_at(&header, 8).unwrap_or_default();
        let size = u32_at(&header, 12).unwrap_or_default();
        if size > MAX_CLIENT_PACKET {
            return;
        }
        let mut payload = vec![0; size as usize];
        if reader.read_exact(&mut payload).await.is_err() {
            return;
        }
        let (replies, delay) = {
            let mut state = state.lock().expect("fake server state poisoned");
            let target = state
                .controller(device_id)
                .map(|controller| controller.label.clone());
            let replies = state.respond(&mut negotiated, device_id, packet_id, &payload);
            state.received.push(ReceivedPacket {
                session,
                connection,
                device_id,
                packet_id,
                payload,
                target,
            });
            (replies, state.reply_delays.get(&packet_id).copied())
        };
        if let Some(delay) = delay {
            tokio::time::sleep(delay).await;
        }
        for reply in replies {
            if outgoing.send(reply).is_err() {
                return;
            }
        }
    }
}

async fn forward(mut queue: UnboundedReceiver<Vec<u8>>, mut writer: WriteHalf<DuplexStream>) {
    while let Some(bytes) = queue.recv().await {
        if writer.write_all(&bytes).await.is_err() {
            return;
        }
    }
}
