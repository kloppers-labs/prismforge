use serde::{Deserialize, Serialize};

use crate::device::DeviceRef;
use crate::profile::Extra;
use crate::scene::OutputTarget;
use crate::setup::geometry::{Geometry, Location};
use crate::setup::ids::{ComponentId, OutputId, ViewId};

/// Where each LED of the user's devices is: components with a shape, the outputs they are wired to, and views that place them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Setup {
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub outputs: Vec<SetupOutput>,
    #[serde(default)]
    pub views: Vec<View>,
    #[serde(flatten)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Component {
    pub id: ComponentId,
    pub name: String,
    pub geometry: Geometry,
    #[serde(flatten)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub extra: Extra,
}

/// One channel PrismForge writes: a whole device, a zone, or the part of a zone from `first_led`.
/// Every branch gets the same data from `first_led` on; the components of one branch follow each other on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SetupOutput {
    pub id: OutputId,
    pub device: DeviceRef,
    pub zone: Option<usize>,
    pub label: String,
    pub zone_name: Option<String>,
    pub first_led: u32,
    pub branches: Vec<Vec<ComponentId>>,
    #[serde(flatten)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub extra: Extra,
}

/// A named picture of the setup, such as the PC or the desk; a component appears at most once in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(rename = "SetupView"))]
pub struct View {
    pub id: ViewId,
    pub name: String,
    pub placements: Vec<Placement>,
    #[serde(flatten)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub extra: Extra,
}

/// The centre (`x`, `y`) in view units, the length of the longer side, degrees clockwise, and a left-right flip before the turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Placement {
    pub component: ComponentId,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub rotation: u16,
    pub flipped: bool,
    #[serde(flatten)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub extra: Extra,
}

impl Setup {
    pub fn component(&self, id: &ComponentId) -> Option<&Component> {
        self.components.iter().find(|component| &component.id == id)
    }

    pub fn output(&self, id: &OutputId) -> Option<&SetupOutput> {
        self.outputs.iter().find(|output| &output.id == id)
    }

    pub fn view(&self, id: &ViewId) -> Option<&View> {
        self.views.iter().find(|view| &view.id == id)
    }

    pub(crate) fn led_count(&self, id: &ComponentId) -> u32 {
        self.component(id)
            .map_or(0, |component| component.geometry.led_count())
    }

    /// LEDs the output uses from `first_led`: the longest of its branches.
    pub(crate) fn span(&self, output: &SetupOutput) -> u64 {
        output
            .branches
            .iter()
            .map(|branch| {
                branch
                    .iter()
                    .map(|component| u64::from(self.led_count(component)))
                    .sum::<u64>()
            })
            .max()
            .unwrap_or(0)
    }
}

impl SetupOutput {
    pub fn target(&self) -> OutputTarget {
        OutputTarget {
            device: self.device.clone(),
            zone: self.zone,
        }
    }
}

impl View {
    pub fn placement(&self, component: &ComponentId) -> Option<&Placement> {
        self.placements
            .iter()
            .find(|placement| &placement.component == component)
    }
}

impl Placement {
    /// Where a point of the component's own space lands in the view.
    pub fn locate(&self, local: Location) -> Location {
        let size = f64::from(self.size);
        let x = if self.flipped { -local.x } else { local.x } * size;
        let y = local.y * size;
        let (sin, cos) = f64::from(self.rotation).to_radians().sin_cos();
        Location::new(
            f64::from(self.x) + x * cos - y * sin,
            f64::from(self.y) + x * sin + y * cos,
        )
    }
}
