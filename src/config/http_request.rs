use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// HTTP request tool configuration (`[http_request]` section).
///
/// Deny-by-default: if `allowed_domains` is empty, all HTTP requests are rejected.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HttpRequestConfig {
    /// Enable `http_request` tool for API interactions
    #[serde(default)]
    pub enabled: bool,
    /// Allowed domains for HTTP requests (exact or subdomain match)
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Maximum response size in bytes (default: 1MB, 0 = unlimited)
    #[serde(default = "default_http_max_response_size")]
    pub max_response_size: usize,
    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_http_timeout_secs")]
    pub timeout_secs: u64,
    /// User-Agent string sent with HTTP requests (env: TOPCLAW_HTTP_REQUEST_USER_AGENT)
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

impl Default for HttpRequestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_domains: vec![],
            max_response_size: default_http_max_response_size(),
            timeout_secs: default_http_timeout_secs(),
            user_agent: default_user_agent(),
        }
    }
}

fn default_http_max_response_size() -> usize {
    1_000_000
}

fn default_http_timeout_secs() -> u64 {
    30
}

fn default_user_agent() -> String {
    format!("TopClaw/{}", env!("CARGO_PKG_VERSION"))
}
