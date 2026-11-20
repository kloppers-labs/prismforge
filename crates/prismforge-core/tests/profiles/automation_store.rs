use prismforge_core::automation::{
    Rule, RuleAction, RuleProblem, RuleRun, RunFailure, RunOutcome, Trigger,
};
use prismforge_core::profile::ProfileId;
use prismforge_core::store::{NoticeEvent, ProfileStore, StdFileOps, StoreError};
use serde_json::{Value, json};

use crate::support::{TempDir, profile};

const SETTINGS: &str = "settings.json";
const RUNS: &str = "rule-runs.json";

fn id(value: &str) -> ProfileId {
    ProfileId::parse(value).expect("valid id")
}

fn old_settings(mark: Option<&ProfileId>) -> Vec<u8> {
    serde_json::to_vec_pretty(&json!({
        "format": "prismforge.settings",
        "schema_version": {"major": 1, "minor": 0},
        "apply_on_connect": mark.map(ProfileId::as_str),
    }))
    .expect("json")
}

fn stored(dir: &TempDir, name: &str) -> Value {
    serde_json::from_slice(&dir.read(name)).expect("json")
}

fn apply(profile: &ProfileId) -> Rule {
    Rule {
        enabled: true,
        action: RuleAction::ApplyProfile {
            profile: profile.clone(),
        },
        ..Rule::default()
    }
}

fn lights_off() -> Rule {
    Rule {
        enabled: true,
        action: RuleAction::LightsOff,
        ..Rule::default()
    }
}

/// A store an earlier PrismForge left: one profile, marked or not.
fn earlier_store(label: &str, marked: bool) -> (TempDir, ProfileId) {
    let dir = TempDir::new(label);
    let id = {
        let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
        store
            .create(profile("Evening", Vec::new()))
            .expect("create")
    };
    dir.write(SETTINGS, &old_settings(marked.then_some(&id)));
    (dir, id)
}

#[test]
fn a_marked_profile_becomes_the_connects_rule_and_loading_writes_nothing() {
    let (dir, id) = earlier_store("migrate-marked", true);
    let before = dir.files();
    let bytes = dir.read(SETTINGS);

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert_eq!(*store.rules().rule(Trigger::Connects), apply(&id));
    assert_eq!(store.apply_on_connect(), Some(id.clone()));
    assert_eq!(store.summary(None).apply_on_connect, Some(id));
    assert_eq!(dir.files(), before);
    assert_eq!(dir.read(SETTINGS), bytes);
}

#[test]
fn without_a_mark_the_connects_rule_exists_and_is_off() {
    let (dir, _) = earlier_store("migrate-unmarked", false);
    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    for trigger in Trigger::ALL {
        assert!(!store.rules().rule(trigger).enabled, "{trigger:?}");
    }
    assert_eq!(store.apply_on_connect(), None);
}

#[test]
fn a_mark_of_a_missing_profile_stays_and_is_reported_not_guessed() {
    let dir = TempDir::new("migrate-missing");
    dir.write(SETTINGS, &old_settings(Some(&id("p-gone"))));
    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert_eq!(*store.rules().rule(Trigger::Connects), apply(&id("p-gone")));
    assert_eq!(
        store.profile(&id("p-gone")),
        Err(StoreError::UnknownProfile { id: id("p-gone") })
    );
}

#[test]
fn the_first_rule_change_stores_the_rules_and_keeps_the_mark_for_older_versions() {
    let (dir, id) = earlier_store("migrate-write", true);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.set_rule(Trigger::Sleeps, lights_off()).expect("set");

    let written = stored(&dir, SETTINGS);
    assert_eq!(written["schema_version"], json!({"major": 1, "minor": 1}));
    assert_eq!(written["apply_on_connect"], json!(id.as_str()));
    assert_eq!(
        written["automation"]["connects"],
        json!({"enabled": true, "action": {"ApplyProfile": {"profile": id.as_str()}}})
    );
    assert_eq!(
        written["automation"]["sleeps"],
        json!({"enabled": true, "action": "LightsOff"})
    );

    store
        .set_rule(Trigger::Connects, Rule::default())
        .expect("switch off");
    assert_eq!(stored(&dir, SETTINGS)["apply_on_connect"], Value::Null);
    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert!(!reopened.rules().rule(Trigger::Connects).enabled);
    assert_eq!(*reopened.rules().rule(Trigger::Sleeps), lights_off());
}

