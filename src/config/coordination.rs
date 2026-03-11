use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const fn default_coordination_enabled() -> bool {
    true
}

fn default_coordination_lead_agent() -> String {
    "delegate-lead".into()
}

const fn default_coordination_max_inbox_messages_per_agent() -> usize {
    256
}

const fn default_coordination_max_dead_letters() -> usize {
    256
}

const fn default_coordination_max_context_entries() -> usize {
    512
}

const fn default_coordination_max_seen_message_ids() -> usize {
    4096
}

/// Delegate coordination runtime configuration (`[coordination]` section).
///
/// Controls typed delegate message-bus integration used by `delegate` and
/// `delegate_coordination_status` tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordinationConfig {
    /// Enable delegate coordination tracing/runtime bus integration.
    #[serde(default = "default_coordination_enabled")]
    pub enabled: bool,
    /// Logical lead-agent identity used as coordinator sender/recipient.
    #[serde(default = "default_coordination_lead_agent")]
    pub lead_agent: String,
    /// Maximum retained inbox messages per registered agent.
    #[serde(default = "default_coordination_max_inbox_messages_per_agent")]
    pub max_inbox_messages_per_agent: usize,
    /// Maximum retained dead-letter entries.
    #[serde(default = "default_coordination_max_dead_letters")]
    pub max_dead_letters: usize,
    /// Maximum retained shared-context entries (`ContextPatch` state keys).
    #[serde(default = "default_coordination_max_context_entries")]
    pub max_context_entries: usize,
    /// Maximum retained dedupe window size for processed message IDs.
    #[serde(default = "default_coordination_max_seen_message_ids")]
    pub max_seen_message_ids: usize,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            enabled: default_coordination_enabled(),
            lead_agent: default_coordination_lead_agent(),
            max_inbox_messages_per_agent: default_coordination_max_inbox_messages_per_agent(),
            max_dead_letters: default_coordination_max_dead_letters(),
            max_context_entries: default_coordination_max_context_entries(),
            max_seen_message_ids: default_coordination_max_seen_message_ids(),
        }
    }
}
