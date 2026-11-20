use std::sync::Arc;
use std::time::Duration;

use prismforge_core::Rgb;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, WriteHalf};
use tokio::sync::{oneshot, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use prismforge_core::automation::EventProfiles;

use crate::protocol::{
    ACK_STATUS_OK, ControllerData, ControllerDataError, FrameError, MAX_PACKET_SIZE, ModeData,
    PROFILE_MANAGER_KEY, decode_controller_ids, decode_event_profiles, decode_server_flags,
    decode_u32, encode_packet, encode_settings_key, encode_string, encode_u32, encode_update_leds,
    encode_update_mode, encode_update_zone_leds, packet_id, parse_controller_data,
};
use crate::reader::{self, Completion, Pending, PendingSlot, ReaderStatus, Reply, io_failure};

pub const CLIENT_PROTOCOL_VERSION: u32 = 6;
pub const MIN_PROTOCOL_VERSION: u32 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientConfig {
    pub client_name: String,
    pub request_timeout: Duration,
    pub max_packet_size: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            client_name: "PrismForge".into(),
            request_timeout: Duration::from_secs(2),
            max_packet_size: MAX_PACKET_SIZE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClientError {
    #[error("i/o failure: {detail}")]
    Io { detail: String },
    #[error("connection closed: {detail}")]
    Closed { detail: String },
    #[error("an earlier request was abandoned mid-exchange")]
    Abandoned,
    #[error("{operation} timed out after {after:?}")]
    Timeout {
        operation: &'static str,
        after: Duration,
    },
    #[error(transparent)]
    Frame(#[from] FrameError),
    #[error("server protocol {server_version:?} is not supported")]
    UnsupportedProtocol { server_version: Option<u32> },
    #[error("malformed reply to packet {packet_id}: {detail}")]
    MalformedReply { packet_id: u32, detail: String },
    #[error("controller {controller_id} sent invalid data: {error}")]
    ControllerData {
        controller_id: u32,
        error: ControllerDataError,
    },
    #[error("controller {controller_id} is not on the server")]
    ControllerNotFound { controller_id: u32 },
    #[error("server rejected packet {packet_id} for device {device_id} with status {status}")]
    Rejected {
        packet_id: u32,
        device_id: u32,
        status: u32,
    },
    #[error("{count} colours exceed the protocol limit")]
    TooManyColors { count: usize },
    #[error("mode {mode_index} does not fit an UpdateMode packet")]
    UnencodableMode { mode_index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    DeviceListUpdated,
    Closed(ClientError),
}

pub struct OpenRgbClient<S> {
    writer: WriteHalf<S>,
    pending: Arc<PendingSlot>,
    status: watch::Receiver<ReaderStatus>,
    reader: JoinHandle<()>,
    config: ClientConfig,
    protocol_version: u32,
    seen_list_epoch: u64,
    failure: Option<ClientError>,
    exchange_open: bool,
}

impl<S> OpenRgbClient<S>
where
    S: AsyncRead + AsyncWrite + Send + 'static,
{
    /// Negotiates protocol 6 (or the server's lower version, which is refused) and names the client.
    pub async fn connect(stream: S, config: ClientConfig) -> Result<Self, ClientError> {
        let (read_half, writer) = tokio::io::split(stream);
        let pending = Arc::new(PendingSlot::default());
        let (status_sender, status) = watch::channel(ReaderStatus::default());
        let reader = tokio::spawn(reader::run(
            read_half,
            pending.clone(),
            status_sender,
            config.max_packet_size,
        ));
        let mut client = Self {
            writer,
            pending,
            status,
            reader,
            config,
            protocol_version: 0,
            seen_list_epoch: 0,
            failure: None,
            exchange_open: false,
        };
        let server_version = client.request_server_version().await?;
        client.protocol_version = server_version.min(CLIENT_PROTOCOL_VERSION);
        if client.protocol_version < MIN_PROTOCOL_VERSION {
            return Err(ClientError::UnsupportedProtocol {
                server_version: Some(server_version),
            });
        }
        let name = encode_string(&client.config.client_name);
        client
            .send(0, packet_id::SET_CLIENT_NAME, &name, "client name")
            .await?;
        Ok(client)
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn server_name(&self) -> Option<String> {
        self.status.borrow().server_name.clone()
    }

    pub async fn controller_ids(&mut self) -> Result<Vec<u32>, ClientError> {
        let packet = packet_id::REQUEST_CONTROLLER_COUNT;
        let reply = self
            .request(0, packet, &[], Completion::Ack, "controller count request")
            .await?;
        let payload = self.expect_payload(packet, reply)?;
        decode_controller_ids(&payload).map_err(|error| self.malformed(packet, error))
    }

    pub async fn controller_data(&mut self, id: u32) -> Result<ControllerData, ClientError> {
        let packet = packet_id::REQUEST_CONTROLLER_DATA;
        let version = encode_u32(self.protocol_version);
        let reply = self
            .request(
                id,
                packet,
                &version,
                Completion::Ack,
                "controller data request",
            )
            .await?;
        let payload = reply
            .payload
            .ok_or(ClientError::ControllerNotFound { controller_id: id })?;
        parse_controller_data(&payload).map_err(|error| ClientError::ControllerData {
            controller_id: id,
            error,
        })
    }

    pub async fn set_custom_mode(&mut self, id: u32) -> Result<(), ClientError> {
        self.command(id, packet_id::SET_CUSTOM_MODE, &[], "custom mode request")
            .await
    }

    pub async fn update_mode(
        &mut self,
        id: u32,
        mode_index: usize,
        mode: &ModeData,
    ) -> Result<(), ClientError> {
        let payload = u16::try_from(mode_index)
            .ok()
            .and_then(|index| encode_update_mode(index, mode))
            .ok_or(ClientError::UnencodableMode { mode_index })?;
        self.command(id, packet_id::UPDATE_MODE, &payload, "update mode request")
            .await
    }

    pub async fn update_leds(&mut self, id: u32, colors: &[Rgb]) -> Result<(), ClientError> {
        let payload = encode_update_leds(colors).ok_or(ClientError::TooManyColors {
            count: colors.len(),
        })?;
        self.send(id, packet_id::UPDATE_LEDS, &payload, "colour update")
            .await
    }

    pub async fn update_zone_leds(
        &mut self,
        id: u32,
        zone: u32,
        colors: &[Rgb],
    ) -> Result<(), ClientError> {
        let payload = encode_update_zone_leds(zone, colors).ok_or(ClientError::TooManyColors {
            count: colors.len(),
        })?;
        self.send(
            id,
            packet_id::UPDATE_ZONE_LEDS,
            &payload,
            "zone colour update",
        )
        .await
    }

    /// Sends empty client flags, which the server answers with its own; PrismForge asks for no role.
    pub async fn server_flags(&mut self) -> Result<u32, ClientError> {
        let packet = packet_id::SET_CLIENT_FLAGS;
        let reply = self
            .request_answered_by(
                0,
                (packet, packet_id::SET_SERVER_FLAGS),
                &encode_u32(0),
                "client flags",
            )
            .await?;
        match reply.payload {
            None => Ok(0),
            Some(payload) => decode_server_flags(&payload).map_err(|error| readable(packet, error)),
        }
    }

    /// Reads OpenRGB's `ProfileManager` settings; nothing here can change them.
    pub async fn event_profiles(&mut self) -> Result<EventProfiles, ClientError> {
        let packet = packet_id::SETTINGS_GET;
        let key = encode_settings_key(PROFILE_MANAGER_KEY);
        let reply = self
            .request(0, packet, &key, Completion::Ack, "settings request")
            .await?;
        match (reply.payload, reply.status) {
            (Some(payload), _) => {
                decode_event_profiles(&payload).map_err(|error| readable(packet, error))
            }
            (None, Some(status)) if status != ACK_STATUS_OK => Err(ClientError::Rejected {
                packet_id: packet,
                device_id: 0,
                status,
            }),
            (None, _) => Err(readable(packet, "acknowledged without a reply")),
        }
    }

    pub fn poll_notification(&mut self) -> Option<Notification> {
        if let Some(failure) = &self.failure {
            return Some(Notification::Closed(failure.clone()));
        }
        let status = self.status.borrow();
        if status.list_epoch > self.seen_list_epoch {
            self.seen_list_epoch = status.list_epoch;
            return Some(Notification::DeviceListUpdated);
        }
        status.closed.clone().map(Notification::Closed)
    }

    pub async fn next_notification(&mut self) -> Notification {
        loop {
            if let Some(notification) = self.poll_notification() {
                return notification;
            }
            if self.status.changed().await.is_err() {
                return Notification::Closed(ClientError::Closed {
                    detail: "reader stopped".into(),
                });
            }
        }
    }

    async fn request_server_version(&mut self) -> Result<u32, ClientError> {
        let packet = packet_id::REQUEST_PROTOCOL_VERSION;
        let offer = encode_u32(CLIENT_PROTOCOL_VERSION);
        match self
            .request(
                0,
                packet,
                &offer,
                Completion::Payload,
                "protocol version request",
            )
            .await
        {
            Ok(reply) => {
                let payload = self.expect_payload(packet, reply)?;
                decode_u32(&payload).map_err(|error| self.malformed(packet, error))
            }
            Err(ClientError::Timeout { .. }) => Err(ClientError::UnsupportedProtocol {
                server_version: None,
            }),
            Err(error) => Err(error),
        }
    }

    async fn command(
        &mut self,
        id: u32,
        packet: u32,
        payload: &[u8],
        operation: &'static str,
    ) -> Result<(), ClientError> {
        let reply = self
            .request(id, packet, payload, Completion::Ack, operation)
            .await?;
        match (reply.payload, reply.status) {
            (None, Some(ACK_STATUS_OK)) => Ok(()),
            (None, Some(status)) => Err(ClientError::Rejected {
                packet_id: packet,
                device_id: id,
                status,
            }),
            _ => Err(self.malformed(packet, "unexpected reply payload")),
        }
    }

    async fn request(
        &mut self,
        device_id: u32,
        packet_id: u32,
        payload: &[u8],
        completion: Completion,
        operation: &'static str,
    ) -> Result<Reply, ClientError> {
        self.exchange(
            device_id,
            (packet_id, packet_id),
            payload,
            completion,
            operation,
        )
        .await
    }

    async fn request_answered_by(
        &mut self,
        device_id: u32,
        packets: (u32, u32),
        payload: &[u8],
        operation: &'static str,
    ) -> Result<Reply, ClientError> {
        self.exchange(device_id, packets, payload, Completion::Ack, operation)
            .await
    }

    async fn exchange(
        &mut self,
        device_id: u32,
        (packet_id, reply_packet_id): (u32, u32),
        payload: &[u8],
        completion: Completion,
        operation: &'static str,
    ) -> Result<Reply, ClientError> {
        self.begin_exchange()?;
        let (reply, response) = oneshot::channel();
        self.pending.set(Pending::new(
            device_id,
            (packet_id, reply_packet_id),
            completion,
            reply,
        ));
        if let Err(error) = self.ensure_open() {
            self.pending.take();
            self.exchange_open = false;
            return Err(error);
        }
        let packet = encode_packet(device_id, packet_id, payload);
        let writer = &mut self.writer;
        let exchange = async move {
            writer.write_all(&packet).await.map_err(io_failure)?;
            response.await.unwrap_or_else(|_| {
                Err(ClientError::Closed {
                    detail: "reader stopped".into(),
                })
            })
        };
        let outcome = timeout(self.config.request_timeout, exchange).await;
        self.pending.take();
        self.exchange_open = false;
        match outcome {
            Ok(Ok(reply)) => Ok(reply),
            Ok(Err(error)) => Err(self.fail(error)),
            Err(_) => Err(self.fail(ClientError::Timeout {
                operation,
                after: self.config.request_timeout,
            })),
        }
    }

    async fn send(
        &mut self,
        device_id: u32,
        packet_id: u32,
        payload: &[u8],
        operation: &'static str,
    ) -> Result<(), ClientError> {
        self.begin_exchange()?;
        let packet = encode_packet(device_id, packet_id, payload);
        let outcome = timeout(self.config.request_timeout, self.writer.write_all(&packet)).await;
        self.exchange_open = false;
        match outcome {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(self.fail(io_failure(error))),
            Err(_) => Err(self.fail(ClientError::Timeout {
                operation,
                after: self.config.request_timeout,
            })),
        }
    }

    fn expect_payload(&mut self, packet_id: u32, reply: Reply) -> Result<Vec<u8>, ClientError> {
        match reply.payload {
            Some(payload) => Ok(payload),
            None => Err(self.malformed(packet_id, "acknowledged without a reply")),
        }
    }

    /// A dropped request future may leave its reply or ACK unread, so the next exchange closes the connection.
    fn begin_exchange(&mut self) -> Result<(), ClientError> {
        self.ensure_open()?;
        if self.exchange_open {
            return Err(self.fail(ClientError::Abandoned));
        }
        self.exchange_open = true;
        Ok(())
    }

    fn ensure_open(&self) -> Result<(), ClientError> {
        match (&self.failure, &self.status.borrow().closed) {
            (Some(error), _) | (None, Some(error)) => Err(error.clone()),
            (None, None) => Ok(()),
        }
    }

    fn malformed(&mut self, packet_id: u32, detail: impl ToString) -> ClientError {
        self.fail(ClientError::MalformedReply {
            packet_id,
            detail: detail.to_string(),
        })
    }

    /// Any transport or framing fault leaves the stream position unknown, so the connection is closed.
    fn fail(&mut self, error: ClientError) -> ClientError {
        self.reader.abort();
        self.failure.get_or_insert(error).clone()
    }
}

/// A reply that was framed correctly but cannot be read; the connection itself stays in step.
fn readable(packet_id: u32, detail: impl ToString) -> ClientError {
    ClientError::MalformedReply {
        packet_id,
        detail: detail.to_string(),
    }
}

impl<S> Drop for OpenRgbClient<S> {
    fn drop(&mut self) {
        self.reader.abort();
    }
}
