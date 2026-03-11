use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for the autonomous goal loop engine (`[goal_loop]`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalLoopConfig {
    /// Enable autonomous goal execution. Default: `false`.
    pub enabled: bool,
    /// Interval in minutes between goal loop cycles. Default: `10`.
    pub interval_minutes: u32,
    /// Timeout in seconds for a single step execution. Default: `120`.
    pub step_timeout_secs: u64,
    /// Maximum steps to execute per cycle. Default: `3`.
    pub max_steps_per_cycle: u32,
    /// Optional channel to deliver goal events to (e.g. "lark", "telegram").
    #[serde(default)]
    pub channel: Option<String>,
    /// Optional recipient/chat_id for goal event delivery.
    #[serde(default)]
    pub target: Option<String>,
}

impl Default for GoalLoopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 10,
            step_timeout_secs: 120,
            max_steps_per_cycle: 3,
            channel: None,
            target: None,
        }
    }
}
