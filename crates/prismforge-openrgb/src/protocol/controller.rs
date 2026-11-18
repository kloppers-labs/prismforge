use prismforge_core::Rgb;

use super::cursor::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerData {
    pub device_type: i32,
    pub name: String,
    pub vendor: String,
    pub description: String,
    pub version: String,
    pub serial: String,
    pub location: String,
    pub active_mode: i32,
    pub modes: Vec<ModeData>,
    pub zones: Vec<ZoneData>,
    pub led_names: Vec<String>,
    pub colors: Vec<Rgb>,
    pub led_display_names: Vec<String>,
    pub flags: u32,
    pub display_name: String,
    pub configuration: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeData {
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
    pub colors: Vec<Rgb>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneData {
    pub name: String,
    pub zone_type: i32,
    pub leds_min: u32,
    pub leds_max: u32,
    pub leds_count: u32,
    pub matrix: Option<MatrixMap>,
    pub segments: Vec<SegmentData>,
    pub flags: u32,
    pub active_mode: i32,
    pub modes: Vec<ModeData>,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentData {
    pub name: String,
    pub segment_type: i32,
    pub start: u32,
    pub leds_count: u32,
    pub matrix: Option<MatrixMap>,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixMap {
    pub height: u32,
    pub width: u32,
    pub cells: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringProblem {
    Empty,
    Unterminated,
    InteriorNul,
    InvalidUtf8,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ControllerDataError {
    #[error("controller data ends inside {field}")]
    Truncated { field: &'static str },
    #[error("declared data size {declared} does not match the {actual} byte payload")]
    SizeMismatch { declared: u32, actual: usize },
    #[error("{count} unexpected bytes after controller data")]
    TrailingBytes { count: usize },
    #[error("string {field} is invalid: {problem:?}")]
    InvalidString {
        field: &'static str,
        problem: StringProblem,
    },
    #[error("matrix map length {declared} does not fit a {height}x{width} map")]
    MatrixSizeMismatch {
        declared: u16,
        height: u32,
        width: u32,
    },
    #[error("active mode {active} is outside {modes} modes")]
    ActiveModeOutOfRange { active: i32, modes: usize },
    #[error("zone {zone} active mode {active} is outside {modes} zone modes")]
    ZoneActiveModeOutOfRange {
        zone: usize,
        active: i32,
        modes: usize,
    },
    #[error("zone {zone} minimum {min} exceeds maximum {max}")]
    ZoneBoundsInverted { zone: usize, min: u32, max: u32 },
    #[error("zones hold {zone_leds} LEDs but the LED list has {leds}")]
    ZoneLedTotalMismatch { zone_leds: u64, leds: usize },
    #[error("{colors} colours for {leds} LEDs")]
    ColorCountMismatch { colors: usize, leds: usize },
    #[error("zone {zone} segment {segment} ends at LED {end}, beyond {zone_leds}")]
    SegmentOutOfRange {
        zone: usize,
        segment: usize,
        end: u64,
        zone_leds: u32,
    },
}

impl ControllerDataError {
    /// True when the bytes parsed but describe a controller that cannot exist.
    pub fn is_inconsistency(&self) -> bool {
        matches!(
            self,
            Self::ActiveModeOutOfRange { .. }
                | Self::ZoneActiveModeOutOfRange { .. }
                | Self::ZoneBoundsInverted { .. }
                | Self::ZoneLedTotalMismatch { .. }
                | Self::ColorCountMismatch { .. }
                | Self::SegmentOutOfRange { .. }
        )
    }
}

/// Parses and validates a protocol 6 controller data reply payload.
pub fn parse_controller_data(payload: &[u8]) -> Result<ControllerData, ControllerDataError> {
    let mut cursor = Cursor::new(payload);
    let declared = cursor.u32("data_size")?;
    if declared as usize != payload.len() {
        return Err(ControllerDataError::SizeMismatch {
            declared,
            actual: payload.len(),
        });
    }
    let data = parse_device(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(ControllerDataError::TrailingBytes {
            count: cursor.remaining(),
        });
    }
    data.validate()?;
    Ok(data)
}

fn parse_device(cursor: &mut Cursor) -> Result<ControllerData, ControllerDataError> {
    let device_type = cursor.i32("device_type")?;
    let name = cursor.string("name")?;
    let vendor = cursor.string("vendor")?;
    let description = cursor.string("description")?;
    let version = cursor.string("version")?;
    let serial = cursor.string("serial")?;
    let location = cursor.string("location")?;
    let mode_count = cursor.u16("mode_count")?;
    let active_mode = cursor.i32("active_mode")?;
    let modes = list(cursor, mode_count, parse_mode)?;
    let zone_count = cursor.u16("zone_count")?;
    let zones = list(cursor, zone_count, parse_zone)?;
    let led_count = cursor.u16("led_count")?;
    let led_names = list(cursor, led_count, |cursor| cursor.string("led_name"))?;
    let colors = cursor.colors("colors")?;
    let display_name_count = cursor.u16("led_display_name_count")?;
    let led_display_names = list(cursor, display_name_count, |cursor| {
        cursor.string("led_display_name")
    })?;
    Ok(ControllerData {
        device_type,
        name,
        vendor,
        description,
        version,
        serial,
        location,
        active_mode,
        modes,
        zones,
        led_names,
        colors,
        led_display_names,
        flags: cursor.u32("flags")?,
        display_name: cursor.string("display_name")?,
        configuration: cursor.long_string("configuration")?,
    })
}

fn parse_mode(cursor: &mut Cursor) -> Result<ModeData, ControllerDataError> {
    Ok(ModeData {
        name: cursor.string("mode_name")?,
        flags: cursor.u32("mode_flags")?,
        speed_min: cursor.u32("mode_speed_min")?,
        speed_max: cursor.u32("mode_speed_max")?,
        brightness_min: cursor.u32("mode_brightness_min")?,
        brightness_max: cursor.u32("mode_brightness_max")?,
        colors_min: cursor.u32("mode_colors_min")?,
        colors_max: cursor.u32("mode_colors_max")?,
        speed: cursor.u32("mode_speed")?,
        brightness: cursor.u32("mode_brightness")?,
        direction: cursor.u32("mode_direction")?,
        color_mode: cursor.u32("mode_color_mode")?,
        colors: cursor.colors("mode_colors")?,
    })
}

fn parse_zone(cursor: &mut Cursor) -> Result<ZoneData, ControllerDataError> {
    let name = cursor.string("zone_name")?;
    let zone_type = cursor.i32("zone_type")?;
    let leds_min = cursor.u32("zone_leds_min")?;
    let leds_max = cursor.u32("zone_leds_max")?;
    let leds_count = cursor.u32("zone_leds_count")?;
    let matrix = parse_matrix(cursor, "zone_matrix")?;
    let segment_count = cursor.u16("segment_count")?;
    let segments = list(cursor, segment_count, parse_segment)?;
    let flags = cursor.u32("zone_flags")?;
    let active_mode = cursor.i32("zone_active_mode")?;
    let mode_count = cursor.u16("zone_mode_count")?;
    let modes = list(cursor, mode_count, parse_mode)?;
    Ok(ZoneData {
        name,
        zone_type,
        leds_min,
        leds_max,
        leds_count,
        matrix,
        segments,
        flags,
        active_mode,
        modes,
        display_name: cursor.string("zone_display_name")?,
    })
}

fn parse_segment(cursor: &mut Cursor) -> Result<SegmentData, ControllerDataError> {
    Ok(SegmentData {
        name: cursor.string("segment_name")?,
        segment_type: cursor.i32("segment_type")?,
        start: cursor.u32("segment_start")?,
        leds_count: cursor.u32("segment_leds_count")?,
        matrix: parse_matrix(cursor, "segment_matrix")?,
        flags: cursor.u32("segment_flags")?,
    })
}

fn parse_matrix(
    cursor: &mut Cursor,
    field: &'static str,
) -> Result<Option<MatrixMap>, ControllerDataError> {
    let declared = cursor.u16(field)?;
    if declared == 0 {
        return Ok(None);
    }
    let height = cursor.u32(field)?;
    let width = cursor.u32(field)?;
    let cell_bytes = u64::from(height)
        .checked_mul(u64::from(width))
        .and_then(|cells| cells.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or(ControllerDataError::Truncated { field })?;
    // OpenRGB sends this length as a u16, so maps above 64 KiB arrive with it wrapped.
    if (cell_bytes.wrapping_add(8) & 0xFFFF) as u16 != declared {
        return Err(ControllerDataError::MatrixSizeMismatch {
            declared,
            height,
            width,
        });
    }
    let cells = cursor
        .take(cell_bytes, field)?
        .as_chunks::<4>()
        .0
        .iter()
        .map(|cell| u32::from_le_bytes(*cell))
        .collect();
    Ok(Some(MatrixMap {
        height,
        width,
        cells,
    }))
}

fn list<T>(
    cursor: &mut Cursor,
    count: u16,
    parse: impl Fn(&mut Cursor) -> Result<T, ControllerDataError>,
) -> Result<Vec<T>, ControllerDataError> {
    (0..count).map(|_| parse(cursor)).collect()
}
