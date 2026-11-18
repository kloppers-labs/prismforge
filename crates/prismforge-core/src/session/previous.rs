use prismforge_effects::Rgb;

use crate::backend::RgbBackend;
use crate::device::{DeviceDescriptor, DeviceRef};
use crate::error::BackendError;
use crate::inventory::Inventory;
use crate::session::identify::RestoreFailure;

/// The mode and colours the inventory reported for a device before PrismForge changed it.
#[derive(Clone)]
pub(crate) struct PreviousState {
    pub device: DeviceRef,
    pub known: bool,
    steps: Vec<RestoreStep>,
    colors: Vec<Rgb>,
}

#[derive(Clone)]
enum RestoreStep {
    Mode { index: usize, name: String },
    Colors,
}

pub(crate) struct RestoreError {
    pub steps_done: usize,
    pub error: BackendError,
}

impl PreviousState {
    pub fn capture(device: &DeviceRef, descriptor: &DeviceDescriptor, suspect: bool) -> Self {
        let capabilities = &descriptor.capabilities;
        let restore_mode = capabilities
            .active_mode
            .filter(|_| !capabilities.in_direct_mode())
            .and_then(|index| Some((index, capabilities.modes.get(index)?)));
        let mut steps = Vec::new();
        if let Some((index, mode)) = restore_mode {
            steps.push(RestoreStep::Mode {
                index,
                name: mode.name.clone(),
            });
        }
        if restore_mode.is_none_or(|(_, mode)| mode.per_led_color) {
            steps.push(RestoreStep::Colors);
        }
        Self {
            device: device.clone(),
            known: capabilities.active_mode.is_some() && !suspect,
            steps,
            colors: descriptor.colors.clone(),
        }
    }

    /// The colours a device showed under PrismForge's own effects, written back as one frame.
    pub fn shown(device: &DeviceRef, colors: Vec<Rgb>) -> Self {
        Self {
            device: device.clone(),
            known: true,
            steps: vec![RestoreStep::Colors],
            colors,
        }
    }

    /// Runs the restore steps after the first `skip`; the count in an error includes the skipped ones.
    pub async fn restore<B: RgbBackend>(
        &self,
        backend: &mut B,
        skip: usize,
    ) -> Result<(), RestoreError> {
        for (position, step) in self.steps.iter().enumerate().skip(skip) {
            let result = match step {
                RestoreStep::Mode { index, .. } => backend.reapply_mode(&self.device, *index).await,
                RestoreStep::Colors => backend.write_device_frame(&self.device, &self.colors).await,
            };
            result.map_err(|error| RestoreError {
                steps_done: position,
                error,
            })?;
        }
        Ok(())
    }

    /// Checks that a refreshed inventory still resolves this reference to a device with the remembered shape.
    pub fn still_matches(&self, inventory: &Inventory) -> Result<(), RestoreFailure> {
        let index = inventory
            .locate(&self.device)
            .map_err(|error| RestoreFailure::Refused { error })?;
        let descriptor = &inventory.devices[index].descriptor;
        let modes_match = self.steps.iter().all(|step| match step {
            RestoreStep::Mode { index, name } => descriptor
                .capabilities
                .modes
                .get(*index)
                .is_some_and(|mode| &mode.name == name),
            RestoreStep::Colors => descriptor.led_count == self.colors.len(),
        });
        if modes_match {
            Ok(())
        } else {
            Err(RestoreFailure::TargetChanged)
        }
    }
}
