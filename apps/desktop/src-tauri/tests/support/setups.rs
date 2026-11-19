use prismforge_core::DeviceRef;
use prismforge_core::profile::Extra;
use prismforge_core::setup::{
    Component, ComponentId, Corner, FirstLed, Geometry, Matrix, OutputId, Placement, Ring, Runs,
    Setup, SetupOutput, Strip, Turn, View, ViewId,
};

pub fn component(id: &str, name: &str, geometry: Geometry) -> Component {
    Component {
        id: ComponentId::parse(id).expect("component id"),
        name: name.into(),
        geometry,
        extra: Extra::new(),
    }
}

pub fn fan(id: &str, name: &str, leds: u32) -> Component {
    component(
        id,
        name,
        Geometry::Ring(Ring {
            leds,
            first_angle: 0,
            turn: Turn::Clockwise,
            extra: Extra::new(),
        }),
    )
}

pub fn strip(id: &str, name: &str, leds: u32) -> Component {
    component(
        id,
        name,
        Geometry::Strip(Strip {
            leds,
            first: FirstLed::AtStart,
            extra: Extra::new(),
        }),
    )
}

pub fn matrix(id: &str, name: &str, columns: u32, rows: u32) -> Component {
    component(
        id,
        name,
        Geometry::Matrix(Matrix {
            columns,
            rows,
            first: Corner::TopLeft,
            runs: Runs::AlongRows,
            serpentine: false,
            extra: Extra::new(),
        }),
    )
}

pub fn output(
    id: &str,
    device: &DeviceRef,
    zone: Option<(usize, &str)>,
    label: &str,
    branches: &[&[&str]],
) -> SetupOutput {
    SetupOutput {
        id: OutputId::parse(id).expect("output id"),
        device: device.clone(),
        zone: zone.map(|(index, _)| index),
        label: label.into(),
        zone_name: zone.map(|(_, name)| name.into()),
        first_led: 0,
        branches: branches
            .iter()
            .map(|branch| {
                branch
                    .iter()
                    .map(|name| ComponentId::parse(name).expect("component id"))
                    .collect()
            })
            .collect(),
        extra: Extra::new(),
    }
}

pub fn place(id: &str, x: i32, y: i32, size: u32, rotation: u16) -> Placement {
    Placement {
        component: ComponentId::parse(id).expect("component id"),
        x,
        y,
        size,
        rotation,
        flipped: false,
        extra: Extra::new(),
    }
}

pub fn pc() -> ViewId {
    ViewId::parse("v-pc").expect("view id")
}

pub fn desk() -> ViewId {
    ViewId::parse("v-desk").expect("view id")
}

pub fn view(id: ViewId, name: &str, placements: Vec<Placement>) -> View {
    View {
        id,
        name: name.into(),
        placements,
        extra: Extra::new(),
    }
}

pub fn setup(components: Vec<Component>, outputs: Vec<SetupOutput>, views: Vec<View>) -> Setup {
    Setup {
        components,
        outputs,
        views,
        extra: Extra::new(),
    }
}
