pub const HEADER_LEN: usize = 16;
pub const MAGIC: [u8; 4] = *b"ORGB";
/// Matches the OpenRGB SDK's own maximum packet size (8 MiB).
pub const MAX_PACKET_SIZE: u32 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub device_id: u32,
    pub packet_id: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FrameError {
    #[error("packet magic {0:?} is not ORGB")]
    BadMagic([u8; 4]),
    #[error("packet of {size} bytes exceeds the {limit} byte limit")]
    TooLarge { size: u32, limit: u32 },
}

impl PacketHeader {
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut bytes = [0; HEADER_LEN];
        bytes[..4].copy_from_slice(&MAGIC);
        bytes[4..8].copy_from_slice(&self.device_id.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.packet_id.to_le_bytes());
        bytes[12..].copy_from_slice(&self.size.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8; HEADER_LEN], limit: u32) -> Result<Self, FrameError> {
        let word = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if magic != MAGIC {
            return Err(FrameError::BadMagic(magic));
        }
        let header = Self {
            device_id: word(4),
            packet_id: word(8),
            size: word(12),
        };
        if header.size > limit {
            return Err(FrameError::TooLarge {
                size: header.size,
                limit,
            });
        }
        Ok(header)
    }
}
