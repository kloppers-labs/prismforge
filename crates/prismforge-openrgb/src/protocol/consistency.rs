use super::controller::{ControllerData, ControllerDataError, ZoneData};

impl ControllerData {
    pub(super) fn validate(&self) -> Result<(), ControllerDataError> {
        if !self.modes.is_empty() && !index_in(self.active_mode, self.modes.len()) {
            return Err(ControllerDataError::ActiveModeOutOfRange {
                active: self.active_mode,
                modes: self.modes.len(),
            });
        }
        for (index, zone) in self.zones.iter().enumerate() {
            validate_zone(index, zone)?;
        }
        let zone_leds: u64 = self
            .zones
            .iter()
            .map(|zone| u64::from(zone.leds_count))
            .sum();
        if zone_leds != self.led_names.len() as u64 {
            return Err(ControllerDataError::ZoneLedTotalMismatch {
                zone_leds,
                leds: self.led_names.len(),
            });
        }
        if self.colors.len() != self.led_names.len() {
            return Err(ControllerDataError::ColorCountMismatch {
                colors: self.colors.len(),
                leds: self.led_names.len(),
            });
        }
        Ok(())
    }
}

fn validate_zone(index: usize, zone: &ZoneData) -> Result<(), ControllerDataError> {
    if zone.leds_min > zone.leds_max {
        return Err(ControllerDataError::ZoneBoundsInverted {
            zone: index,
            min: zone.leds_min,
            max: zone.leds_max,
        });
    }
    if zone.active_mode >= 0 && !index_in(zone.active_mode, zone.modes.len()) {
        return Err(ControllerDataError::ZoneActiveModeOutOfRange {
            zone: index,
            active: zone.active_mode,
            modes: zone.modes.len(),
        });
    }
    for (segment_index, segment) in zone.segments.iter().enumerate() {
        let end = u64::from(segment.start) + u64::from(segment.leds_count);
        if end > u64::from(zone.leds_count) {
            return Err(ControllerDataError::SegmentOutOfRange {
                zone: index,
                segment: segment_index,
                end,
                zone_leds: zone.leds_count,
            });
        }
    }
    Ok(())
}

fn index_in(value: i32, len: usize) -> bool {
    usize::try_from(value).is_ok_and(|index| index < len)
}
