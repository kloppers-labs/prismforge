use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::io::{AsyncRead, AsyncReadExt, ReadHalf};
use tokio::sync::{oneshot, watch};

use crate::client::ClientError;
use crate::protocol::{HEADER_LEN, PacketHeader, decode_ack, packet_id};

pub(crate) enum Reply {
    Payload(Vec<u8>),
    Ack(u32),
}

pub(crate) struct Pending {
    pub device_id: u32,
    pub packet_id: u32,
    pub reply: oneshot::Sender<Result<Reply, ClientError>>,
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

    fn take_matching(&self, device_id: u32, packet_id: u32) -> Option<Pending> {
        let mut slot = self.lock();
        let matches = slot.as_ref().is_some_and(|pending| {
            pending.device_id == device_id && pending.packet_id == packet_id
        });
        if matches { slot.take() } else { None }
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
        let _ = waiting.reply.send(Err(error));
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
        packet_id::ACK => {
            if let Ok(ack) = decode_ack(&payload) {
                deliver(
                    pending,
                    header.device_id,
                    ack.packet_id,
                    Reply::Ack(ack.status),
                );
            }
        }
        other => deliver(pending, header.device_id, other, Reply::Payload(payload)),
    }
}

fn deliver(pending: &PendingSlot, device_id: u32, packet_id: u32, reply: Reply) {
    if let Some(waiting) = pending.take_matching(device_id, packet_id) {
        let _ = waiting.reply.send(Ok(reply));
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
