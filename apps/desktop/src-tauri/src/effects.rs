use std::future::{Future, ready};

use prismforge_core::session::{
    MAX_FRAME_RATE, MIN_FRAME_RATE, OutputError, RestoreOutcome, SessionHandle,
};
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_effects::{
    Bounds, EffectDefinition, EffectInstance, PREVIEW_LIMITS, PreviewBatch, PreviewLimits,
    PreviewRequest,
};
use serde::Serialize;

use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::view::CommandError;

const MAX_LABEL_CHARS: usize = 60;
const UNNAMED_PROGRAM: &str = "Another program";

/// The effects and the limits the interface offers; every number comes from the Rust side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EffectCatalog {
    pub effects: Vec<EffectDefinition>,
    pub frame_rate: Bounds,
    pub preview: PreviewLimits,
}

/// Renders preview frames in Rust; it uses no session and no backend.
pub fn preview_effect(request: PreviewRequest) -> Result<PreviewBatch, CommandError> {
    Ok(prismforge_effects::preview(&request)?)
}

impl<S: ViewSink> Shell<S> {
    pub async fn catalog(&self) -> Result<EffectCatalog, CommandError> {
        let effects = self
            .queue(|handle| ready(handle.effect_definitions()))
            .await?
            .await;
        Ok(EffectCatalog {
            effects,
            frame_rate: Bounds {
                min: MIN_FRAME_RATE,
                max: MAX_FRAME_RATE,
            },
            preview: PREVIEW_LIMITS,
        })
    }

    pub async fn apply_effect(
        &self,
        target: OutputTarget,
        effect: EffectInstance,
    ) -> Result<(), CommandError> {
        self.output(|handle| handle.apply_effect(target, effect))
            .await
    }

    pub async fn stop_effect(
        &self,
        target: OutputTarget,
    ) -> Result<Option<RestoreOutcome>, CommandError> {
        self.output(|handle| handle.stop_effect(target)).await
    }

    pub async fn select_mode(&self, device: DeviceRef, mode: usize) -> Result<(), CommandError> {
        self.output(|handle| handle.select_mode(device, mode)).await
    }

    pub async fn declare_external(
        &self,
        device: DeviceRef,
        label: String,
    ) -> Result<(), CommandError> {
        let label = program_label(&label);
        self.output(|handle| handle.declare_external(device, label))
            .await
    }

    pub async fn clear_external(&self, device: DeviceRef) -> Result<(), CommandError> {
        self.output(|handle| handle.clear_external(device)).await
    }

    pub async fn set_frame_rate(&self, frame_rate: u32) -> Result<u32, CommandError> {
        self.output(|handle| handle.set_frame_rate(frame_rate))
            .await
    }

    async fn output<T, F>(
        &self,
        command: impl FnOnce(&SessionHandle) -> F,
    ) -> Result<T, CommandError>
    where
        F: Future<Output = Result<T, OutputError>>,
    {
        Ok(self.queue(command).await?.await?)
    }
}

fn program_label(label: &str) -> String {
    let label: String = label.trim().chars().take(MAX_LABEL_CHARS).collect();
    let label = label.trim_end();
    if label.is_empty() {
        UNNAMED_PROGRAM.into()
    } else {
        label.into()
    }
}
