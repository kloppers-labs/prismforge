use crate::wire::{Reader, Writer};

const FLAG_HAS_SPEED: u32 = 1 << 0;
const FLAG_HAS_DIRECTION_LR: u32 = 1 << 1;
const FLAG_HAS_BRIGHTNESS: u32 = 1 << 4;
const FLAG_HAS_PER_LED_COLOR: u32 = 1 << 5;
const FLAG_HAS_MODE_SPECIFIC_COLOR: u32 = 1 << 6;
const FLAG_MANUAL_SAVE: u32 = 1 << 8;
const COLORS_PER_LED: u32 = 1;
const COLORS_MODE_SPECIFIC: u32 = 2;
const ZONE_TYPE_LINEAR: i32 = 1;
const ZONE_TYPE_MATRIX: i32 = 2;
const DEVICE_TYPE_LEDSTRIP: i32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerBlock {
    Device(Box<FixtureDevice>),
    Raw(Vec<u8>),
}

impl From<FixtureDevice> for ControllerBlock {
    fn from(device: FixtureDevice) -> Self {
        Self::Device(Box::new(device))
    }
}

impl ControllerBlock {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Device(device) => device.encode(),
            Self::Raw(bytes) => bytes.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureMode {
    pub name: String,
    pub flags: u32,
    pub speed_min: u32,
    pub speed_max: u32,
    pub brightness_min: u32,
    pub brightness_max: u32,
    pub colors_min: u32,
    pub colors_max: u32,
    pub speed: u32,
    pub brightness: u32,
    pub direction: u32,
    pub color_mode: u32,
    pub colors: Vec<[u8; 3]>,
}

impl FixtureMode {
    fn plain(name: &str, flags: u32, color_mode: u32) -> Self {
        Self {
            name: name.into(),
            flags,
            speed_min: 0,
            speed_max: 0,
            brightness_min: 0,
            brightness_max: 0,
            colors_min: 0,
            colors_max: 0,
            speed: 0,
            brightness: 0,
            direction: 0,
            color_mode,
            colors: Vec::new(),
        }
    }

    pub fn direct() -> Self {
        Self::plain("Direct", FLAG_HAS_PER_LED_COLOR, COLORS_PER_LED)
    }

    pub fn static_color() -> Self {
        Self {
            colors_min: 1,
            colors_max: 1,
            colors: vec![[255, 0, 0]],
            ..Self::plain(
                "Static",
                FLAG_HAS_MODE_SPECIFIC_COLOR | FLAG_MANUAL_SAVE,
                COLORS_MODE_SPECIFIC,
            )
        }
    }

    pub fn breathing() -> Self {
        Self {
            speed_min: 1,
            speed_max: 5,
            speed: 3,
            brightness_min: 0,
            brightness_max: 100,
            brightness: 100,
            colors_min: 1,
            colors_max: 2,
            colors: vec![[0, 0, 255]],
            ..Self::plain(
                "Breathing",
                FLAG_HAS_SPEED
                    | FLAG_HAS_DIRECTION_LR
                    | FLAG_HAS_BRIGHTNESS
                    | FLAG_HAS_MODE_SPECIFIC_COLOR
                    | FLAG_MANUAL_SAVE,
                COLORS_MODE_SPECIFIC,
            )
        }
    }

    pub fn decode(input: &mut Reader) -> Self {
        let mut mode = Self {
            name: input.string(),
            flags: input.u32(),
            speed_min: input.u32(),
            speed_max: input.u32(),
            brightness_min: input.u32(),
            brightness_max: input.u32(),
            colors_min: input.u32(),
            colors_max: input.u32(),
            speed: input.u32(),
            brightness: input.u32(),
            direction: input.u32(),
            color_mode: input.u32(),
            colors: Vec::new(),
        };
        let count = input.u16();
        mode.colors = (0..count).map(|_| input.color()).collect();
        mode
    }

