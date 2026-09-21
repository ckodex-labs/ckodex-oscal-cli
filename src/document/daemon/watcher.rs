use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::Duration,
};

use crate::{
    document::{
        cas::CasStore,
        fsm::{ComplianceState, EvidenceLevel, FsmEvent, FsmRuntime},
        linter::lint_document,
        parser::OscalDocument,
        validator::{validate_document, ValidationOptions},
    },
    error::{io_error, AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub watch_dir: PathBuf,
    pub debounce_ms: u64,
    pub auto_fsm: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            watch_dir: PathBuf::from("."),
            debounce_ms: 500,
            auto_fsm: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DaemonCycleReport {
    pub files_scanned: usize,
    pub valid_documents: usize,
    pub lint_issues: usize,
    pub cas_digests: Vec<String>,
    pub current_fsm_state: ComplianceState,
}

pub struct ComplianceDaemon {
    config: DaemonConfig,
    fsm: FsmRuntime,
    cas: CasStore,
}

impl ComplianceDaemon {
    pub fn new(config: DaemonConfig) -> Self {
        let fsm = FsmRuntime::new(ComplianceState::Draft, EvidenceLevel::E0Unverified);
        let cas = CasStore::default();
        Self { config, fsm, cas }
    }

    pub fn scan_and_reconcile(&mut self) -> Result<DaemonCycleReport> {
        let mut files_scanned = 0;
        let mut valid_documents = 0;
        let mut lint_issues = 0;
        let mut cas_digests = Vec::new();

        let val_opts = ValidationOptions::default();

        if self.config.watch_dir.exists() {
            for entry in walkdir(&self.config.watch_dir)? {
                let ext = entry.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "json" || ext == "yaml" {
                    files_scanned += 1;
                    if let Ok(mut doc) = OscalDocument::from_file(&entry) {
                        let rep = validate_document(&doc, &val_opts)?;
                        if rep.is_valid {
                            valid_documents += 1;
                        }
                        let l_rep = lint_document(&mut doc, false)?;
                        lint_issues += l_rep.issues.len();

                        if let Ok(bytes) = std::fs::read(&entry) {
                            if let Ok(digest) = self.cas.put_bytes(&bytes) {
                                cas_digests.push(digest);
                            }
                        }
                    }
                }
            }
        }

        // Auto-FSM State Evaluation
        if self.config.auto_fsm && valid_documents > 0 {
            if lint_issues == 0 && self.fsm.state() == ComplianceState::Draft {
                self.fsm.update_evidence(EvidenceLevel::E2LinterPassed);
                let _ = self.fsm.transition(
                    FsmEvent::SubmitForReview,
                    "daemon-autonomous-worker",
                    "Automated schema & lint checks passed cleanly",
                );
            } else if lint_issues > 0 && self.fsm.state() == ComplianceState::Approved {
                let _ = self.fsm.transition(
                    FsmEvent::AuditViolationDetected {
                        violation_count: lint_issues,
                    },
                    "daemon-autonomous-worker",
                    "Lint or invariant issues detected in working tree",
                );
            }
        }

        Ok(DaemonCycleReport {
            files_scanned,
            valid_documents,
            lint_issues,
            cas_digests,
            current_fsm_state: self.fsm.state(),
        })
    }

    pub async fn run_continuous(&mut self, max_cycles: Option<usize>) -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| AppError::Configuration(format!("Failed to create watcher: {e}")))?;

        if self.config.watch_dir.exists() {
            watcher
                .watch(&self.config.watch_dir, RecursiveMode::Recursive)
                .map_err(|e| AppError::Configuration(format!("Failed to watch directory: {e}")))?;
        }

        tracing::info!(
            "Mizan Compliance Daemon listening on {}",
            self.config.watch_dir.display()
        );

        let mut cycles = 0;
        // Initial scan
        let initial_rep = self.scan_and_reconcile()?;
        tracing::info!(
            "Initial scan: {} files, {} valid, {} issues",
            initial_rep.files_scanned,
            initial_rep.valid_documents,
            initial_rep.lint_issues
        );

        loop {
            if let Some(max) = max_cycles {
                if cycles >= max {
                    break;
                }
            }

            match rx.recv_timeout(Duration::from_millis(self.config.debounce_ms)) {
                Ok(Ok(Event { .. })) => {
                    cycles += 1;
                    let rep = self.scan_and_reconcile()?;
                    tracing::info!(
                        "Cycle {}: scanned {} docs, FSM state is {}",
                        cycles,
                        rep.files_scanned,
                        rep.current_fsm_state
                    );
                }
                Ok(Err(e)) => {
                    tracing::warn!("Watch event error: {e}");
                }
                Err(_) => {
                    // Timeout (debounce window / idle tick)
                    if max_cycles.is_some() {
                        cycles += 1;
                    }
                }
            }
        }

        Ok(())
    }
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)
            .map_err(|e| io_error(dir, e))?
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path)?);
            } else {
                files.push(path);
            }
        }
    } else {
        files.push(dir.to_path_buf());
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_scan_and_reconcile() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-daemon-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let sample_catalog = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "Continuous Daemon Sample",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "controls": [
                    { "id": "ac-1", "title": "Access Control" }
                ]
            }
        }"#;

        std::fs::write(temp_dir.join("catalog.json"), sample_catalog).unwrap();

        let config = DaemonConfig {
            watch_dir: temp_dir.clone(),
            debounce_ms: 100,
            auto_fsm: true,
        };

        let mut daemon = ComplianceDaemon::new(config);
        let rep = daemon.scan_and_reconcile().expect("scan should succeed");

        assert_eq!(rep.files_scanned, 1);
        assert_eq!(rep.valid_documents, 1);
        assert_eq!(rep.lint_issues, 0);
        assert_eq!(rep.current_fsm_state, ComplianceState::UnderReview);

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
