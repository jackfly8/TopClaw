//! Stub — self-improvement subsystem has been removed.
//! Types preserved for API compatibility; all operations are no-ops.

use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOutcome {
    pub action: String,
    pub reason: Option<String>,
    pub task_id: Option<String>,
    pub job_id: Option<String>,
}

pub async fn sync_scheduled_job(_config: &Config) -> Result<SyncOutcome> {
    Ok(SyncOutcome {
        action: "disabled".to_string(),
        reason: Some("self-improvement subsystem removed".to_string()),
        task_id: None,
        job_id: None,
    })
}

pub fn check_git_readiness(_config: &Config) -> bool {
    false
}

pub async fn publish_draft_pr_for_task(
    _config: &Config,
    _task_id: &str,
    _workspace: &Path,
) -> Result<String> {
    anyhow::bail!("self-improvement subsystem removed")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelfImprovementTask {
    pub id: String,
    pub title: String,
    pub problem: String,
    pub evidence: Option<String>,
}

pub struct SelfImprovementManager {
    _workspace: std::path::PathBuf,
}

impl SelfImprovementManager {
    pub fn new(workspace: &Path) -> Self {
        Self {
            _workspace: workspace.to_path_buf(),
        }
    }

    pub async fn enqueue_task(
        &self,
        _config: &Config,
        _title: &str,
        _problem: &str,
        _evidence: Option<String>,
        _sender: Option<String>,
        _channel: Option<String>,
    ) -> Result<SelfImprovementTask> {
        anyhow::bail!("self-improvement subsystem removed")
    }

    pub async fn repair_state_file(&self) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({"status": "disabled", "reason": "self-improvement subsystem removed"}),
        )
    }
}
