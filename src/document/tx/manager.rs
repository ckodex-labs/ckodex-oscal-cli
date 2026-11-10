use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    document::{
        io_safe::{MAX_OSCAL_FILE_BYTES, atomic_write_bytes, read_bounded_utf8_file},
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
        let pre_existed = path.exists();
        let before_content = if pre_existed {
            Some(read_bounded_utf8_file(path, MAX_OSCAL_FILE_BYTES)?)
        } else {
            None
        };
        self.wal.append_entry_with_state(
            path,
            MutationKind::Create,
            before_content,
            pre_existed,
            Some(content.to_string()),
        );
        self.wal.save_to_file(&self.wal_path)?;
        Ok(())
    }

    pub fn stage_update(&mut self, path: &Path, new_content: &str) -> Result<()> {
        if self.status != TransactionStatus::Active {
            return Err(AppError::Configuration(
                "Cannot mutate inactive transaction".to_string(),
            ));
        }
        let pre_existed = path.exists();
        let before_content = if pre_existed {
            Some(read_bounded_utf8_file(path, MAX_OSCAL_FILE_BYTES)?)
        } else {
            None
        };

        self.wal.append_entry_with_state(
            path,
            MutationKind::Update,
            before_content,
            pre_existed,
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
        let pre_existed = path.exists();
        let before_content = if pre_existed {
            Some(read_bounded_utf8_file(path, MAX_OSCAL_FILE_BYTES)?)
        } else {
            None
        };

        self.wal.append_entry_with_state(
            path,
            MutationKind::Delete,
            before_content,
            pre_existed,
            None,
        );
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

        let mut failed_validation: Option<(PathBuf, String)> = None;
        for entry in &self.wal.entries {
            if let Some(content) = &entry.staged_content
                && let Some(ext) = entry.target_file.extension().and_then(|e| e.to_str())
                && matches!(ext.to_ascii_lowercase().as_str(), "json" | "yaml" | "yml")
            {
                match OscalDocument::from_str(content, Some(entry.target_file.clone())) {
                    Ok(doc) => {
                        let rep = validate_document(&doc, &val_opts)?;
                        if !rep.is_valid {
                            failed_validation = Some((
                                entry.target_file.clone(),
                                "Schema errors detected".to_string(),
                            ));
                            break;
                        }
                    }
                    Err(e) => {
                        failed_validation = Some((entry.target_file.clone(), e.to_string()));
                        break;
                    }
                }
            }
        }

        if let Some((path, reason)) = failed_validation {
            self.rollback_entries(0)?;
            return Err(AppError::Configuration(format!(
                "Transaction validation failed on {}: {reason}",
                path.display()
            )));
        }

        // 2. Commit Phase: Atomically write staged files to targets; roll back applied prefix on partial failure
        let mut applied_count = 0usize;
        let mut commit_error: Option<AppError> = None;
        for (idx, entry) in self.wal.entries.iter().enumerate() {
            let step_res: Result<()> = match entry.mutation {
                MutationKind::Create | MutationKind::Update => {
                    if let Some(content) = &entry.staged_content {
                        atomic_write_bytes(&entry.target_file, content.as_bytes())
                    } else {
                        Ok(())
                    }
                }
                MutationKind::Delete => {
                    if entry.target_file.exists() {
                        fs::remove_file(&entry.target_file)
                            .map_err(|e| io_error(&entry.target_file, e))
                    } else {
                        Ok(())
                    }
                }
            };
            if let Err(err) = step_res {
                commit_error = Some(err);
                break;
            }
            applied_count = idx + 1;
        }

        if let Some(commit_err) = commit_error {
            let _ = self.rollback_entries(applied_count);
            return Err(commit_err);
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
        let total = self.wal.entries.len();
        self.rollback_entries(total)
    }

    fn rollback_entries(&mut self, limit: usize) -> Result<TransactionReport> {
        let bounded_limit = limit.min(self.wal.entries.len());
        let mut rollback_errors: Vec<String> = Vec::new();

        // Restore all applied entries safely in reverse order without early-aborting on a single entry failure
        for entry in self.wal.entries[..bounded_limit].iter().rev() {
            let already_matches_before = if let Some(expected_sha) = &entry.before_sha256
                && let Ok(current_bytes) = crate::document::io_safe::read_bounded_regular_file(
                    &entry.target_file,
                    MAX_OSCAL_FILE_BYTES,
                ) {
                crate::document::cas::CasStore::compute_digest(&current_bytes) == *expected_sha
            } else {
                false
            };

            if already_matches_before {
                continue;
            }

            let res: Result<()> = match entry.mutation {
                MutationKind::Create => {
                    if !entry.pre_existed {
                        if entry.target_file.exists() {
                            fs::remove_file(&entry.target_file)
                                .map_err(|e| io_error(&entry.target_file, e))
                        } else {
                            Ok(())
                        }
                    } else if let Some(prev) = &entry.before_content {
                        atomic_write_bytes(&entry.target_file, prev.as_bytes())
                    } else {
                        Ok(())
                    }
                }
                MutationKind::Update => {
                    if let Some(prev) = &entry.before_content {
                        atomic_write_bytes(&entry.target_file, prev.as_bytes())
                    } else if !entry.pre_existed && entry.target_file.exists() {
                        fs::remove_file(&entry.target_file)
                            .map_err(|e| io_error(&entry.target_file, e))
                    } else {
                        Ok(())
                    }
                }
                MutationKind::Delete => {
                    if let Some(prev) = &entry.before_content {
                        atomic_write_bytes(&entry.target_file, prev.as_bytes())
                    } else {
                        Ok(())
                    }
                }
            };

            if let Err(err) = res {
                rollback_errors.push(format!("{}: {err}", entry.target_file.display()));
            }
        }

        if !rollback_errors.is_empty() {
            self.status = TransactionStatus::Failed;
            self.wal.status = "failed".to_string();
            let _ = self.wal.save_to_file(&self.wal_path);
            return Err(AppError::Configuration(format!(
                "Transaction rollback encountered {} error(s): {}",
                rollback_errors.len(),
                rollback_errors.join("; ")
            )));
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

fn validate_tx_id(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains("..")
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('\0')
        || trimmed.chars().any(char::is_control)
    {
        return Err(AppError::InvalidArgument(format!(
            "Invalid transaction ID '{raw}': must not be empty or contain path separators, '..', or control characters"
        )));
    }
    Ok(trimmed.to_string())
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
        let tx_id = match custom_id {
            Some(raw) => validate_tx_id(raw)?,
            None => format!("tx-{}", uuid::Uuid::new_v4()),
        };
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

    #[test]
    fn test_tx_rejects_malformed_json_and_non_oscal_on_commit_and_preserves_existing() {
        let temp_dir = std::env::temp_dir().join(format!(
            "mizan-tx-validation-bypass-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let manager = ComplianceTransactionManager::new(temp_dir.join("journals"));

        // Reject unsafe custom_id values
        assert!(manager.begin_transaction(Some("")).is_err());
        assert!(manager.begin_transaction(Some("../escape")).is_err());
        assert!(manager.begin_transaction(Some("foo/bar")).is_err());
        assert!(manager.begin_transaction(Some("bad\0id")).is_err());

        let existing_file = temp_dir.join("existing-catalog.json");
        let valid_oscal = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "Pre-Existing Baseline",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                }
            }
        }"#;
        fs::write(&existing_file, valid_oscal).unwrap();

        // 1. Malformed JSON syntax in staged update must fail verify_and_commit and roll back cleanly
        let mut s1 = manager
            .begin_transaction(Some("tx-malformed-json"))
            .unwrap();
        s1.stage_update(&existing_file, "{ broken json")
            .expect("stage_update succeeds");
        let err1 = s1
            .verify_and_commit()
            .expect_err("malformed JSON must abort commit");
        assert!(err1.to_string().contains("Transaction validation failed"));
        assert_eq!(s1.status, TransactionStatus::RolledBack);
        assert!(
            fs::read_to_string(&existing_file)
                .unwrap()
                .contains("Pre-Existing Baseline")
        );

        // 2. Valid JSON that is NOT an OSCAL model must ALSO fail verify_and_commit
        let mut s2 = manager.begin_transaction(Some("tx-non-oscal")).unwrap();
        s2.stage_create(&existing_file, r#"{"not_oscal": true}"#)
            .unwrap();
        let err2 = s2
            .verify_and_commit()
            .expect_err("non-OSCAL JSON must abort commit");
        assert!(err2.to_string().contains("Transaction validation failed"));
        assert_eq!(s2.status, TransactionStatus::RolledBack);
        // Pre-existing file must NOT be deleted when stage_create rolls back!
        assert!(
            existing_file.exists(),
            "pre-existing file must not be deleted by rollback of stage_create"
        );
        assert!(
            fs::read_to_string(&existing_file)
                .unwrap()
                .contains("Pre-Existing Baseline")
        );

        // 3. Malformed .yml file must also fail validation
        let yml_file = temp_dir.join("broken.yml");
        let mut s3 = manager.begin_transaction(Some("tx-broken-yml")).unwrap();
        s3.stage_create(&yml_file, "::: invalid: [yaml").unwrap();
        assert!(s3.verify_and_commit().is_err());
        assert!(!yml_file.exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_tx_partial_multi_file_commit_failure_rolls_back_earlier_mutations() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-tx-partial-commit-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let manager = ComplianceTransactionManager::new(temp_dir.join("journals"));
        let file1 = temp_dir.join("catalog-one.json");
        let file2 = temp_dir.join("catalog-two.json");
        let file3_created = temp_dir.join("catalog-three-new.json");

        let baseline_1 = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "Baseline One",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                }
            }
        }"#;
        let updated_valid = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "Updated Mutated Catalog",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "2.0.0",
                    "oscal-version": "1.2.3"
                }
            }
        }"#;
        fs::write(&file1, baseline_1).unwrap();
        fs::write(&file2, baseline_1).unwrap();

        let mut session = manager.begin_transaction(Some("tx-partial-fail")).unwrap();
        session.stage_update(&file1, updated_valid).unwrap();
        session.stage_create(&file3_created, updated_valid).unwrap();
        session.stage_update(&file2, updated_valid).unwrap();

        // Replace file2 with a non-empty directory after staging so Entry 1 and Entry 2 succeed,
        // and Entry 3 fails during atomic rename in verify_and_commit!
        fs::remove_file(&file2).unwrap();
        fs::create_dir_all(file2.join("blocker")).unwrap();

        let commit_err = session
            .verify_and_commit()
            .expect_err("commit must fail when file2 was replaced by a directory");
        let _ = commit_err;

        assert_eq!(
            session.status,
            TransactionStatus::RolledBack,
            "session must transition to RolledBack after rolling back applied prefix"
        );
        assert!(
            fs::read_to_string(&file1).unwrap().contains("Baseline One"),
            "Entry 1 must be rolled back to its original Baseline One content"
        );
        assert!(
            !file3_created.exists(),
            "Entry 2 (newly created file) must be deleted during partial-commit rollback"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}
