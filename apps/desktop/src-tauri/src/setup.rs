use std::time::Duration;

use prismforge_core::OutputTarget;
use prismforge_core::profile::NAME_LIMIT;
use prismforge_core::session::BatchReport;
use prismforge_core::setup::{
    BRANCH_LIMIT, COMPONENT_LED_LIMIT, COMPONENT_LIMIT, FRAME_LIMITS, LINE_POINT_LIMIT,
    OUTLINE_SIZE_LIMIT, OUTPUT_LIMIT, OutputStatus, Setup, VIEW_LIMIT, ViewId, plan_across,
    resolve_outputs,
};
use prismforge_effects::{EffectInstance, Flow, PREVIEW_LIMITS, Rgb, prepare};
use serde::{Deserialize, Serialize};

use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::stores::StoreKind;
use crate::view::CommandError;

/// Most LEDs one draft preview draws; a real PC or desk has a few hundred.
pub const PREVIEW_LED_LIMIT: usize = 16_384;
/// Most colours, LEDs times frames, in one draft preview answer.
pub const PREVIEW_COLOUR_LIMIT: usize = 60_000;

/// The stored setup of the store in use, each output matched against the session's devices of now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SetupStoreView {
    pub kind: StoreKind,
    pub setup: Setup,
    pub outputs: Vec<OutputStatus>,
    pub limits: SetupLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SetupLimits {
    pub components: usize,
    pub outputs: usize,
    pub views: usize,
    pub branches: usize,
    pub component_leds: u32,
    pub line_points: usize,
    pub name: usize,
    pub frame_min: u16,
    pub frame_max: u16,
    pub outline_size: u32,
}

const LIMITS: SetupLimits = SetupLimits {
    components: COMPONENT_LIMIT,
    outputs: OUTPUT_LIMIT,
    views: VIEW_LIMIT,
    branches: BRANCH_LIMIT,
    component_leds: COMPONENT_LED_LIMIT,
    line_points: LINE_POINT_LIMIT,
    name: NAME_LIMIT,
    frame_min: FRAME_LIMITS.0,
    frame_max: FRAME_LIMITS.1,
    outline_size: OUTLINE_SIZE_LIMIT,
};

/// What one apply across a view did: the batch report, the outputs that got nothing, and LEDs left dark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct AcrossReport {
    pub report: BatchReport,
    /// Outputs whose zone changed or whose parts no longer fit; nothing was sent to them.
    pub excluded: Vec<OutputStatus>,
    pub not_placed: Vec<NotPlaced>,
}

/// A target with `leds` of its `of` LEDs outside every placed component; they are written black.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct NotPlaced {
    pub target: OutputTarget,
    pub leds: u32,
    pub of: u32,
}

/// A draft preview of an effect across a view of a setup that may be unsaved; it uses no session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct AcrossPreviewRequest {
    pub setup: Setup,
    pub view: ViewId,
    pub flow: Flow,
    pub instance: EffectInstance,
    pub start_ms: u64,
    pub frame_count: u32,
    pub interval_ms: u32,
}

/// Frames at `start_ms`, then one every `interval_ms`, for each target by LED index; LEDs without a place are black.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct AcrossPreview {
    pub start_ms: u64,
    pub interval_ms: u32,
    pub is_static: bool,
    pub targets: Vec<PreviewTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PreviewTarget {
    pub target: OutputTarget,
    pub frames: Vec<Vec<Rgb>>,
}