#[test]
fn a_mark_changed_by_an_older_version_is_taken_over() {
    let (dir, first) = earlier_store("older-writer", true);
    let second = {
        let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
        store.set_rule(Trigger::Sleeps, lights_off()).expect("set");
        store
            .create(profile("Morning", Vec::new()))
            .expect("create")
    };
    let mut older = stored(&dir, SETTINGS);
    older["apply_on_connect"] = json!(second.as_str());
    dir.write(SETTINGS, &serde_json::to_vec(&older).expect("json"));

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert_eq!(*store.rules().rule(Trigger::Connects), apply(&second));
    assert_eq!(*store.rules().rule(Trigger::Sleeps), lights_off());
    assert_ne!(first, second);

    older["apply_on_connect"] = Value::Null;
    dir.write(SETTINGS, &serde_json::to_vec(&older).expect("json"));
    let store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert!(!store.rules().rule(Trigger::Connects).enabled);
}

#[test]
fn an_older_version_keeps_the_rules_it_does_not_know() {
    let (dir, _) = earlier_store("older-reader", true);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store
        .set_rule(Trigger::Wakes, lights_off())
        .expect("set wake rule");
    store
        .set_rule(Trigger::Quits, lights_off())
        .expect("set quit rule");
    let written = stored(&dir, SETTINGS);
    let object = written.as_object().expect("object");
    let known = ["format", "schema_version", "apply_on_connect"];
    let unknown: Vec<&String> = object
        .keys()
        .filter(|key| !known.contains(&key.as_str()))
        .collect();
    assert_eq!(unknown, ["automation"]);
}

#[test]
fn a_rule_must_name_a_profile_the_store_has_and_an_action_this_version_knows() {
    let (dir, id) = earlier_store("rule-checks", false);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert_eq!(
        store.set_rule(
            Trigger::Wakes,
            apply(&ProfileId::parse("p-none").expect("id"))
        ),
        Err(StoreError::UnknownProfile {
            id: ProfileId::parse("p-none").expect("id")
        })
    );
    let newer = Rule {
        enabled: true,
        action: RuleAction::Newer,
        ..Rule::default()
    };
    assert_eq!(
        store.set_rule(Trigger::Connects, newer),
        Err(StoreError::InvalidRule {
            problem: RuleProblem::NeedsNewerPrismForge
        })
    );
    store
        .set_rule(Trigger::Connects, lights_off())
        .expect("lights off when PrismForge connects");
    store.set_rule(Trigger::Wakes, apply(&id)).expect("set");
    assert_eq!(*store.rules().rule(Trigger::Wakes), apply(&id));
}

#[test]
fn a_rule_whose_profile_was_deleted_can_still_be_switched_off() {
    let (dir, id) = earlier_store("rule-off-deleted", false);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.set_rule(Trigger::Wakes, apply(&id)).expect("set");
    store.delete(&id).expect("delete");
    let off = Rule {
        enabled: false,
        ..apply(&id)
    };
    store
        .set_rule(Trigger::Wakes, off.clone())
        .expect("switch off");
    assert_eq!(*store.rules().rule(Trigger::Wakes), off);
    assert_eq!(
        store.set_rule(Trigger::Wakes, apply(&id)),
        Err(StoreError::UnknownProfile { id })
    );
}

#[test]
fn deleting_a_profile_keeps_the_rules_that_name_it() {
    let (dir, id) = earlier_store("delete-keeps", true);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store.set_rule(Trigger::Wakes, apply(&id)).expect("set");
    store.delete(&id).expect("delete");
    assert_eq!(*store.rules().rule(Trigger::Wakes), apply(&id));
    assert_eq!(*store.rules().rule(Trigger::Connects), apply(&id));
    assert_eq!(
        store.profile(&id),
        Err(StoreError::UnknownProfile { id: id.clone() })
    );
}

