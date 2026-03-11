use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Web fetch tool configuration (`[web_fetch]` section).
///
/// Fetches web pages and converts HTML to plain text for LLM consumption.
/// Domain filtering: `allowed_domains` controls which hosts are reachable (use `["*"]`
/// for all public hosts). `blocked_domains` takes priority over `allowed_domains`.
/// If `allowed_domains` is empty, all requests are rejected (deny-by-default).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebFetchConfig {
    /// Enable `web_fetch` tool for fetching web page content
    #[serde(default)]
    pub enabled: bool,
    /// Provider: "fast_html2md", "nanohtml2text", "firecrawl", or "tavily"
    #[serde(default = "default_web_fetch_provider")]
    pub provider: String,
    /// Optional provider API key (required for provider = "firecrawl" or "tavily").
    /// Multiple keys can be comma-separated for round-robin load balancing.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Optional provider API URL override (for self-hosted providers)
    #[serde(default)]
    pub api_url: Option<String>,
    /// Allowed domains for web fetch (exact or subdomain match; `["*"]` = all public hosts)
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Blocked domains (exact or subdomain match; always takes priority over allowed_domains)
    #[serde(default)]
    pub blocked_domains: Vec<String>,
    /// Maximum response size in bytes (default: 500KB, plain text is much smaller than raw HTML)
    #[serde(default = "default_web_fetch_max_response_size")]
    pub max_response_size: usize,
    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_web_fetch_timeout_secs")]
    pub timeout_secs: u64,
    /// User-Agent string sent with fetch requests (env: TOPCLAW_WEB_FETCH_USER_AGENT)
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

fn default_web_fetch_max_response_size() -> usize {
    500_000
}

fn default_web_fetch_provider() -> String {
    "fast_html2md".into()
}

fn default_web_fetch_timeout_secs() -> u64 {
    30
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_web_fetch_provider(),
            api_key: None,
            api_url: None,
            allowed_domains: vec!["*".into()],
            blocked_domains: vec![],
            max_response_size: default_web_fetch_max_response_size(),
            timeout_secs: default_web_fetch_timeout_secs(),
            user_agent: default_user_agent(),
        }
    }
}

/// Web search tool configuration (`[web_search]` section).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebSearchConfig {
    /// Enable `web_search_tool` for web searches
    #[serde(default)]
    pub enabled: bool,
    /// Search provider: "duckduckgo" (free, no API key), "searxng" (self-hosted), "brave", "firecrawl", or "tavily"
    #[serde(default = "default_web_search_provider")]
    pub provider: String,
    /// Generic provider API key (used by firecrawl, tavily, and as fallback for brave).
    /// Multiple keys can be comma-separated for round-robin load balancing.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Optional provider API URL override (for self-hosted providers)
    #[serde(default)]
    pub api_url: Option<String>,
    /// Brave Search API key (required if provider is "brave")
    #[serde(default)]
    pub brave_api_key: Option<String>,
    /// Maximum results per search (1-10)
    #[serde(default = "default_web_search_max_results")]
    pub max_results: usize,
    /// Request timeout in seconds
    #[serde(default = "default_web_search_timeout_secs")]
    pub timeout_secs: u64,
    /// User-Agent string sent with search requests (env: TOPCLAW_WEB_SEARCH_USER_AGENT)
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

fn default_web_search_provider() -> String {
    "duckduckgo".into()
}

fn default_web_search_max_results() -> usize {
    5
}

fn default_web_search_timeout_secs() -> u64 {
    15
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_web_search_provider(),
            api_key: None,
            api_url: None,
            brave_api_key: None,
            max_results: default_web_search_max_results(),
            timeout_secs: default_web_search_timeout_secs(),
            user_agent: default_user_agent(),
        }
    }
}

fn default_user_agent() -> String {
    "TopClaw/1.0".into()
}
