use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! setup_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// The prefix, then lower-case letters, digits and hyphens only; at most 64 characters.
            pub fn parse(value: &str) -> Option<Self> {
                let valid = (3..=64).contains(&value.len())
                    && value.starts_with($prefix)
                    && value.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    });
                valid.then(|| Self(value.to_owned()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) fn is_valid(&self) -> bool {
                Self::parse(&self.0).is_some()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

setup_id!(ComponentId, "c-");
setup_id!(OutputId, "o-");
setup_id!(ViewId, "v-");
