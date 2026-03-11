use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reliability and supervision configuration (`[reliability]` section).
///
/// Controls provider retries, fallback chains, API key rotation, and channel restart backoff.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReliabilityConfig {
    /// Retries per provider before failing over.
    #[serde(default = "default_provider_retries")]
    pub provider_retries: u32,
    /// Base backoff (ms) for provider retry delay.
    #[serde(default = "default_provider_backoff_ms")]
    pub provider_backoff_ms: u64,
    /// Fallback provider chain (e.g. `["anthropic", "openai"]`).
    #[serde(default)]
    pub fallback_providers: Vec<String>,
    /// Optional per-fallback provider API keys keyed by fallback entry name.
    /// This allows distinct credentials for multiple `custom:<url>` endpoints.
    ///
    /// Contract:
    /// - Default/omitted (`{}` via `#[serde(default)]`): no per-entry override is used.
    /// - Compatibility: additive and non-breaking for existing configs that omit this field.
    /// - Rollback/migration: remove this map (or specific entries) to revert to provider/env-based
    ///   credential resolution.
    #[serde(default)]
    pub fallback_api_keys: HashMap<String, String>,
    /// Additional API keys for round-robin rotation on rate-limit (429) errors.
    /// The primary `api_key` is always tried first; these are extras.
    #[serde(default)]
    pub api_keys: Vec<String>,
    /// Per-model fallback chains. When a model fails, try these alternatives in order.
    /// Example: `{ "claude-opus-4-20250514" = ["claude-sonnet-4-20250514", "gpt-4o"] }`
    ///
    /// Compatibility behavior: keys matching configured provider names are treated
    /// as provider-scoped remap chains during provider fallback.
    #[serde(default)]
    pub model_fallbacks: HashMap<String, Vec<String>>,
    /// Initial backoff for channel/daemon restarts.
    #[serde(default = "default_channel_backoff_secs")]
    pub channel_initial_backoff_secs: u64,
    /// Max backoff for channel/daemon restarts.
    #[serde(default = "default_channel_backoff_max_secs")]
    pub channel_max_backoff_secs: u64,
    /// Scheduler polling cadence in seconds.
    #[serde(default = "default_scheduler_poll_secs")]
    pub scheduler_poll_secs: u64,
    /// Max retries for cron job execution attempts.
    #[serde(default = "default_scheduler_retries")]
    pub scheduler_retries: u32,
}

const fn default_provider_retries() -> u32 {
    2
}

const fn default_provider_backoff_ms() -> u64 {
    500
}

const fn default_channel_backoff_secs() -> u64 {
    2
}

const fn default_channel_backoff_max_secs() -> u64 {
    60
}

const fn default_scheduler_poll_secs() -> u64 {
    15
}

const fn default_scheduler_retries() -> u32 {
    2
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            provider_retries: default_provider_retries(),
            provider_backoff_ms: default_provider_backoff_ms(),
            fallback_providers: Vec::new(),
            fallback_api_keys: HashMap::new(),
            api_keys: Vec::new(),
            model_fallbacks: HashMap::new(),
            channel_initial_backoff_secs: default_channel_backoff_secs(),
            channel_max_backoff_secs: default_channel_backoff_max_secs(),
            scheduler_poll_secs: default_scheduler_poll_secs(),
            scheduler_retries: default_scheduler_retries(),
        }
    }
}
