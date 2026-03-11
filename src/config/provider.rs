use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Provider behavior overrides (`[provider]` section).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProviderConfig {
    /// Optional reasoning level override for providers that support explicit levels
    /// (e.g. OpenAI Codex `/responses` reasoning effort).
    #[serde(default)]
    pub reasoning_level: Option<String>,
}
