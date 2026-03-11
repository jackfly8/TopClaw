use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Multi-workspace registry configuration (`[workspaces]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct WorkspacesConfig {
    /// Enables in-process workspace registry behavior.
    #[serde(default)]
    pub enabled: bool,
    /// Optional workspace registry root override.
    /// If omitted, defaults to `<config_dir>/workspaces`.
    #[serde(default)]
    pub root: Option<String>,
}

impl WorkspacesConfig {
    /// Resolve the workspace registry root from config and runtime context.
    pub fn resolve_root(&self, config_dir: &Path) -> PathBuf {
        match self
            .root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(value) => {
                let expanded = shellexpand::tilde(value).into_owned();
                let path = PathBuf::from(expanded);
                if path.is_absolute() {
                    path
                } else {
                    config_dir.join(path)
                }
            }
            None => config_dir.join("workspaces"),
        }
    }
}