#[test]
fn rules_of_a_newer_settings_file_are_refused_and_the_file_is_left_alone() {
    let dir = TempDir::new("rules-newer");
    let newer = serde_json::to_vec(&json!({
        "format": "prismforge.settings",
        "schema_version": {"major": 2, "minor": 0},
    }))
    .expect("json");
    dir.write(SETTINGS, &newer);
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert!(matches!(
        store.set_rule(Trigger::Sleeps, lights_off()),
        Err(StoreError::NewerFile { .. })
    ));
    assert_eq!(dir.read(SETTINGS), newer);
}

fn failed(trigger: Trigger, at_ms: u64) -> RuleRun {
    RuleRun {
        trigger,
        at_ms,
        outcome: RunOutcome::Failed {
            failure: RunFailure::TimedOut { limit_ms: 1000 },
        },
        seen: false,
    }
}

#[test]
fn a_run_is_unseen_until_the_user_saw_it_and_that_is_kept() {
    let dir = TempDir::new("rule-runs-seen");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    store
        .record_run(failed(Trigger::Sleeps, 1))
        .expect("record");
    store
        .record_run(failed(Trigger::SessionEnds, 2))
        .expect("record");
    store.mark_runs_seen(&[Trigger::Sleeps]).expect("seen");

    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    let seen: Vec<(Trigger, bool)> = reopened
        .runs()
        .iter()
        .map(|run| (run.trigger, run.seen))
        .collect();
    assert_eq!(
        seen,
        [(Trigger::Sleeps, true), (Trigger::SessionEnds, false)]
    );
}

#[test]
fn the_last_run_of_each_rule_is_kept_across_a_restart() {
    let dir = TempDir::new("rule-runs");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert!(store.runs().is_empty());
    store
        .record_run(failed(Trigger::Sleeps, 1))
        .expect("record");
    store.record_run(failed(Trigger::Wakes, 2)).expect("record");
    store
        .record_run(failed(Trigger::Sleeps, 3))
        .expect("record");
    assert_eq!(stored(&dir, RUNS)["format"], json!("prismforge.rule-runs"));

    let reopened = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert_eq!(
        reopened.runs(),
        [failed(Trigger::Sleeps, 3), failed(Trigger::Wakes, 2)]
    );
}

#[test]
fn a_damaged_run_file_is_set_aside_and_never_blocks_the_store() {
    let dir = TempDir::new("rule-runs-damaged");
    dir.write(RUNS, b"{ not json");
    let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
    assert!(store.runs().is_empty());
    assert!(
        store.notices().iter().any(|notice| notice.file == RUNS
            && matches!(notice.event, NoticeEvent::StartedEmpty { .. }))
    );
    store.record_run(failed(Trigger::Quits, 5)).expect("record");
    assert_eq!(store.runs(), [failed(Trigger::Quits, 5)]);
}

/// The settings type of PrismForge before rules: the mark and whatever it does not know.
#[derive(serde::Serialize, serde::Deserialize)]
struct EarlierSettings {
    apply_on_connect: Option<ProfileId>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

#[test]
fn rules_survive_a_read_and_write_by_the_earlier_settings_type() {
    let (dir, id) = earlier_store("earlier-round-trip", true);
    {
        let mut store = ProfileStore::open(dir.path(), StdFileOps).expect("open");
        store
            .set_rule(Trigger::Sleeps, lights_off())
            .expect("sleep");
    }
    let mut fields = stored(&dir, SETTINGS);
    let object = fields.as_object_mut().expect("object");
    object.remove("format");
    object.remove("schema_version");
    let earlier: EarlierSettings = serde_json::from_value(fields).expect("earlier shape");
    let mut written = serde_json::to_value(&earlier).expect("encode");
    let object = written.as_object_mut().expect("object");
    object.insert("format".into(), json!("prismforge.settings"));
    object.insert("schema_version".into(), json!({"major": 1, "minor": 0}));
    dir.write(SETTINGS, &serde_json::to_vec(&written).expect("json"));

    let store = ProfileStore::open(dir.path(), StdFileOps).expect("reopen");
    assert_eq!(*store.rules().rule(Trigger::Sleeps), lights_off());
    assert_eq!(*store.rules().rule(Trigger::Connects), apply(&id));
}
