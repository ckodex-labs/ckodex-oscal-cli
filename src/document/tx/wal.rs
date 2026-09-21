use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{io_error, AppError, Result};

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
        let seq = (self.entries.len() + 1) as u64;
        let entry = WalEntry {
            sequence_id: seq,
            target_file: target_file.to_path_buf(),
            mutation,
            before_content,
            staged_content,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.entries.push(entry);
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        fs::write(path, json_str).map_err(|e| io_error(path, e))
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| io_error(path, e))?;
        serde_json::from_str(&content).map_err(|e| AppError::Configuration(e.to_string()))
    }
}
