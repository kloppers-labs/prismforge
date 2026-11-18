use crate::fixture::FixtureMode;
use crate::wire::{self, Reader, u16_at, u32_at};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedPacket {
    pub session: u32,
    pub connection: usize,
    pub device_id: u32,
    pub packet_id: u32,
    pub payload: Vec<u8>,
    /// Label of the fixture that owned `device_id` on the server when the packet arrived.
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorWrite {
    pub session: u32,
    pub device_id: u32,
    pub target: Option<String>,
    pub zone: Option<u32>,
    pub colors: Vec<[u8; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeUpdate {
    pub session: u32,
    pub device_id: u32,
    pub target: Option<String>,
    pub mode_index: i32,
    pub mode: FixtureMode,
}

impl ReceivedPacket {
    /// Decodes a protocol 6 UpdateMode packet; panics on a malformed one.
    pub fn mode_update(&self) -> Option<ModeUpdate> {
        if self.packet_id != wire::UPDATE_MODE {
            return None;
        }
        let mut input = Reader::new(&self.payload);
        assert_eq!(
            input.u32() as usize,
            self.payload.len(),
            "mode packet data size does not match packet size"
        );
        let mode_index = input.i32();
        let mode = FixtureMode::decode(&mut input);
        assert_eq!(input.remaining(), 0, "bytes after mode data");
        Some(ModeUpdate {
            session: self.session,
            device_id: self.device_id,
            target: self.target.clone(),
            mode_index,
            mode,
        })
    }

    /// Decodes UpdateLEDs / UpdateZoneLEDs; panics on a malformed colour packet.
    pub fn color_write(&self) -> Option<ColorWrite> {
        let (zone, count_offset) = match self.packet_id {
            wire::UPDATE_LEDS => (None, 4),
            wire::UPDATE_ZONE_LEDS => (u32_at(&self.payload, 4), 8),
            _ => return None,
        };
        let declared = u32_at(&self.payload, 0).expect("colour packet without data size");
        assert_eq!(
            declared as usize,
            self.payload.len(),
            "colour packet data size does not match packet size"
        );
        let count = u16_at(&self.payload, count_offset).expect("colour packet without count");
        let colors_start = count_offset + 2;
        assert_eq!(
            self.payload.len(),
            colors_start + count as usize * 4,
            "colour packet length does not match colour count"
        );
        let colors = self.payload[colors_start..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect();
        Some(ColorWrite {
            session: self.session,
            device_id: self.device_id,
            target: self.target.clone(),
            zone,
            colors,
        })
    }
}
