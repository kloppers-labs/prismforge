use serde::{Deserialize, Serialize};

use crate::automation::Rules;
use crate::profile::{Extra, ProfileId};

/// Store-wide choices of the user.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Settings {
    /// The profile the connects rule applies, kept for versions that know only this mark.
    pub apply_on_connect: Option<ProfileId>,
    /// Absent until the first rule change; until then the rules come from `apply_on_connect`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation: Option<Rules>,
    #[serde(flatten)]
    pub extra: Extra,
}
