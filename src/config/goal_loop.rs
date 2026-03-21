use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stub configuration — goal loop subsystem has been removed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct GoalLoopConfig {
    #[serde(default)]
    pub enabled: bool,
}
