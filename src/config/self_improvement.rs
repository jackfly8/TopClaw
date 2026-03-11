use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for queued, candidate-only self-improvement automation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SelfImprovementConfig {
    /// Enable scheduled self-improvement automation. Default: `false`.
    #[serde(default)]
    pub enabled: bool,
    /// Absolute path to the stable TopClaw git checkout that seeds candidate worktrees.
    #[serde(default)]
    pub repository_path: Option<String>,
    /// Poll interval in minutes for the scheduled self-improvement cron job. Default: `30`.
    #[serde(default = "default_self_improvement_interval_minutes")]
    pub interval_minutes: u32,
    /// Push validated fixes to a remote branch automatically. Default: `true`.
    #[serde(default = "default_true")]
    pub auto_push_branch: bool,
    /// Open a draft PR automatically after a successful push. Default: `true`.
    #[serde(default = "default_true")]
    pub auto_open_draft_pr: bool,
    /// Prefix used for auto-generated user/task branches. Default: `users`.
    #[serde(default = "default_self_improvement_branch_prefix")]
    pub branch_prefix: String,
}

const fn default_true() -> bool {
    true
}

const fn default_self_improvement_interval_minutes() -> u32 {
    30
}

fn default_self_improvement_branch_prefix() -> String {
    "users".to_string()
}

impl Default for SelfImprovementConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            repository_path: None,
            interval_minutes: default_self_improvement_interval_minutes(),
            auto_push_branch: true,
            auto_open_draft_pr: true,
            branch_prefix: default_self_improvement_branch_prefix(),
        }
    }
}
