use prismforge_core::DeviceRef;
use prismforge_core::setup::{Control, SetupProblem};

use crate::support::{
    BOARD, HEADER_1, HEADER_2, STRIP, board, chained_fans, component, component_id, mirrored_fans,
    output, output_id, ring, strip,
};

#[test]
fn represents_mirrored_outputs_as_shared_control() {
    let setup = mirrored_fans();
    setup.validate().expect("valid");
    setup.check_devices(&[board()]).expect("fits");

    let control = |id: &str| setup.control(&component_id(id)).expect("wired");
    let target = output("o-hub", HEADER_1, 0, &[]).target();

    assert_eq!(
        control("c-fan-a"),
        Control {
            output: output_id("o-hub"),
            target: target.clone(),
            leds: 0..12,
            shared_with: vec![component_id("c-fan-b"), component_id("c-fan-c")],
        }
    );
    for fan in ["c-fan-b", "c-fan-c"] {
        assert_eq!(control(fan).output, output_id("o-hub"));
        assert_eq!(control(fan).target, target);
        assert_eq!(control(fan).leds, 0..12);
        assert_eq!(control(fan).shared_with.len(), 2);
    }
    assert_eq!(setup.outputs.len(), 1);
}

#[test]
fn represents_a_daisy_chain_as_consecutive_parts_of_one_zone() {
    let setup = chained_fans();
    setup.validate().expect("valid");
    setup.check_devices(&[board()]).expect("fits");

    let leds: Vec<_> = ["c-fan-1", "c-fan-2", "c-fan-3"]
        .iter()
        .map(|id| setup.control(&component_id(id)).expect("wired"))
        .map(|control| (control.output, control.leds, control.shared_with))
        .collect();

    assert_eq!(
        leds,
        vec![
            (output_id("o-chain"), 0..12, Vec::new()),
            (output_id("o-chain"), 12..24, Vec::new()),
            (output_id("o-chain"), 24..36, Vec::new()),
        ]
    );
}

#[test]
fn rejects_led_count_outside_backend_bounds() {
    let mut fixed = mirrored_fans();
    fixed.components.push(component("c-bar", strip(12)));
    fixed
        .outputs
        .push(output("o-strip", STRIP, 0, &[&["c-bar"]]));
    let mut resizable = chained_fans();
    resizable.components[0] = component("c-fan-1", ring(1025));

    fixed.validate().expect("valid without devices");
    assert_eq!(
        fixed.check_devices(&[board()]),
        Err(SetupProblem::LedCountOutsideBounds {
            component: component_id("c-bar"),
            leds: 12,
            min: 1,
            max: 8,
        })
    );
    assert_eq!(
        resizable.check_devices(&[board()]),
        Err(SetupProblem::LedCountOutsideBounds {
            component: component_id("c-fan-1"),
            leds: 1025,
            min: 1,
            max: 1024,
        })
    );
}

#[test]
fn refuses_parts_beyond_the_leds_the_zone_has_now() {
    let mut setup = chained_fans();
    setup.components.push(component("c-fan-4", ring(12)));
    setup.outputs[0].branches[0].push(component_id("c-fan-4"));

    setup.validate().expect("valid without devices");
    assert_eq!(
        setup.check_devices(&[board()]),
        Err(SetupProblem::ExceedsOutput {
            output: output_id("o-chain"),
            needed: 48,
            available: 36,
        })
    );
}

#[test]
fn refuses_overlapping_parts_of_one_zone() {
    let mut setup = chained_fans();
    setup.components.push(component("c-strip", strip(10)));
    setup
        .outputs
        .push(output("o-tail", HEADER_2, 30, &[&["c-strip"]]));
    let mut apart = setup.clone();
    apart.outputs[0].branches[0].pop();
    apart.outputs[1].first_led = 24;

    assert_eq!(
        setup.validate(),
        Err(SetupProblem::OverlappingParts {
            first: output_id("o-chain"),
            second: output_id("o-tail"),
        })
    );
    apart.validate().expect("parts side by side");
    apart.check_devices(&[board()]).expect("fits");
}

#[test]
fn refuses_a_whole_device_beside_one_of_its_zones() {
    let mut setup = chained_fans();
    setup.components.push(component("c-all", strip(4)));
    let mut whole = output("o-board", HEADER_1, 0, &[&["c-all"]]);
    whole.zone = None;
    whole.zone_name = None;
    whole.device = DeviceRef::new(BOARD);
    setup.outputs.push(whole);

    assert_eq!(
        setup.validate(),
        Err(SetupProblem::OverlappingParts {
            first: output_id("o-chain"),
            second: output_id("o-board"),
        })
    );
}

#[test]
fn refuses_a_component_wired_to_two_places() {
    let mut setup = mirrored_fans();
    setup
        .outputs
        .push(output("o-other", HEADER_2, 0, &[&["c-fan-a"]]));

    assert_eq!(
        setup.validate(),
        Err(SetupProblem::WiredTwice {
            component: component_id("c-fan-a"),
        })
    );
}
