use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Gateway server configuration (`[gateway]` section).
///
/// Controls the HTTP gateway for webhook and pairing endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GatewayConfig {
    /// Gateway port (default: 42617)
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    /// Gateway host (default: 127.0.0.1)
    #[serde(default = "default_gateway_host")]
    pub host: String,
    /// Require pairing before accepting requests (default: true)
    #[serde(default = "default_true")]
    pub require_pairing: bool,
    /// Allow binding to non-localhost without a tunnel (default: false)
    #[serde(default)]
    pub allow_public_bind: bool,
    /// Paired bearer tokens (managed automatically, not user-edited)
    #[serde(default)]
    pub paired_tokens: Vec<String>,

    /// Max `/pair` requests per minute per client key.
    #[serde(default = "default_pair_rate_limit")]
    pub pair_rate_limit_per_minute: u32,

    /// Max `/webhook` requests per minute per client key.
    #[serde(default = "default_webhook_rate_limit")]
    pub webhook_rate_limit_per_minute: u32,

    /// Trust proxy-forwarded client IP headers (`X-Forwarded-For`, `X-Real-IP`).
    /// Disabled by default; enable only behind a trusted reverse proxy.
    #[serde(default)]
    pub trust_forwarded_headers: bool,

    /// Trusted reverse proxy CIDRs allowed to supply forwarded client IP headers.
    /// Loopback proxies remain trusted when forwarded headers are enabled.
    #[serde(default)]
    pub trusted_proxy_cidrs: Vec<String>,

    /// Maximum distinct client keys tracked by gateway rate limiter maps.
    #[serde(default = "default_gateway_rate_limit_max_keys")]
    pub rate_limit_max_keys: usize,

    /// TTL for webhook idempotency keys.
    #[serde(default = "default_idempotency_ttl_secs")]
    pub idempotency_ttl_secs: u64,

    /// Maximum distinct idempotency keys retained in memory.
    #[serde(default = "default_gateway_idempotency_max_keys")]
    pub idempotency_max_keys: usize,

    /// Node-control protocol scaffold (`[gateway.node_control]`).
    #[serde(default)]
    pub node_control: NodeControlConfig,
}

/// Node-control scaffold settings under `[gateway.node_control]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct NodeControlConfig {
    /// Enable experimental node-control API endpoints.
    #[serde(default)]
    pub enabled: bool,

    /// Optional extra shared token for node-control API calls.
    /// When set, clients must send this value in `X-Node-Control-Token`.
    #[serde(default)]
    pub auth_token: Option<String>,

    /// Allowlist of remote node IDs for `node.describe`/`node.invoke`.
    /// Empty means "no explicit allowlist" (accept all IDs).
    #[serde(default)]
    pub allowed_node_ids: Vec<String>,
}

const fn default_gateway_port() -> u16 {
    42617
}

fn default_gateway_host() -> String {
    "127.0.0.1".into()
}

const fn default_pair_rate_limit() -> u32 {
    10
}

const fn default_webhook_rate_limit() -> u32 {
    60
}

const fn default_idempotency_ttl_secs() -> u64 {
    300
}

const fn default_gateway_rate_limit_max_keys() -> usize {
    10_000
}

const fn default_gateway_idempotency_max_keys() -> usize {
    10_000
}

const fn default_true() -> bool {
    true
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: default_gateway_port(),
            host: default_gateway_host(),
            require_pairing: true,
            allow_public_bind: false,
            paired_tokens: Vec::new(),
            pair_rate_limit_per_minute: default_pair_rate_limit(),
            webhook_rate_limit_per_minute: default_webhook_rate_limit(),
            trust_forwarded_headers: false,
            trusted_proxy_cidrs: Vec::new(),
            rate_limit_max_keys: default_gateway_rate_limit_max_keys(),
            idempotency_ttl_secs: default_idempotency_ttl_secs(),
            idempotency_max_keys: default_gateway_idempotency_max_keys(),
            node_control: NodeControlConfig::default(),
        }
    }
}
