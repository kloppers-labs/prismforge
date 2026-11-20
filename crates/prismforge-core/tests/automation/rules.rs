use prismforge_core::automation::{Rule, RuleAction, RuleProblem, Rules, Trigger};
use prismforge_core::profile::ProfileId;
use serde_json::json;

fn id(value: &str) -> ProfileId {
    ProfileId::parse(value).expect("profile id")
}

fn on(action: RuleAction) -> Rule {
    Rule {
        enabled: true,
        action,
        ..Rule::default()
    }
}

#[test]
fn every_event_has_exactly_one_rule_and_none_is_on_by_default() {
    let rules = Rules::default();
    assert_eq!(
        Trigger::ALL,
        [
            Trigger::Starts,
            Trigger::Connects,
            Trigger::Sleeps,
            Trigger::Wakes,
            Trigger::SessionEnds,
            Trigger::Quits,
        ]
    );
    for trigger in Trigger::ALL {
        let rule = rules.rule(trigger);
        assert!(!rule.enabled, "{trigger:?} is on by default");
        assert_eq!(rule.action, RuleAction::Nothing);
    }
    assert_eq!(rules.mark(), None);
}

#[test]
fn a_marked_profile_becomes_the_connects_rule_switched_on() {
    let rules = Rules::migrated(Some(id("p-evening")));
    assert_eq!(
        *rules.rule(Trigger::Connects),
        on(RuleAction::ApplyProfile {
            profile: id("p-evening")
        })
    );
    for trigger in Trigger::ALL
        .into_iter()
        .filter(|trigger| *trigger != Trigger::Connects)
    {
        assert!(!rules.rule(trigger).enabled);
    }
    assert_eq!(rules.mark(), Some(&id("p-evening")));
}

#[test]
fn no_mark_gives_a_connects_rule_that_is_off() {
    let rules = Rules::migrated(None);
    assert_eq!(rules, Rules::default());
    assert!(!rules.rule(Trigger::Connects).enabled);
}

#[test]
fn only_a_switched_on_connects_rule_with_a_profile_is_a_mark() {
    let mut rules = Rules::default();
    rules.set(
        Trigger::Connects,
        Rule {
            enabled: false,
            action: RuleAction::ApplyProfile { profile: id("p-a") },
            ..Rule::default()
        },
    );
    assert_eq!(rules.mark(), None);
    rules.set(
        Trigger::Wakes,
        on(RuleAction::ApplyProfile { profile: id("p-b") }),
    );
    assert_eq!(rules.mark(), None);
}

#[test]
fn every_event_allows_every_action_this_version_knows() {
    for trigger in Trigger::ALL {
        assert_eq!(on(RuleAction::LightsOff).check(), Ok(()), "{trigger:?}");
        let profile = on(RuleAction::ApplyProfile { profile: id("p-a") });
        assert_eq!(profile.check(), Ok(()), "{trigger:?}");
        assert_eq!(on(RuleAction::Nothing).check(), Ok(()));
    }
}

#[test]
fn an_action_from_a_newer_version_cannot_be_set() {
    assert_eq!(
        on(RuleAction::Newer).check(),
        Err(RuleProblem::NeedsNewerPrismForge)
    );
}

#[test]
fn stored_rules_keep_what_a_newer_version_wrote() {
    let stored = json!({
        "sleeps": {"enabled": true, "action": {"Dim": {"level": 3}}, "delay_ms": 20},
        "wakes": {"enabled": true, "action": {"ApplyProfile": {"profile": "p-morning"}}},
        "display_off": {"enabled": true, "action": "LightsOff"},
    });
    let rules: Rules = serde_json::from_value(stored.clone()).expect("rules");
    assert_eq!(rules.rule(Trigger::Sleeps).action, RuleAction::Newer);
    assert!(rules.rule(Trigger::Sleeps).enabled);
    assert_eq!(
        rules.rule(Trigger::Wakes).action,
        RuleAction::ApplyProfile {
            profile: id("p-morning")
        }
    );
    assert!(!rules.rule(Trigger::Starts).enabled);
    let written = serde_json::to_value(&rules).expect("encode");
    assert_eq!(written["sleeps"], stored["sleeps"]);
    assert_eq!(written["display_off"], stored["display_off"]);
    assert_eq!(written["wakes"], stored["wakes"]);
}
