use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    document::{
        cas::CasStore,
        io_safe::{MAX_OSCAL_FILE_BYTES, atomic_write_bytes, read_bounded_utf8_file},
    },
    error::{AppError, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationKind {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalEntry {
    pub sequence_id: u64,
    pub target_file: PathBuf,
    pub mutation: MutationKind,
    pub before_content: Option<String>,
    #[serde(default)]
    pub before_sha256: Option<String>,
    #[serde(default)]
    pub pre_existed: bool,
    pub staged_content: Option<String>,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WriteAheadLog {
    pub transaction_id: String,
    pub status: String,
    pub created_at: String,
    pub entries: Vec<WalEntry>,
}

impl WriteAheadLog {
    pub fn new(tx_id: &str) -> Self {
        Self {
            transaction_id: tx_id.to_string(),
            status: "active".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            entries: Vec::new(),
        }
    }

    pub fn append_entry(
        &mut self,
        target_file: &Path,
        mutation: MutationKind,
        before_content: Option<String>,
        staged_content: Option<String>,
    ) {
        let pre_existed = before_content.is_some();
        self.append_entry_with_state(
            target_file,
            mutation,
            before_content,
            pre_existed,
            staged_content,
        );
    }

    pub fn append_entry_with_state(
        &mut self,
        target_file: &Path,
        mutation: MutationKind,
        before_content: Option<String>,
        pre_existed: bool,
        staged_content: Option<String>,
    ) {
        let seq = (self.entries.len() + 1) as u64;
        let before_sha256 = before_content
            .as_ref()
            .map(|c| CasStore::compute_digest(c.as_bytes()));
        let entry = WalEntry {
            sequence_id: seq,
            target_file: target_file.to_path_buf(),
            mutation,
            before_content,
            before_sha256,
            pre_existed,
            staged_content,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.entries.push(entry);
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        atomic_write_bytes(path, json_str.as_bytes())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = read_bounded_utf8_file(path, MAX_OSCAL_FILE_BYTES)?;
        serde_json::from_str(&content).map_err(|e| AppError::Configuration(e.to_string()))
    }
}
