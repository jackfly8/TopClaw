use crate::config::traits::ChannelConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_bridge_bind_host() -> String {
    "127.0.0.1".into()
}

const fn default_bridge_bind_port() -> u16 {
    8765
}

fn default_bridge_path() -> String {
    "/ws".into()
}

fn default_bridge_auth_token() -> String {
    String::new()
}

const fn default_bridge_max_connections() -> usize {
    64
}

/// Bridge WebSocket channel configuration.
///
/// This listener is local-only by default (`127.0.0.1`) for safety.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BridgeConfig {
    /// Local bind host for the bridge listener.
    #[serde(default = "default_bridge_bind_host")]
    pub bind_host: String,
    /// TCP port for incoming websocket bridge clients.
    #[serde(default = "default_bridge_bind_port")]
    pub bind_port: u16,
    /// HTTP path for websocket upgrade requests.
    #[serde(default = "default_bridge_path")]
    pub path: String,
    /// Shared bearer token required from bridge websocket clients.
    ///
    /// Empty default means bridge auth is not configured yet; listener startup
    /// will fail fast until this is explicitly set.
    #[serde(default = "default_bridge_auth_token")]
    pub auth_token: String,
    /// Allowlisted sender IDs that can authenticate over bridge.
    ///
    /// Empty list is deny-by-default.
    #[serde(default)]
    pub allowed_senders: Vec<String>,
    /// Allow non-localhost binds.
    ///
    /// Defaults to `false`; public bind addresses require an explicit opt-in.
    #[serde(default)]
    pub allow_public_bind: bool,
    /// Maximum concurrent websocket bridge connections.
    #[serde(default = "default_bridge_max_connections")]
    pub max_connections: usize,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            bind_host: default_bridge_bind_host(),
            bind_port: default_bridge_bind_port(),
            path: default_bridge_path(),
            auth_token: default_bridge_auth_token(),
            allowed_senders: Vec::new(),
            allow_public_bind: false,
            max_connections: default_bridge_max_connections(),
        }
    }
}

impl ChannelConfig for BridgeConfig {
    fn name() -> &'static str {
        "Bridge"
    }

    fn desc() -> &'static str {
        "Local websocket bridge"
    }
}
