use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stub configuration — self-improvement subsystem has been removed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct SelfImprovementConfig {
    #[serde(default)]
    pub enabled: bool,
}
