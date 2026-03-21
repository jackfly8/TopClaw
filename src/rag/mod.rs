//! Stub — RAG subsystem removed. Types preserved for compatibility.

use std::path::Path;

/// A retrieved documentation chunk.
pub struct RagChunk {
    pub board: Option<String>,
    pub source: String,
    pub content: String,
}

/// Empty stub for hardware datasheet RAG.
#[derive(Debug, Clone)]
pub struct HardwareRag {
    _private: (),
}

impl HardwareRag {
    pub fn load(_workspace: &Path, _dir: &str) -> anyhow::Result<Self> {
        Ok(Self { _private: () })
    }

    pub fn is_empty(&self) -> bool {
        true
    }

    pub fn len(&self) -> usize {
        0
    }

    pub fn search(&self, _query: &str, _limit: usize) -> Vec<String> {
        Vec::new()
    }

    pub fn pin_alias_context(&self, _user_msg: &str, _boards: &[String]) -> String {
        String::new()
    }

    pub fn retrieve(
        &self,
        _user_msg: &str,
        _boards: &[String],
        _chunk_limit: usize,
    ) -> Vec<RagChunk> {
        Vec::new()
    }
}
