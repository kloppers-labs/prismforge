use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::color::Rgb;
use crate::control::Bounds;
use crate::instance::{EffectInstance, InstanceError};
use crate::registry::prepare;

pub const PREVIEW_LIMITS: PreviewLimits = PreviewLimits {
    leds: Bounds { min: 1, max: 60 },
    frames: Bounds { min: 1, max: 90 },
    interval_ms: Bounds {
        min: 20,
        max: 1_000,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PreviewLimits {
    pub leds: Bounds,
    pub frames: Bounds,
    pub interval_ms: Bounds,
}

/// A request for preview frames; every number is clamped to [`PREVIEW_LIMITS`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PreviewRequest {
    pub instance: EffectInstance,
    pub led_count: u32,
    pub start_ms: u64,
    pub frame_count: u32,
    pub interval_ms: u32,
}

/// Frames at `start_ms`, then one every `interval_ms`; a static effect gets one frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PreviewBatch {
    pub led_count: u32,
    pub start_ms: u64,
    pub interval_ms: u32,
    pub is_static: bool,
    pub frames: Vec<Vec<Rgb>>,
}

pub fn preview(request: &PreviewRequest) -> Result<PreviewBatch, InstanceError> {
    let effect = prepare(&request.instance)?;
    let limits = PREVIEW_LIMITS;
    let led_count = limits.leds.clamp(request.led_count);
    let interval_ms = limits.interval_ms.clamp(request.interval_ms);
    let is_static = effect.is_static();
    let frame_count = if is_static {
        1
    } else {
        limits.frames.clamp(request.frame_count)
    };
    let frames = (0..u64::from(frame_count))
        .map(|index| {
            let at = request
                .start_ms
                .saturating_add(index * u64::from(interval_ms));
            effect.render_strip(Duration::from_millis(at), led_count as usize)
        })
        .collect();
    Ok(PreviewBatch {
        led_count,
        start_ms: request.start_ms,
        interval_ms,
        is_static,
        frames,
    })
}
