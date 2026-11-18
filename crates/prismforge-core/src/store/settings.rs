use serde::{Deserialize, Serialize};

use crate::profile::{Extra, ProfileId};

/// Store-wide choices of the user.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Settings {
    /// The one profile applied when PrismForge connects; none by default.
    pub apply_on_connect: Option<ProfileId>,
    #[serde(flatten)]
    pub extra: Extra,
}
