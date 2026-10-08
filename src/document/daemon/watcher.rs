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
        validator::{ValidationOptions, validate_document},
    },
    error::{AppError, Result},
};

pub const MIN_DEBOUNCE_MS: u64 = 50;
pub const MAX_DEBOUNCE_MS: u64 = 60_000;
pub const DEFAULT_DEBOUNCE_MS: u64 = 500;

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
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            auto_fsm: true,
        }
    }
}

impl DaemonConfig {
    pub fn new(watch_dir: PathBuf, debounce_ms: u64, auto_fsm: bool) -> Self {
        Self {
            watch_dir,
            debounce_ms: debounce_ms.clamp(MIN_DEBOUNCE_MS, MAX_DEBOUNCE_MS),
            auto_fsm,
        }
    }

    pub fn bounded_debounce_ms(&self) -> u64 {
        self.debounce_ms.clamp(MIN_DEBOUNCE_MS, MAX_DEBOUNCE_MS)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DaemonCycleReport {
    pub files_scanned: usize,
    pub valid_documents: usize,
    #[serde(default)]
    pub corrupt_documents: usize,
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
    pub fn new(mut config: DaemonConfig) -> Self {
        let clamped = config.bounded_debounce_ms();
        if clamped != config.debounce_ms {
            tracing::warn!(
                "Debounce interval {} ms out of bounds [{}..{}]; clamped to {} ms",
                config.debounce_ms,
                MIN_DEBOUNCE_MS,
                MAX_DEBOUNCE_MS,
                clamped
            );
            config.debounce_ms = clamped;
        }
        let fsm = FsmRuntime::new(ComplianceState::Draft, EvidenceLevel::E0Unverified);
        let cas = CasStore::default();
        Self { config, fsm, cas }
    }

    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    pub fn fsm(&self) -> &FsmRuntime {
        &self.fsm
    }

    pub fn cas(&self) -> &CasStore {
        &self.cas
    }

    pub fn scan_and_reconcile(&mut self) -> Result<DaemonCycleReport> {
        let mut files_scanned = 0;
        let mut valid_documents = 0;
        let mut corrupt_documents = 0;
        let mut lint_issues = 0;
        let mut cas_digests = Vec::new();

        let val_opts = ValidationOptions::default();

        if !self.config.watch_dir.exists() {
            tracing::warn!(
                "Watch directory '{}' does not exist; skipping scan cycle",
                self.config.watch_dir.display()
            );
            return Ok(DaemonCycleReport {
                files_scanned: 0,
                valid_documents: 0,
                corrupt_documents: 0,
                lint_issues: 0,
                cas_digests: Vec::new(),
                current_fsm_state: self.fsm.state(),
            });
        }

        for entry in walkdir(&self.config.watch_dir) {
            let ext = entry.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "json" || ext == "yaml" {
                files_scanned += 1;
                match OscalDocument::from_file(&entry) {
                    Ok(mut doc) => {
                        match validate_document(&doc, &val_opts) {
                            Ok(rep) => {
                                if rep.is_valid {
                                    valid_documents += 1;
                                } else {
                                    tracing::debug!(
                                        "Document '{}' has {} validation errors",
                                        entry.display(),
                                        rep.error_count()
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Validation execution error for '{}': {e}",
                                    entry.display()
                                );
                            }
                        }

                        match lint_document(&mut doc, false) {
                            Ok(l_rep) => {
                                lint_issues += l_rep.issues.len();
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Linter execution error for '{}': {e}",
                                    entry.display()
                                );
                            }
                        }

                        match std::fs::read(&entry) {
                            Ok(bytes) => match self.cas.put_bytes(&bytes) {
                                Ok(digest) => cas_digests.push(digest),
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to store '{}' in CAS: {e}",
                                        entry.display()
                                    );
                                }
                            },
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to read file bytes for CAS from '{}': {e}",
                                    entry.display()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        corrupt_documents += 1;
                        tracing::warn!(
                            "Failed to parse OSCAL document at '{}': {e} (syntax error contained)",
                            entry.display()
                        );
                    }
                }
            }
        }

        // Auto-FSM State Evaluation
        if self.config.auto_fsm && valid_documents > 0 {
            if lint_issues == 0
                && corrupt_documents == 0
                && self.fsm.state() == ComplianceState::Draft
            {
                self.fsm.update_evidence(EvidenceLevel::E2LinterPassed);
                let _ = self.fsm.transition(
                    FsmEvent::SubmitForReview,
                    "daemon-autonomous-worker",
                    "Automated schema & lint checks passed cleanly",
                );
            } else if (lint_issues > 0 || corrupt_documents > 0)
                && (self.fsm.state() == ComplianceState::Approved
                    || self.fsm.state() == ComplianceState::Operational)
            {
                let violation_count = lint_issues + corrupt_documents;
                let _ = self.fsm.transition(
                    FsmEvent::AuditViolationDetected { violation_count },
                    "daemon-autonomous-worker",
                    "Violations or corrupt documents detected in working tree",
                );
            }
        }

        let report = DaemonCycleReport {
            files_scanned,
            valid_documents,
            corrupt_documents,
            lint_issues,
            cas_digests,
            current_fsm_state: self.fsm.state(),
        };

        tracing::info!(
            "Reconciliation cycle diagnostics: scanned={}, valid={}, corrupt={}, lint_issues={}, fsm_state={}",
            report.files_scanned,
            report.valid_documents,
            report.corrupt_documents,
            report.lint_issues,
            report.current_fsm_state
        );

        Ok(report)
    }

    pub async fn run_continuous(&mut self, max_cycles: Option<usize>) -> Result<()> {
        let (tx, rx) = channel();
        let debounce = Duration::from_millis(self.config.bounded_debounce_ms());
        let poll_interval = Duration::from_millis(self.config.bounded_debounce_ms().clamp(50, 500));

        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(poll_interval),
        )
        .map_err(|e| AppError::Configuration(format!("Failed to create watcher: {e}")))?;

        if self.config.watch_dir.exists() {
            if let Err(e) = watcher.watch(&self.config.watch_dir, RecursiveMode::Recursive) {
                tracing::warn!(
                    "Failed to watch directory '{}': {e}; continuing in periodic reconciliation mode",
                    self.config.watch_dir.display()
                );
            }
        } else {
            tracing::warn!(
                "Watch directory '{}' does not exist yet; continuing in periodic reconciliation mode",
                self.config.watch_dir.display()
            );
        }

        tracing::info!(
            "Mizan Compliance Daemon listening on {} (debounce: {} ms)",
            self.config.watch_dir.display(),
            debounce.as_millis()
        );

        let mut cycles = 0;
        // Initial scan with error containment
        match self.scan_and_reconcile() {
            Ok(initial_rep) => {
                tracing::info!(
                    "Initial scan diagnostics: {} files scanned, {} valid, {} corrupt, {} lint issues",
                    initial_rep.files_scanned,
                    initial_rep.valid_documents,
                    initial_rep.corrupt_documents,
                    initial_rep.lint_issues
                );
            }
            Err(e) => {
                tracing::warn!("Initial scan error contained: {e}");
            }
        }

        loop {
            if let Some(max) = max_cycles
                && cycles >= max
            {
                break;
            }

            match rx.recv_timeout(debounce) {
                Ok(Ok(Event { .. })) => {
                    // Drain any queued events within debounce window
                    while let Ok(Ok(_)) = rx.try_recv() {}

                    cycles += 1;
                    match self.scan_and_reconcile() {
                        Ok(rep) => {
                            tracing::info!(
                                "Cycle {} diagnostics: scanned={}, valid={}, corrupt={}, lint_issues={}, fsm_state={}",
                                cycles,
                                rep.files_scanned,
                                rep.valid_documents,
                                rep.corrupt_documents,
                                rep.lint_issues,
                                rep.current_fsm_state
                            );
                        }
                        Err(e) => {
                            tracing::error!("Cycle {} reconciliation error contained: {e}", cycles);
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("Watch event error: {e}");
                }
                Err(_) => {
                    // Timeout (debounce window / idle tick)
                    if max_cycles.is_some() {
                        cycles += 1;
                        match self.scan_and_reconcile() {
                            Ok(rep) => {
                                tracing::info!(
                                    "Periodic cycle {} diagnostics: scanned={}, valid={}, corrupt={}, lint_issues={}, fsm_state={}",
                                    cycles,
                                    rep.files_scanned,
                                    rep.valid_documents,
                                    rep.corrupt_documents,
                                    rep.lint_issues,
                                    rep.current_fsm_state
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Periodic cycle {} reconciliation error contained: {e}",
                                    cycles
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    if dir.is_dir() {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(walkdir(&path));
                    } else {
                        files.push(path);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to read directory entries at '{}': {e}",
                    dir.display()
                );
            }
        }
    } else {
        files.push(dir.to_path_buf());
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CATALOG: &str = r#"{
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

    #[test]
    fn test_daemon_scan_and_reconcile() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-daemon-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        std::fs::write(temp_dir.join("catalog.json"), SAMPLE_CATALOG).unwrap();

        let config = DaemonConfig {
            watch_dir: temp_dir.clone(),
            debounce_ms: 100,
            auto_fsm: true,
        };

        let mut daemon = ComplianceDaemon::new(config);
        let rep = daemon.scan_and_reconcile().expect("scan should succeed");

        assert_eq!(rep.files_scanned, 1);
        assert_eq!(rep.valid_documents, 1);
        assert_eq!(rep.corrupt_documents, 0);
        assert_eq!(rep.lint_issues, 0);
        assert_eq!(rep.current_fsm_state, ComplianceState::UnderReview);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_daemon_recovers_from_invalid_syntax() {
        let temp_dir = std::env::temp_dir().join(format!(
            "mizan-daemon-recovery-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let cat_path = temp_dir.join("catalog.json");

        // 1. Initial valid catalog
        std::fs::write(&cat_path, SAMPLE_CATALOG).unwrap();

        let config = DaemonConfig {
            watch_dir: temp_dir.clone(),
            debounce_ms: 50,
            auto_fsm: true,
        };

        let mut daemon = ComplianceDaemon::new(config);
        let rep1 = daemon.scan_and_reconcile().expect("initial scan succeeds");
        assert_eq!(rep1.valid_documents, 1);
        assert_eq!(rep1.corrupt_documents, 0);
        assert_eq!(rep1.current_fsm_state, ComplianceState::UnderReview);

        // 2. Corrupt file: invalid syntax / partial write
        std::fs::write(
            &cat_path,
            "{ corrupt json syntax: missing quotes and brackets",
        )
        .unwrap();

        let rep2 = daemon
            .scan_and_reconcile()
            .expect("scan with corrupt file must not crash");
        assert_eq!(rep2.files_scanned, 1);
        assert_eq!(rep2.valid_documents, 0);
        assert_eq!(rep2.corrupt_documents, 1);
        // FSM state is preserved from cycle 1
        assert_eq!(rep2.current_fsm_state, ComplianceState::UnderReview);

        // 3. Recovery: restore valid catalog syntax
        std::fs::write(&cat_path, SAMPLE_CATALOG).unwrap();

        let rep3 = daemon.scan_and_reconcile().expect("recovery scan succeeds");
        assert_eq!(rep3.files_scanned, 1);
        assert_eq!(rep3.valid_documents, 1);
        assert_eq!(rep3.corrupt_documents, 0);
        assert_eq!(rep3.current_fsm_state, ComplianceState::UnderReview);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_daemon_watch_loop_resilience_to_corrupt_file() {
        let temp_dir = std::env::temp_dir().join(format!(
            "mizan-daemon-loop-resilience-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Place a corrupt file before loop starts
        std::fs::write(temp_dir.join("corrupt.json"), "{{{{ not valid json at all").unwrap();

        let config = DaemonConfig {
            watch_dir: temp_dir.clone(),
            debounce_ms: 50,
            auto_fsm: true,
        };

        let mut daemon = ComplianceDaemon::new(config);
        // Running 1 cycle of continuous loop must complete cleanly without crash
        let res = daemon.run_continuous(Some(1)).await;
        assert!(
            res.is_ok(),
            "daemon run_continuous must not fail on corrupt syntax"
        );

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_daemon_debounce_interval_bounding() {
        let config_under = DaemonConfig {
            watch_dir: PathBuf::from("."),
            debounce_ms: 10,
            auto_fsm: false,
        };
        let daemon_under = ComplianceDaemon::new(config_under);
        assert_eq!(daemon_under.config().debounce_ms, MIN_DEBOUNCE_MS);

        let config_over = DaemonConfig {
            watch_dir: PathBuf::from("."),
            debounce_ms: 500_000,
            auto_fsm: false,
        };
        let daemon_over = ComplianceDaemon::new(config_over);
        assert_eq!(daemon_over.config().debounce_ms, MAX_DEBOUNCE_MS);

        let config_valid = DaemonConfig {
            watch_dir: PathBuf::from("."),
            debounce_ms: 1_000,
            auto_fsm: false,
        };
        let daemon_valid = ComplianceDaemon::new(config_valid);
        assert_eq!(daemon_valid.config().debounce_ms, 1_000);
    }

    #[test]
    fn test_daemon_handles_nonexistent_directory() {
        let non_existent = PathBuf::from("/nonexistent/mizan/path/that/does/not/exist");
        let config = DaemonConfig {
            watch_dir: non_existent,
            debounce_ms: 100,
            auto_fsm: true,
        };
        let mut daemon = ComplianceDaemon::new(config);
        let res = daemon.scan_and_reconcile();
        assert!(res.is_ok());
        let rep = res.unwrap();
        assert_eq!(rep.files_scanned, 0);
        assert_eq!(rep.valid_documents, 0);
        assert_eq!(rep.corrupt_documents, 0);
    }
}
