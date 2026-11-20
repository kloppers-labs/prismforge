use prismforge_core::automation::{OpenRgbAct, RuleAction, RuleProblem, RuleRun, Trigger};
use prismforge_core::profile::ProfileId;
use serde::Serialize;

use crate::automation::OpenRgbCheck;
use crate::automation::run::is_live;
use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::stores::StoreKind;
use crate::system_events::EventSupport;
use crate::view::CommandError;

/// Everything the Automation screen shows: one rule per event, in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct AutomationView {
    pub store: StoreKind,
    /// Whether this system reports sleep, wake and session end to PrismForge.
    pub support: EventSupport,
    pub rules: Vec<RuleView>,
    /// The profiles a rule can apply, by name.
    pub profiles: Vec<ProfileChoice>,
    pub openrgb: OpenRgbCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct RuleView {
    pub trigger: Trigger,
    pub enabled: bool,
    pub action: RuleAction,
    /// The profile the action names, when it names one.
    pub profile: Option<RuleProfile>,
    /// Why the rule cannot run as stored, if it cannot.
    pub problem: Option<RuleProblem>,
    pub last_run: Option<RuleRun>,
    /// OpenRGB's own settings that act on the same event.
    pub openrgb_also: Vec<OpenRgbAct>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RuleProfile {
    Ready {
        name: String,
    },
    /// The profile was deleted; the rule reports it and never picks another one.
    Missing,
    NeedsNewer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ProfileChoice {
    pub id: ProfileId,
    pub name: String,
}

pub(crate) async fn build<S: ViewSink>(
    shell: &Shell<S>,
    support: EventSupport,
) -> Result<AutomationView, CommandError> {
    let (handle, kind) = shell.current().await;
    let profiles = shell.stores.profiles(kind).await?;
    let openrgb = match &handle {
        Some(handle) if is_live(&handle.snapshot().connection) => {
            match shell.checks.get(handle.session_id()) {
                OpenRgbCheck::NotChecked => shell.checks.refresh(handle).await,
                check => check,
            }
        }
        _ => OpenRgbCheck::NotChecked,
    };
    drop(handle);
    let summary = profiles.summary(None).await;
    let rules = profiles.rules().await;
    let runs = profiles.runs().await;
    let mut choices: Vec<ProfileChoice> = summary
        .profiles
        .iter()
        .map(|profile| ProfileChoice {
            id: profile.id.clone(),
            name: profile.name.clone(),
        })
        .collect();
    choices.sort_by_cached_key(|choice| choice.name.to_lowercase());
    let rules = Trigger::ALL
        .into_iter()
        .map(|trigger| {
            let rule = rules.rule(trigger);
            let profile = match &rule.action {
                RuleAction::ApplyProfile { profile } => Some(
                    match summary.profiles.iter().find(|stored| &stored.id == profile) {
                        Some(stored) => RuleProfile::Ready {
                            name: stored.name.clone(),
                        },
                        None if summary
                            .unreadable
                            .iter()
                            .any(|locked| &locked.id == profile) =>
                        {
                            RuleProfile::NeedsNewer
                        }
                        None => RuleProfile::Missing,
                    },
                ),
                _ => None,
            };
            RuleView {
                trigger,
                enabled: rule.enabled,
                action: rule.action.clone(),
                profile,
                problem: rule.check(trigger).err(),
                last_run: runs.iter().find(|run| run.trigger == trigger).cloned(),
                openrgb_also: match &openrgb {
                    OpenRgbCheck::Read { profiles } => profiles.acting_on(trigger),
                    _ => Vec::new(),
                },
            }
        })
        .collect();
    Ok(AutomationView {
        store: kind,
        support,
        rules,
        profiles: choices,
        openrgb,
    })
}
