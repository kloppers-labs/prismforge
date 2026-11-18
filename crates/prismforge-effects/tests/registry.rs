mod support;

use std::collections::BTreeSet;

use prismforge_effects::{ControlKind, ControlValue, definitions, prepare};

use support::{RED, color, instance};

#[test]
fn built_in_effects_are_listed_in_a_stable_order() {
    let ids: Vec<String> = definitions().into_iter().map(|effect| effect.id).collect();

    assert_eq!(
        ids,
        [
            "solid",
            "breathing",
            "gradient",
            "rainbow-wave",
            "color-cycle",
            "moving-highlight"
        ]
    );
}

#[test]
fn every_definition_describes_its_controls_for_a_generic_interface() {
    for effect in definitions() {
        assert!(!effect.label.is_empty() && !effect.description.is_empty());
        let ids: BTreeSet<&str> = effect
            .controls
            .iter()
            .map(|control| control.id.as_str())
            .collect();
        assert_eq!(ids.len(), effect.controls.len(), "{}", effect.id);
        assert!(ids.contains("brightness"), "{}", effect.id);
        for control in &effect.controls {
            assert!(!control.label.is_empty() && !control.description.is_empty());
        }
    }
}

#[test]
fn the_defaults_of_every_definition_are_valid() {
    for effect in definitions() {
        let required: Vec<(&str, ControlValue)> = effect
            .controls
            .iter()
            .filter(|control| matches!(control.kind, ControlKind::Color { default: None }))
            .map(|control| (control.id.as_str(), color(RED)))
            .collect();

        let prepared = prepare(&instance(&effect.id, &required));

        assert!(prepared.is_ok(), "{}: {prepared:?}", effect.id);
        assert_eq!(
            prepared.expect("valid").instance().values.len(),
            effect.controls.len()
        );
    }
}
