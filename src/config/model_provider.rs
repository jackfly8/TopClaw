use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Named provider profile definition compatible with Codex app-server style config.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ModelProviderConfig {
    /// Optional provider type/name override (e.g. "openai", "openai-codex", or custom profile id).
    #[serde(default)]
    pub name: Option<String>,
    /// Optional base URL for OpenAI-compatible endpoints.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Provider protocol variant ("responses" or "chat_completions").
    #[serde(default)]
    pub wire_api: Option<String>,
    /// If true, load OpenAI auth material (OPENAI_API_KEY or ~/.codex/auth.json).
    #[serde(default)]
    pub requires_openai_auth: bool,
}
