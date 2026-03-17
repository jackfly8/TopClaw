use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_identity_format() -> String {
    "bootstrap".into()
}

/// Identity format configuration (`[identity]` section).
///
/// Supports `"bootstrap"` (default, markdown files) or `"aieos"` (JSON) identity documents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IdentityConfig {
    /// Identity format: "bootstrap" (default, markdown) or "aieos" (JSON)
    #[serde(default = "default_identity_format")]
    pub format: String,
    /// Path to AIEOS JSON file (relative to workspace)
    #[serde(default)]
    pub aieos_path: Option<String>,
    /// Inline AIEOS JSON (alternative to file path)
    #[serde(default)]
    pub aieos_inline: Option<String>,
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            format: default_identity_format(),
            aieos_path: None,
            aieos_inline: None,
        }
    }
}
