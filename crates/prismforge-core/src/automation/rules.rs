use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::profile::{Extra, ProfileId};

/// The events PrismForge can act on; each has exactly one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Trigger {
    Starts,
    Connects,
    /// Sleep and hibernate are one event: Windows gives one signal for both.
    Sleeps,
    Wakes,
    /// Sign-out, shutdown and restart.
    SessionEnds,
    Quits,
}

impl Trigger {
    pub const ALL: [Self; 6] = [
        Self::Starts,
        Self::Connects,
        Self::Sleeps,
        Self::Wakes,
        Self::SessionEnds,
        Self::Quits,
    ];

    /// Switching the lights off is for events after which the lights should stay dark.
    pub fn allows_lights_off(self) -> bool {
        matches!(self, Self::Sleeps | Self::SessionEnds | Self::Quits)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RuleAction {
    #[default]
    Nothing,
    LightsOff,
    ApplyProfile {
        profile: ProfileId,
    },
    /// An action a newer PrismForge wrote; this version keeps it and never runs it.
    Newer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum RuleProblem {
    #[error("switching the lights off is not an action for this event")]
    LightsOffNotForThisEvent,
    #[error("this action needs a newer PrismForge")]
    NeedsNewerPrismForge,
}

/// "When <event>, <action>": acts only after the user switched it on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rule {
    pub enabled: bool,
    pub action: RuleAction,
    pub extra: Extra,
    /// The stored action when it is [`RuleAction::Newer`], written back as it was.
    pub newer: Option<Value>,
}

impl Rule {
    pub fn check(&self, trigger: Trigger) -> Result<(), RuleProblem> {
        match self.action {
            RuleAction::Newer => Err(RuleProblem::NeedsNewerPrismForge),
            RuleAction::LightsOff if !trigger.allows_lights_off() => {
                Err(RuleProblem::LightsOffNotForThisEvent)
            }
            _ => Ok(()),
        }
    }

    /// The action to run now, if the rule is on and this version can run it.
    pub fn due(&self, trigger: Trigger) -> Option<&RuleAction> {
        let runnable = self.enabled && self.action != RuleAction::Nothing;
        (runnable && self.check(trigger).is_ok()).then_some(&self.action)
    }
}

#[derive(Serialize, Deserialize)]
struct StoredRule {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    action: Value,
    #[serde(flatten)]
    extra: Extra,
}

impl Serialize for Rule {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let action = match (&self.action, &self.newer) {
            (RuleAction::Newer, Some(raw)) => raw.clone(),
            (action, _) => serde_json::to_value(action).map_err(serde::ser::Error::custom)?,
        };
        StoredRule {
            enabled: self.enabled,
            action,
            extra: self.extra.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let stored = StoredRule::deserialize(deserializer)?;
        let (action, newer) = match stored.action {
            Value::Null => (RuleAction::Nothing, None),
            raw => match serde_json::from_value::<RuleAction>(raw.clone()) {
                Ok(RuleAction::Newer) | Err(_) => (RuleAction::Newer, Some(raw)),
                Ok(action) => (action, None),
            },
        };
        Ok(Self {
            enabled: stored.enabled,
            action,
            extra: stored.extra,
            newer,
        })
    }
}

/// One rule for each event; events a newer version added are kept in `extra`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rules {
    #[serde(default)]
    starts: Rule,
    #[serde(default)]
    connects: Rule,
    #[serde(default)]
    sleeps: Rule,
    #[serde(default)]
    wakes: Rule,
    #[serde(default)]
    session_ends: Rule,
    #[serde(default)]
    quits: Rule,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Rules {
    /// The rules for a store whose only automation was the mark "Apply this profile when PrismForge connects".
    pub fn migrated(mark: Option<ProfileId>) -> Self {
        let mut rules = Self::default();
        if let Some(profile) = mark {
            rules.connects = Rule {
                enabled: true,
                action: RuleAction::ApplyProfile { profile },
                ..Rule::default()
            };
        }
        rules
    }

    pub fn rule(&self, trigger: Trigger) -> &Rule {
        match trigger {
            Trigger::Starts => &self.starts,
            Trigger::Connects => &self.connects,
            Trigger::Sleeps => &self.sleeps,
            Trigger::Wakes => &self.wakes,
            Trigger::SessionEnds => &self.session_ends,
            Trigger::Quits => &self.quits,
        }
    }

    pub fn set(&mut self, trigger: Trigger, rule: Rule) {
        *self.rule_mut(trigger) = rule;
    }

    /// The profile the connects rule applies, written beside the rules for versions that know only the mark.
    pub fn mark(&self) -> Option<&ProfileId> {
        match self.connects.due(Trigger::Connects) {
            Some(RuleAction::ApplyProfile { profile }) => Some(profile),
            _ => None,
        }
    }

    /// Takes over a mark an older version changed: the connects rule follows it.
    pub(crate) fn follow_mark(&mut self, mark: Option<ProfileId>) {
        if self.mark() == mark.as_ref() {
            return;
        }
        match mark {
            Some(profile) => {
                self.connects.enabled = true;
                self.connects.action = RuleAction::ApplyProfile { profile };
                self.connects.newer = None;
            }
            None => self.connects.enabled = false,
        }
    }

    fn rule_mut(&mut self, trigger: Trigger) -> &mut Rule {
        match trigger {
            Trigger::Starts => &mut self.starts,
            Trigger::Connects => &mut self.connects,
            Trigger::Sleeps => &mut self.sleeps,
            Trigger::Wakes => &mut self.wakes,
            Trigger::SessionEnds => &mut self.session_ends,
            Trigger::Quits => &mut self.quits,
        }
    }
}
