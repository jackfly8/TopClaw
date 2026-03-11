use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Route a task hint to a specific provider + model.
///
/// ```toml
/// [[model_routes]]
/// hint = "reasoning"
/// provider = "openrouter"
/// model = "anthropic/claude-opus-4-20250514"
///
/// [[model_routes]]
/// hint = "fast"
/// provider = "groq"
/// model = "llama-3.3-70b-versatile"
/// ```
///
/// Usage: pass `hint:reasoning` as the model parameter to route the request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelRouteConfig {
    /// Task hint name (e.g. "reasoning", "fast", "code", "summarize")
    pub hint: String,
    /// Provider to route to (must match a known provider name)
    pub provider: String,
    /// Model to use with that provider
    pub model: String,
    /// Optional max_tokens override for this route.
    /// When set, provider requests cap output tokens to this value.
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Optional API key override for this route's provider
    #[serde(default)]
    pub api_key: Option<String>,
}