/// Renders the draft in Rust from the setup it is given, against no device; nothing is stored or sent.
pub fn preview_across(request: AcrossPreviewRequest) -> Result<AcrossPreview, CommandError> {
    let effect = prepare(&request.instance)?;
    let plan = plan_across(&request.setup, &request.view, request.flow, &[])?;
    let leds: usize = plan.targets.iter().map(|placed| placed.points.len()).sum();
    if leds > PREVIEW_LED_LIMIT {
        return Err(CommandError::PreviewTooLarge {
            limit: PREVIEW_LED_LIMIT,
        });
    }
    let limits = PREVIEW_LIMITS;
    let interval_ms = request
        .interval_ms
        .clamp(limits.interval_ms.min, limits.interval_ms.max);
    let is_static = effect.is_static();
    let frame_count = if is_static {
        1
    } else {
        let room = u32::try_from(PREVIEW_COLOUR_LIMIT / leds.max(1)).unwrap_or(u32::MAX);
        request
            .frame_count
            .clamp(limits.frames.min, limits.frames.max)
            .min(room.max(1))
    };
    let times: Vec<Duration> = (0..u64::from(frame_count))
        .map(|index| {
            Duration::from_millis(
                request
                    .start_ms
                    .saturating_add(index * u64::from(interval_ms)),
            )
        })
        .collect();
    let targets = plan
        .targets
        .iter()
        .map(|placed| PreviewTarget {
            target: placed.target.clone(),
            frames: times
                .iter()
                .map(|time| effect.render_across(*time, plan.flow(), plan.plane(), &placed.points))
                .collect(),
        })
        .collect();
    Ok(AcrossPreview {
        start_ms: request.start_ms,
        interval_ms,
        is_static,
        targets,
    })
}

impl<S: ViewSink> Shell<S> {
    /// Takes the session slot only to read the devices and the store kind, then works on the store alone.
    pub async fn setup_view(&self) -> Result<SetupStoreView, CommandError> {
        let (handle, kind) = self.current().await;
        let profiles = self.stores.profiles(kind).await?;
        let setup = profiles.setup().await?;
        let devices = handle.map_or_else(Vec::new, |handle| handle.snapshot().devices);
        Ok(view_of(kind, setup, &devices))
    }

    /// Stores `setup` after the store's checks; it sends nothing to a device.
    pub async fn save_setup(&self, setup: Setup) -> Result<SetupStoreView, CommandError> {
        let (handle, kind) = self.current().await;
        let profiles = self.stores.profiles(kind).await?;
        profiles.save_setup(setup, handle.as_ref()).await?;
        let devices = handle.map_or_else(Vec::new, |handle| handle.snapshot().devices);
        Ok(view_of(kind, profiles.setup().await?, &devices))
    }

    /// Plans `view` of the saved setup against the devices of now and runs `effect` across it, under an output permit.
    pub async fn apply_across(
        &self,
        view: ViewId,
        flow: Flow,
        effect: EffectInstance,
        targets: Option<Vec<OutputTarget>>,
    ) -> Result<AcrossReport, CommandError> {
        let (_permit, handle, kind) = self.permitted().await?;
        let handle = handle.ok_or(CommandError::NoSession)?;
        let setup = self.stores.profiles(kind).await?.setup().await?;
        let plan = plan_across(&setup, &view, flow, &handle.snapshot().devices)?;
        let plan = match targets {
            Some(targets) => plan.only(&targets),
            None => plan,
        };
        let excluded = plan.excluded.clone();
        let not_placed = plan
            .targets
            .iter()
            .filter_map(|placed| {
                let leds = placed.not_placed().len();
                (leds > 0).then(|| NotPlaced {
                    target: placed.target.clone(),
                    leds: count(leds),
                    of: count(placed.points.len()),
                })
            })
            .collect();
        let report = handle.apply_across(plan, effect).await?;
        Ok(AcrossReport {
            report,
            excluded,
            not_placed,
        })
    }
}

fn view_of(
    kind: StoreKind,
    setup: Setup,
    devices: &[prismforge_core::session::SessionDevice],
) -> SetupStoreView {
    SetupStoreView {
        kind,
        outputs: resolve_outputs(&setup, devices),
        setup,
        limits: LIMITS,
    }
}

fn count(leds: usize) -> u32 {
    u32::try_from(leds).unwrap_or(u32::MAX)
}
