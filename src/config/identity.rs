use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_identity_format() -> String {
    "bootstrap".into()
}

/// Identity format configuration (`[identity]` section).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct IdentityConfig {
    /// Identity format: "bootstrap" (markdown) is the only supported format.
    #[serde(default = "default_identity_format")]
    pub format: String,
}
