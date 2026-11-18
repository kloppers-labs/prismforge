pub const MAGIC: &[u8; 4] = b"ORGB";
pub const HEADER_LEN: usize = 16;

pub const REQUEST_CONTROLLER_COUNT: u32 = 0;
pub const REQUEST_CONTROLLER_DATA: u32 = 1;
pub const ACK: u32 = 10;
pub const REQUEST_PROTOCOL_VERSION: u32 = 40;
pub const SET_CLIENT_NAME: u32 = 50;
pub const SET_SERVER_NAME: u32 = 51;
pub const DEVICE_LIST_UPDATED: u32 = 100;
pub const DETECTION_PROGRESS_CHANGED: u32 = 102;
pub const UPDATE_LEDS: u32 = 1050;
pub const UPDATE_ZONE_LEDS: u32 = 1051;
pub const SET_CUSTOM_MODE: u32 = 1100;
pub const SIGNAL_UPDATE: u32 = 1150;

pub const STATUS_OK: u32 = 0;
pub const STATUS_INVALID_ID: u32 = 4;
pub const STATUS_INVALID_DATA: u32 = 5;

pub fn frame(device_id: u32, packet_id: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = header(device_id, packet_id, payload.len() as u32);
    out.extend_from_slice(payload);
    out
}

pub fn header(device_id: u32, packet_id: u32, size: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&device_id.to_le_bytes());
    out.extend_from_slice(&packet_id.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out
}

pub fn ack(device_id: u32, acked_packet_id: u32, status: u32) -> Vec<u8> {
    let mut payload = acked_packet_id.to_le_bytes().to_vec();
    payload.extend_from_slice(&status.to_le_bytes());
    frame(device_id, ACK, &payload)
}

#[derive(Default)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn byte(&mut self, value: u8) -> &mut Self {
        self.bytes.push(value);
        self
    }

    pub fn u16(&mut self, value: u16) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn string(&mut self, value: &str) -> &mut Self {
        self.u16(value.len() as u16 + 1);
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
        self
    }

    pub fn color(&mut self, [r, g, b]: [u8; 3]) -> &mut Self {
        self.bytes.extend_from_slice(&[r, g, b, 0]);
        self
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

pub fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes(slice.try_into().ok()?))
}
