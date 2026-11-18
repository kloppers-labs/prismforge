mod support;

use prismforge_effects::{
    ControlValue, Direction, InstanceError, ValueError, ValueProblem, ValueType, prepare,
};

use support::{RED, color, instance, number, palette};

fn errors(effect: &str, values: &[(&str, ControlValue)]) -> Vec<ValueError> {
    match prepare(&instance(effect, values)) {
        Err(InstanceError::InvalidValues { errors }) => errors,
        other => panic!("expected invalid values, got {other:?}"),
    }
}

fn problem(control: &str, problem: ValueProblem) -> ValueError {
    ValueError {
        control: control.into(),
        problem,
    }
}

#[test]
fn an_unknown_effect_is_refused() {
    assert_eq!(
        prepare(&instance("sparkle", &[])).map(|_| ()),
        Err(InstanceError::UnknownEffect {
            effect: "sparkle".into()
        })
    );
}

#[test]
fn an_unknown_control_is_refused() {
    assert_eq!(
        errors("gradient", &[("glitter", number(3))]),
        [problem("glitter", ValueProblem::UnknownControl)]
    );
}

#[test]
fn a_value_of_the_wrong_type_is_refused() {
    assert_eq!(
        errors("breathing", &[("speed", color(RED))]),
        [problem(
            "speed",
            ValueProblem::WrongType {
                expected: ValueType::Number
            }
        )]
    );
}

#[test]
fn brightness_and_speed_outside_their_range_are_refused() {
    assert_eq!(
        errors(
            "rainbow-wave",
            &[("brightness", number(101)), ("speed", number(241))]
        ),
        [
            problem("brightness", ValueProblem::OutOfRange { min: 0, max: 100 }),
            problem("speed", ValueProblem::OutOfRange { min: 0, max: 240 }),
        ]
    );
}

#[test]
fn a_number_between_steps_is_refused() {
    assert_eq!(
        errors("moving-highlight", &[("width", number(22))]),
        [problem("width", ValueProblem::OffStep { step: 5 })]
    );
}

#[test]
fn a_missing_required_colour_is_refused() {
    assert_eq!(
        errors("solid", &[]),
        [problem("color", ValueProblem::Missing)]
    );
}

#[test]
fn a_palette_with_too_few_or_too_many_colours_is_refused() {
    let bounds = ValueProblem::PaletteLength { min: 2, max: 8 };

    assert_eq!(
        errors("gradient", &[("palette", palette(&[RED]))]),
        [problem("palette", bounds.clone())]
    );
    assert_eq!(
        errors("color-cycle", &[("palette", palette(&[RED; 9]))]),
        [problem("palette", bounds)]
    );
}

#[test]
fn defaults_fill_every_missing_optional_value() {
    let effect = prepare(&instance("breathing", &[("speed", number(30))])).expect("valid");

    let values = &effect.instance().values;
    assert_eq!(values.get("speed"), Some(&number(30)));
    assert_eq!(values.get("brightness"), Some(&number(100)));
    assert!(matches!(values.get("color"), Some(ControlValue::Color(_))));
    assert_eq!(values.len(), 3);
}

#[test]
fn the_stored_form_is_plain_json_with_stable_ids() {
    let effect = prepare(&instance(
        "gradient",
        &[
            ("palette", palette(&[RED, RED])),
            ("direction", ControlValue::Direction(Direction::Backward)),
        ],
    ))
    .expect("valid");

    let json = serde_json::to_value(effect.instance()).expect("serialise");

    assert_eq!(
        json,
        serde_json::json!({
            "effect": "gradient",
            "values": {
                "brightness": { "Number": 100 },
                "direction": { "Direction": "Backward" },
                "palette": { "Palette": [
                    { "r": 255, "g": 0, "b": 0 },
                    { "r": 255, "g": 0, "b": 0 }
                ] }
            }
        })
    );
    let read: prismforge_effects::EffectInstance =
        serde_json::from_value(json).expect("deserialise");
    assert_eq!(&read, effect.instance());
}
