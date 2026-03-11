use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_agents_ipc_db_path() -> String {
    "~/.topclaw/agents.db".into()
}

const fn default_agents_ipc_staleness_secs() -> u64 {
    300
}

/// Inter-process agent communication configuration (`[agents_ipc]` section).
///
/// When enabled, registers IPC tools that let independent TopClaw processes
/// on the same host discover each other and exchange messages via a shared
/// SQLite database. Disabled by default (zero overhead when off).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentsIpcConfig {
    /// Enable inter-process agent communication tools.
    #[serde(default)]
    pub enabled: bool,
    /// Path to shared SQLite database (all agents on this host share one file).
    #[serde(default = "default_agents_ipc_db_path")]
    pub db_path: String,
    /// Agents not seen within this window are considered offline (seconds).
    #[serde(default = "default_agents_ipc_staleness_secs")]
    pub staleness_secs: u64,
}

impl Default for AgentsIpcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            db_path: default_agents_ipc_db_path(),
            staleness_secs: default_agents_ipc_staleness_secs(),
        }
    }
}