    fn encode(&self, out: &mut Writer) {
        out.string(&self.name)
            .u32(self.flags)
            .u32(self.speed_min)
            .u32(self.speed_max)
            .u32(self.brightness_min)
            .u32(self.brightness_max)
            .u32(self.colors_min)
            .u32(self.colors_max)
            .u32(self.speed)
            .u32(self.brightness)
            .u32(self.direction)
            .u32(self.color_mode)
            .u16(self.colors.len() as u16);
        for color in &self.colors {
            out.color(*color);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureZone {
    pub name: String,
    pub zone_type: i32,
    pub leds_min: u32,
    pub leds_max: u32,
    pub leds_count: u32,
    pub matrix: Option<(u32, u32, Vec<u32>)>,
}

impl FixtureZone {
    fn encode(&self, out: &mut Writer) {
        out.string(&self.name)
            .i32(self.zone_type)
            .u32(self.leds_min)
            .u32(self.leds_max)
            .u32(self.leds_count);
        match &self.matrix {
            Some((height, width, map)) => {
                out.u16((8 + map.len() * 4) as u16).u32(*height).u32(*width);
                for entry in map {
                    out.u32(*entry);
                }
            }
            None => {
                out.u16(0);
            }
        }
        out.u16(0).u32(0).i32(-1).u16(0).string("");
    }
}

/// `label` names the physical fixture in recordings; it is never sent on the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureDevice {
    pub label: String,
    pub device_type: i32,
    pub name: String,
    pub vendor: String,
    pub description: String,
    pub version: String,
    pub serial: String,
    pub location: String,
    pub modes: Vec<FixtureMode>,
    pub active_mode: i32,
    pub zones: Vec<FixtureZone>,
    pub led_colors: Vec<[u8; 3]>,
    pub led_count_override: Option<u16>,
}

impl FixtureDevice {
    pub fn new(label: &str, name: &str) -> Self {
        Self {
            label: label.into(),
            device_type: DEVICE_TYPE_LEDSTRIP,
            name: name.into(),
            vendor: "Fixture Vendor".into(),
            description: "Fixture device".into(),
            version: "1.0".into(),
            serial: String::new(),
            location: String::new(),
            modes: vec![FixtureMode::direct(), FixtureMode::static_color()],
            active_mode: 0,
            zones: Vec::new(),
            led_colors: Vec::new(),
            led_count_override: None,
        }
    }

    pub fn serial(mut self, serial: &str) -> Self {
        self.serial = serial.into();
        self
    }

    pub fn location(mut self, location: &str) -> Self {
        self.location = location.into();
        self
    }

    pub fn zone(self, name: &str, leds: u32) -> Self {
        self.resizable_zone(name, leds, leds, leds)
    }

    pub fn resizable_zone(mut self, name: &str, min: u32, max: u32, count: u32) -> Self {
        self.zones.push(FixtureZone {
            name: name.into(),
            zone_type: ZONE_TYPE_LINEAR,
            leds_min: min,
            leds_max: max,
            leds_count: count,
            matrix: None,
        });
        self
    }

    pub fn matrix_zone(mut self, name: &str, height: u32, width: u32) -> Self {
        let count = height * width;
        self.zones.push(FixtureZone {
            name: name.into(),
            zone_type: ZONE_TYPE_MATRIX,
            leds_min: count,
            leds_max: count,
            leds_count: count,
            matrix: Some((height, width, (0..count).collect())),
        });
        self
    }

    pub fn modes(mut self, modes: Vec<FixtureMode>) -> Self {
        self.modes = modes;
        self
    }

    pub fn active_mode(mut self, index: i32) -> Self {
        self.active_mode = index;
        self
    }

    /// LEDs past the end of `colors` report black.
    pub fn led_colors(mut self, colors: Vec<[u8; 3]>) -> Self {
        self.led_colors = colors;
        self
    }

    pub fn without_direct_mode(self) -> Self {
        self.modes(vec![FixtureMode::static_color(), FixtureMode::breathing()])
    }

    pub fn led_count(&self) -> usize {
        self.zones.iter().map(|zone| zone.leds_count as usize).sum()
    }

    /// Payload of a protocol 6 controller data reply, including its leading size field.
    pub fn encode(&self) -> Vec<u8> {
        let mut body = Writer::default();
        body.i32(self.device_type)
            .string(&self.name)
            .string(&self.vendor)
            .string(&self.description)
            .string(&self.version)
            .string(&self.serial)
            .string(&self.location)
            .u16(self.modes.len() as u16)
            .i32(self.active_mode);
        for mode in &self.modes {
            mode.encode(&mut body);
        }
        body.u16(self.zones.len() as u16);
        for zone in &self.zones {
            zone.encode(&mut body);
        }
        let leds = self.led_count_override.unwrap_or(self.led_count() as u16);
        body.u16(leds);
        for index in 0..leds {
            body.string(&format!("LED {index}"));
        }
        body.u16(leds);
        for index in 0..leds as usize {
            body.color(self.led_colors.get(index).copied().unwrap_or([0, 0, 0]));
        }
        body.u16(0).u32(0).string(&self.name).u32(1).byte(0);
        let body = body.into_bytes();
        let mut payload = ((body.len() + 4) as u32).to_le_bytes().to_vec();
        payload.extend_from_slice(&body);
        payload
    }
}
