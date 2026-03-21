//! Stub registry — returns an empty integration list.

use super::{IntegrationCategory, IntegrationStatus};
use crate::config::Config;

pub struct IntegrationEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub category: IntegrationCategory,
    pub status_fn: fn(&Config) -> IntegrationStatus,
}

pub fn all_integrations() -> Vec<IntegrationEntry> {
    Vec::new()
}
