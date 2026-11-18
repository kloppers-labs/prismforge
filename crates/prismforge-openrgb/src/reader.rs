use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::io::{AsyncRead, AsyncReadExt, ReadHalf};
use tokio::sync::{oneshot, watch};

use crate::client::ClientError;
use crate::protocol::{HEADER_LEN, PacketHeader, decode_ack, decode_string, packet_id};

/// At protocol 6 the server ACKs every packet after any reply, so an exchange that ends on its own ACK leaves none behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Completion {
    Payload,
    Ack,
}

pub(crate) struct Reply {
    pub payload: Option<Vec<u8>>,
    pub status: Option<u32>,
}

pub(crate) struct Pending {
    device_id: u32,
    packet_id: u32,
    completion: Completion,
    payload: Option<Vec<u8>>,
    reply: oneshot::Sender<Result<Reply, ClientError>>,
}

impl Pending {
    pub fn new(
        device_id: u32,
        packet_id: u32,
        completion: Completion,
        reply: oneshot::Sender<Result<Reply, ClientError>>,
    ) -> Self {
        Self {
            device_id,
            packet_id,
            completion,
            payload: None,
            reply,
        }
    }

    pub fn fail(self, error: ClientError) {
        let _ = self.reply.send(Err(error));
    }

    fn finish(self, status: Option<u32>) {
        let _ = self.reply.send(Ok(Reply {
            payload: self.payload,
            status,
        }));
    }
}

#[derive(Default)]
pub(crate) struct PendingSlot(Mutex<Option<Pending>>);

impl PendingSlot {
    pub fn set(&self, pending: Pending) {
        *self.lock() = Some(pending);
    }

    pub fn take(&self) -> Option<Pending> {
        self.lock().take()
    }

    fn on_payload(&self, device_id: u32, packet_id: u32, payload: Vec<u8>) {
        let mut slot = self.lock();
        let Some(pending) = slot
            .as_mut()
            .filter(|pending| pending.device_id == device_id && pending.packet_id == packet_id)
        else {
            return;
        };
        pending.payload.get_or_insert(payload);
        if pending.completion == Completion::Payload
            && let Some(pending) = slot.take()
        {
            pending.finish(None);
        }
    }

    fn on_ack(&self, device_id: u32, acked_packet_id: u32, status: u32) {
        let mut slot = self.lock();
        let matches = slot.as_ref().is_some_and(|pending| {
            pending.completion == Completion::Ack
                && pending.device_id == device_id
                && pending.packet_id == acked_packet_id
        });
        if matches && let Some(pending) = slot.take() {
            pending.finish(Some(status));
        }
    }

    fn lock(&self) -> MutexGuard<'_, Option<Pending>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ReaderStatus {
    pub list_epoch: u64,
    pub server_name: Option<String>,
    pub closed: Option<ClientError>,
}

/// Owns the read half: routes replies and ACKs to the pending request, coalesces notifications, drops the rest.
pub(crate) async fn run<S: AsyncRead>(
    mut reader: ReadHalf<S>,
    pending: Arc<PendingSlot>,
    status: watch::Sender<ReaderStatus>,
    max_packet_size: u32,
) {
    let error = loop {
        match read_packet(&mut reader, max_packet_size).await {
            Ok((header, payload)) => route(header, payload, &pending, &status),
            Err(error) => break error,
        }
    };
    // Publish closure before failing the pending request so a request registered concurrently sees one of them.
    status.send_modify(|status| status.closed = Some(error.clone()));
    if let Some(waiting) = pending.take() {
        waiting.fail(error);
    }
}

async fn read_packet<S: AsyncRead>(
    reader: &mut ReadHalf<S>,
    max_packet_size: u32,
) -> Result<(PacketHeader, Vec<u8>), ClientError> {
    let mut header = [0; HEADER_LEN];
    reader.read_exact(&mut header).await.map_err(io_failure)?;
    let header = PacketHeader::decode(&header, max_packet_size)?;
    let mut payload = vec![0; header.size as usize];
    reader.read_exact(&mut payload).await.map_err(io_failure)?;
    Ok((header, payload))
}

fn route(
    header: PacketHeader,
    payload: Vec<u8>,
    pending: &PendingSlot,
    status: &watch::Sender<ReaderStatus>,
) {
    match header.packet_id {
        packet_id::DEVICE_LIST_UPDATED => status.send_modify(|status| status.list_epoch += 1),
        packet_id::SET_SERVER_NAME => {
            status.send_modify(|status| status.server_name = decode_string(&payload));
        }
        packet_id::ACK => {
            if let Ok(ack) = decode_ack(&payload) {
                pending.on_ack(header.device_id, ack.packet_id, ack.status);
            }
        }
        other => pending.on_payload(header.device_id, other, payload),
    }
}

pub(crate) fn io_failure(error: io::Error) -> ClientError {
    if error.kind() == io::ErrorKind::UnexpectedEof {
        ClientError::Closed {
            detail: "server closed the connection".into(),
        }
    } else {
        ClientError::Io {
            detail: error.to_string(),
        }
    }
}
