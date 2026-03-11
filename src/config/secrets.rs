use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Secrets encryption configuration (`[secrets]` section).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecretsConfig {
    /// Enable encryption for API keys and tokens in config.toml
    #[serde(default = "default_true")]
    pub encrypt: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self { encrypt: true }
    }
}
