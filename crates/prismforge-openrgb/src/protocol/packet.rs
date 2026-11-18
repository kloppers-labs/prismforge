use prismforge_core::Rgb;

use super::header::PacketHeader;

pub mod packet_id {
    pub const REQUEST_CONTROLLER_COUNT: u32 = 0;
    pub const REQUEST_CONTROLLER_DATA: u32 = 1;
    pub const ACK: u32 = 10;
    pub const REQUEST_PROTOCOL_VERSION: u32 = 40;
    pub const SET_CLIENT_NAME: u32 = 50;
    pub const SET_SERVER_NAME: u32 = 51;
    pub const DEVICE_LIST_UPDATED: u32 = 100;
    pub const UPDATE_LEDS: u32 = 1050;
    pub const UPDATE_ZONE_LEDS: u32 = 1051;
    pub const SET_CUSTOM_MODE: u32 = 1100;
}

pub const ACK_STATUS_OK: u32 = 0;

const SIZE_FIELD_LEN: usize = 4;
const COUNT_FIELD_LEN: usize = 2;
const COLOR_LEN: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayloadError {
    #[error("payload of {actual} bytes does not match expected {expected} bytes")]
    Length { expected: usize, actual: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ack {
    pub packet_id: u32,
    pub status: u32,
}

pub fn encode_packet(device_id: u32, packet_id: u32, payload: &[u8]) -> Vec<u8> {
    let header = PacketHeader {
        device_id,
        packet_id,
        size: payload.len() as u32,
    };
    let mut packet = header.encode().to_vec();
    packet.extend_from_slice(payload);
    packet
}

pub fn encode_u32(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub fn encode_string(value: &str) -> Vec<u8> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

/// `None` when there are more colours than the protocol's 16-bit count can carry.
pub fn encode_update_leds(colors: &[Rgb]) -> Option<Vec<u8>> {
    encode_colors(&[], colors)
}

pub fn encode_update_zone_leds(zone: u32, colors: &[Rgb]) -> Option<Vec<u8>> {
    encode_colors(&zone.to_le_bytes(), colors)
}

/// Colour payloads carry their own total size, which the server checks against the packet size.
fn encode_colors(prefix: &[u8], colors: &[Rgb]) -> Option<Vec<u8>> {
    let count = u16::try_from(colors.len()).ok()?;
    let size = SIZE_FIELD_LEN + prefix.len() + COUNT_FIELD_LEN + colors.len() * COLOR_LEN;
    let mut payload = Vec::with_capacity(size);
    payload.extend_from_slice(&(size as u32).to_le_bytes());
    payload.extend_from_slice(prefix);
    payload.extend_from_slice(&count.to_le_bytes());
    for color in colors {
        payload.extend_from_slice(&[color.r, color.g, color.b, 0]);
    }
    Some(payload)
}

/// `None` unless the payload is UTF-8 text ending in exactly one NUL.
pub fn decode_string(payload: &[u8]) -> Option<String> {
    let text = payload.strip_suffix(&[0])?;
    if text.contains(&0) {
        return None;
    }
    String::from_utf8(text.to_vec()).ok()
}

pub fn decode_u32(payload: &[u8]) -> Result<u32, PayloadError> {
    let bytes: [u8; 4] = payload.try_into().map_err(|_| PayloadError::Length {
        expected: 4,
        actual: payload.len(),
    })?;
    Ok(u32::from_le_bytes(bytes))
}

pub fn decode_controller_ids(payload: &[u8]) -> Result<Vec<u32>, PayloadError> {
    let (count, ids) = payload
        .split_at_checked(SIZE_FIELD_LEN)
        .ok_or(PayloadError::Length {
            expected: SIZE_FIELD_LEN,
            actual: payload.len(),
        })?;
    let count = decode_u32(count)? as usize;
    if count.checked_mul(4) != Some(ids.len()) {
        return Err(PayloadError::Length {
            expected: count.saturating_mul(4).saturating_add(SIZE_FIELD_LEN),
            actual: payload.len(),
        });
    }
    Ok(ids
        .as_chunks::<4>()
        .0
        .iter()
        .map(|id| u32::from_le_bytes(*id))
        .collect())
}

pub fn decode_ack(payload: &[u8]) -> Result<Ack, PayloadError> {
    if payload.len() != 8 {
        return Err(PayloadError::Length {
            expected: 8,
            actual: payload.len(),
        });
    }
    Ok(Ack {
        packet_id: decode_u32(&payload[..4])?,
        status: decode_u32(&payload[4..])?,
    })
}
