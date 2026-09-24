use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    document::{
        parser::OscalDocument,
        tx::wal::{MutationKind, WriteAheadLog},
        validator::{ValidationOptions, validate_document},
    },
    error::{AppError, Result, io_error},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Active,
    Committed,
    RolledBack,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionReport {
    pub transaction_id: String,
    pub status: TransactionStatus,
    pub files_mutated: usize,
    pub wal_journal_path: PathBuf,
    pub validation_passed: bool,
    pub message: String,
}

pub struct TransactionSession {
    pub tx_id: String,
    pub wal_path: PathBuf,
    pub wal: WriteAheadLog,
    pub status: TransactionStatus,
}

impl TransactionSession {
    pub fn stage_create(&mut self, path: &Path, content: &str) -> Result<()> {
        if self.status != TransactionStatus::Active {
            return Err(AppError::Configuration(
                "Cannot mutate inactive transaction".to_string(),
            ));
        }
        self.wal
            .append_entry(path, MutationKind::Create, None, Some(content.to_string()));
        self.wal.save_to_file(&self.wal_path)?;
        Ok(())
    }

    pub fn stage_update(&mut self, path: &Path, new_content: &str) -> Result<()> {
        if self.status != TransactionStatus::Active {
            return Err(AppError::Configuration(
                "Cannot mutate inactive transaction".to_string(),
            ));
        }
        let before_content = if path.exists() {
            Some(fs::read_to_string(path).map_err(|e| io_error(path, e))?)
        } else {
            None
        };

        self.wal.append_entry(
            path,
            MutationKind::Update,
            before_content,
            Some(new_content.to_string()),
        );
        self.wal.save_to_file(&self.wal_path)?;
        Ok(())
    }

    pub fn stage_delete(&mut self, path: &Path) -> Result<()> {
        if self.status != TransactionStatus::Active {
            return Err(AppError::Configuration(
                "Cannot mutate inactive transaction".to_string(),
            ));
        }
        let before_content = if path.exists() {
            Some(fs::read_to_string(path).map_err(|e| io_error(path, e))?)
        } else {
            None
        };

        self.wal
            .append_entry(path, MutationKind::Delete, before_content, None);
        self.wal.save_to_file(&self.wal_path)?;
        Ok(())
    }

    pub fn verify_and_commit(&mut self) -> Result<TransactionReport> {
        if self.status != TransactionStatus::Active {
            return Err(AppError::Configuration(
                "Transaction is not in active state".to_string(),
            ));
        }

        let val_opts = ValidationOptions::default();

        let mut failed_file = None;
        for entry in &self.wal.entries {
            if let Some(content) = &entry.staged_content
                && (entry.target_file.extension().and_then(|e| e.to_str()) == Some("json")
                    || entry.target_file.extension().and_then(|e| e.to_str()) == Some("yaml"))
                && let Ok(doc) = OscalDocument::from_str(content, Some(entry.target_file.clone()))
            {
                let rep = validate_document(&doc, &val_opts)?;
                if !rep.is_valid {
                    failed_file = Some(entry.target_file.clone());
                    break;
                }
            }
        }

        if let Some(path) = failed_file {
            self.rollback()?;
            return Err(AppError::Configuration(format!(
                "Transaction validation failed on {}: Schema errors detected",
                path.display()
            )));
        }

        // 2. Commit Phase: Atomically write staged files to targets
        for entry in &self.wal.entries {
            match entry.mutation {
                MutationKind::Create | MutationKind::Update => {
                    if let Some(content) = &entry.staged_content {
                        if let Some(parent) = entry.target_file.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        fs::write(&entry.target_file, content)
                            .map_err(|e| io_error(&entry.target_file, e))?;
                    }
                }
                MutationKind::Delete => {
                    if entry.target_file.exists() {
                        let _ = fs::remove_file(&entry.target_file);
                    }
                }
            }
        }

        self.status = TransactionStatus::Committed;
        self.wal.status = "committed".to_string();
        self.wal.save_to_file(&self.wal_path)?;

        Ok(TransactionReport {
            transaction_id: self.tx_id.clone(),
            status: TransactionStatus::Committed,
            files_mutated: self.wal.entries.len(),
            wal_journal_path: self.wal_path.clone(),
            validation_passed: true,
            message: format!(
                "Successfully committed {} mutation(s)",
                self.wal.entries.len()
            ),
        })
    }

    pub fn rollback(&mut self) -> Result<TransactionReport> {
        // Restore all previous contents
        for entry in self.wal.entries.iter().rev() {
            match entry.mutation {
                MutationKind::Create => {
                    if entry.target_file.exists() {
                        let _ = fs::remove_file(&entry.target_file);
                    }
                }
                MutationKind::Update => {
                    if let Some(prev) = &entry.before_content {
                        if let Some(parent) = entry.target_file.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        fs::write(&entry.target_file, prev)
                            .map_err(|e| io_error(&entry.target_file, e))?;
                    }
                }
                MutationKind::Delete => {
                    if let Some(prev) = &entry.before_content {
                        if let Some(parent) = entry.target_file.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        fs::write(&entry.target_file, prev)
                            .map_err(|e| io_error(&entry.target_file, e))?;
                    }
                }
            }
        }

        self.status = TransactionStatus::RolledBack;
        self.wal.status = "rolled_back".to_string();
        self.wal.save_to_file(&self.wal_path)?;

        Ok(TransactionReport {
            transaction_id: self.tx_id.clone(),
            status: TransactionStatus::RolledBack,
            files_mutated: self.wal.entries.len(),
            wal_journal_path: self.wal_path.clone(),
            validation_passed: false,
            message: "Transaction safely rolled back to original state".to_string(),
        })
    }
}

pub struct ComplianceTransactionManager {
    journal_root: PathBuf,
}

impl Default for ComplianceTransactionManager {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("mizan-tx-journals"))
    }
}

impl ComplianceTransactionManager {
    pub fn new(journal_root: PathBuf) -> Self {
        let _ = fs::create_dir_all(&journal_root);
        Self { journal_root }
    }

    pub fn begin_transaction(&self, custom_id: Option<&str>) -> Result<TransactionSession> {
        let tx_id = custom_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("tx-{}", uuid::Uuid::new_v4()));
        let wal_path = self.journal_root.join(&tx_id).join("wal.json");
        let wal = WriteAheadLog::new(&tx_id);
        wal.save_to_file(&wal_path)?;

        Ok(TransactionSession {
            tx_id,
            wal_path,
            wal,
            status: TransactionStatus::Active,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_manager_commit_and_rollback() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-tx-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let manager = ComplianceTransactionManager::new(temp_dir.join("journals"));
        let mut session = manager.begin_transaction(Some("test-tx-01")).unwrap();

        let test_file = temp_dir.join("sample-catalog.json");
        let sample_valid_oscal = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "Transactional Catalog",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                }
            }
        }"#;

        session
            .stage_create(&test_file, sample_valid_oscal)
            .unwrap();
        let report = session.verify_and_commit().unwrap();

        assert_eq!(report.status, TransactionStatus::Committed);
        assert!(test_file.exists());

        // Now test rollback
        let mut session_update = manager.begin_transaction(Some("test-tx-02")).unwrap();
        let invalid_content = r#"{"invalid": true}"#;
        session_update
            .stage_update(&test_file, invalid_content)
            .unwrap();
        let rollback_rep = session_update.rollback().unwrap();

        assert_eq!(rollback_rep.status, TransactionStatus::RolledBack);
        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert!(restored_content.contains("Transactional Catalog"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
