use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for a delegate sub-agent used by the `delegate` tool.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct DelegateAgentConfig {
    /// Provider name (e.g. "ollama", "openrouter", "anthropic")
    pub provider: String,
    /// Model name
    pub model: String,
    /// Optional system prompt for the sub-agent
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// Optional API key override
    #[serde(default)]
    pub api_key: Option<String>,
    /// Temperature override
    #[serde(default)]
    pub temperature: Option<f64>,
    /// Max recursion depth for nested delegation
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    /// Enable agentic sub-agent mode (multi-turn tool-call loop).
    #[serde(default)]
    pub agentic: bool,
    /// Allowlist of tool names available to the sub-agent in agentic mode.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Maximum tool-call iterations in agentic mode.
    #[serde(default = "default_max_tool_iterations")]
    pub max_iterations: usize,
}

fn default_max_depth() -> u32 {
    3
}

fn default_max_tool_iterations() -> usize {
    10
}

impl std::fmt::Debug for DelegateAgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegateAgentConfig")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("system_prompt", &self.system_prompt)
            .field("api_key_configured", &self.api_key.is_some())
            .field("temperature", &self.temperature)
            .field("max_depth", &self.max_depth)
            .field("agentic", &self.agentic)
            .field("allowed_tools", &self.allowed_tools)
            .field("max_iterations", &self.max_iterations)
            .finish()
    }
}
