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
const MAX_WALK_DEPTH: usize = 16;

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
    pub invalid_documents: usize,
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

    pub fn with_cas(mut self, cas: CasStore) -> Self {
        self.cas = cas;
        self
    }

    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    pub fn fsm(&self) -> &FsmRuntime {
        &self.fsm
    }

    pub fn fsm_mut(&mut self) -> &mut FsmRuntime {
        &mut self.fsm
    }

    pub fn cas(&self) -> &CasStore {
        &self.cas
    }

    pub fn scan_and_reconcile(&mut self) -> Result<DaemonCycleReport> {
        let mut files_scanned = 0;
        let mut valid_documents = 0;
        let mut invalid_documents = 0;
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
                invalid_documents: 0,
                corrupt_documents: 0,
                lint_issues: 0,
                cas_digests: Vec::new(),
                current_fsm_state: self.fsm.state(),
            });
        }

        for entry in walkdir(&self.config.watch_dir) {
            let ext = entry
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if matches!(ext.as_str(), "json" | "yaml" | "yml") {
                files_scanned += 1;
                match OscalDocument::from_file(&entry) {
                    Ok(mut doc) => {
                        let mut is_schema_valid = false;
                        match validate_document(&doc, &val_opts) {
                            Ok(rep) => {
                                if rep.is_valid {
                                    valid_documents += 1;
                                    is_schema_valid = true;
                                } else {
                                    invalid_documents += 1;
                                    tracing::debug!(
                                        "Document '{}' has {} validation errors",
                                        entry.display(),
                                        rep.error_count()
                                    );
                                }
                            }
                            Err(e) => {
                                invalid_documents += 1;
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

                        if is_schema_valid {
                            match crate::document::io_safe::read_bounded_regular_file(
                                &entry,
                                crate::document::io_safe::MAX_OSCAL_FILE_BYTES,
                            ) {
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
        if self.config.auto_fsm {
            if valid_documents > 0
                && invalid_documents == 0
                && lint_issues == 0
                && corrupt_documents == 0
                && self.fsm.state() == ComplianceState::Draft
            {
                self.fsm.update_evidence(EvidenceLevel::E2LinterPassed);
                let _ = self.fsm.transition(
                    FsmEvent::SubmitForReview,
                    "daemon-autonomous-worker",
                    "Automated schema & lint checks passed cleanly",
                );
            } else if (lint_issues > 0 || corrupt_documents > 0 || invalid_documents > 0)
                && (self.fsm.state() == ComplianceState::Approved
                    || self.fsm.state() == ComplianceState::Operational)
            {
                let violation_count = lint_issues + corrupt_documents + invalid_documents;
                let _ = self.fsm.transition(
                    FsmEvent::AuditViolationDetected { violation_count },
                    "daemon-autonomous-worker",
                    "Violations, invalid documents, or corrupt documents detected in working tree",
                );
            }
        }

        let report = DaemonCycleReport {
            files_scanned,
            valid_documents,
            invalid_documents,
            corrupt_documents,
            lint_issues,
            cas_digests,
            current_fsm_state: self.fsm.state(),
        };

        tracing::info!(
            "Reconciliation cycle diagnostics: scanned={}, valid={}, invalid={}, corrupt={}, lint_issues={}, fsm_state={}",
            report.files_scanned,
            report.valid_documents,
            report.invalid_documents,
            report.corrupt_documents,
            report.lint_issues,
            report.current_fsm_state
        );

        Ok(report)
    }

    pub async fn run_continuous(&mut self, max_cycles: Option<usize>) -> Result<()> {
        let _ = self.run_continuous_with_shutdown(max_cycles, None).await?;
        Ok(())
    }

    pub async fn run_continuous_with_shutdown(
        &mut self,
        max_cycles: Option<usize>,
        stop_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) -> Result<DaemonCycleReport> {
        use std::sync::{atomic::Ordering, mpsc::RecvTimeoutError};

        let (tx, rx) = channel();
        let debounce_ms = self.config.bounded_debounce_ms();
        let debounce = Duration::from_millis(debounce_ms);
        let quiet_window = Duration::from_millis(debounce_ms.min(100));
        let poll_interval = Duration::from_millis(debounce_ms.clamp(50, 500));

        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(poll_interval),
        )
        .map_err(|e| AppError::Configuration(format!("Failed to create watcher: {e}")))?;

        let mut watch_attached = false;
        if self.config.watch_dir.exists() {
            match watcher.watch(&self.config.watch_dir, RecursiveMode::Recursive) {
                Ok(()) => watch_attached = true,
                Err(e) => {
                    tracing::warn!(
                        "Failed to watch directory '{}': {e}; continuing in periodic reconciliation mode",
                        self.config.watch_dir.display()
                    );
                }
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
        let mut last_report = match self.scan_and_reconcile() {
            Ok(initial_rep) => {
                tracing::info!(
                    "Initial scan diagnostics: {} files scanned, {} valid, {} invalid, {} corrupt, {} lint issues",
                    initial_rep.files_scanned,
                    initial_rep.valid_documents,
                    initial_rep.invalid_documents,
                    initial_rep.corrupt_documents,
                    initial_rep.lint_issues
                );
                initial_rep
            }
            Err(e) => {
                tracing::warn!("Initial scan error contained: {e}");
                DaemonCycleReport {
                    files_scanned: 0,
                    valid_documents: 0,
                    invalid_documents: 0,
                    corrupt_documents: 0,
                    lint_issues: 0,
                    cas_digests: Vec::new(),
                    current_fsm_state: self.fsm.state(),
                }
            }
        };

        let mut last_reconcile = std::time::Instant::now();
        let mut disconnected_warned = false;

        loop {
            if let Some(ref flag) = stop_flag
                && flag.load(Ordering::Relaxed)
            {
                break;
            }
            if let Some(max) = max_cycles
                && cycles >= max
            {
                break;
            }

            let wait_slice = if stop_flag.is_some() {
                debounce.min(Duration::from_millis(200))
            } else {
                debounce
            };

            match rx.recv_timeout(wait_slice) {
                Ok(Ok(Event { .. })) => {
                    // Coalesce burst events over a short quiet debounce window so mid-save
                    // editor truncations settle before reading.
                    for _ in 0..10 {
                        match rx.recv_timeout(quiet_window) {
                            Ok(Ok(_)) => while let Ok(Ok(_)) = rx.try_recv() {},
                            _ => break,
                        }
                    }

                    cycles += 1;
                    last_reconcile = std::time::Instant::now();
                    match self.scan_and_reconcile() {
                        Ok(rep) => {
                            tracing::info!(
                                "Cycle {} diagnostics: scanned={}, valid={}, invalid={}, corrupt={}, lint_issues={}, fsm_state={}",
                                cycles,
                                rep.files_scanned,
                                rep.valid_documents,
                                rep.invalid_documents,
                                rep.corrupt_documents,
                                rep.lint_issues,
                                rep.current_fsm_state
                            );
                            last_report = rep;
                        }
                        Err(e) => {
                            tracing::error!("Cycle {} reconciliation error contained: {e}", cycles);
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("Watch event error: {e}");
                }
                Err(RecvTimeoutError::Timeout) => {
                    if last_reconcile.elapsed() < debounce {
                        continue;
                    }
                    if !watch_attached
                        && self.config.watch_dir.exists()
                        && watcher
                            .watch(&self.config.watch_dir, RecursiveMode::Recursive)
                            .is_ok()
                    {
                        watch_attached = true;
                    }
                    if max_cycles.is_some() || !watch_attached {
                        cycles += 1;
                        last_reconcile = std::time::Instant::now();
                        match self.scan_and_reconcile() {
                            Ok(rep) => {
                                tracing::info!(
                                    "Periodic cycle {} diagnostics: scanned={}, valid={}, invalid={}, corrupt={}, lint_issues={}, fsm_state={}",
                                    cycles,
                                    rep.files_scanned,
                                    rep.valid_documents,
                                    rep.invalid_documents,
                                    rep.corrupt_documents,
                                    rep.lint_issues,
                                    rep.current_fsm_state
                                );
                                last_report = rep;
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Periodic cycle {} reconciliation error contained: {e}",
                                    cycles
                                );
                            }
                        }
                    } else {
                        last_reconcile = std::time::Instant::now();
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    if !disconnected_warned {
                        tracing::warn!(
                            "File watcher channel disconnected; falling back to bounded sleep without spinning"
                        );
                        disconnected_warned = true;
                    }
                    watch_attached = false;
                    if max_cycles.is_none() && stop_flag.is_none() {
                        break;
                    }
                    tokio::time::sleep(wait_slice).await;
                    if last_reconcile.elapsed() >= debounce {
                        cycles += 1;
                        last_reconcile = std::time::Instant::now();
                        if let Ok(rep) = self.scan_and_reconcile() {
                            last_report = rep;
                        }
                    }
                }
            }
        }

        Ok(last_report)
    }
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walkdir_bounded(dir, 0, &mut files);
    files
}

fn walkdir_bounded(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    if depth > MAX_WALK_DEPTH {
        return;
    }
    let meta = if depth == 0 {
        let Ok(m) = std::fs::metadata(dir) else {
            return;
        };
        m
    } else {
        let Ok(m) = std::fs::symlink_metadata(dir) else {
            return;
        };
        if m.file_type().is_symlink() {
            return;
        }
        m
    };
    if meta.file_type().is_dir() {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if matches!(
                        file_name,
                        ".git" | "node_modules" | "target" | ".next" | ".mizan"
                    ) {
                        continue;
                    }
                    let Ok(entry_meta) = std::fs::symlink_metadata(&path) else {
                        continue;
                    };
                    if entry_meta.file_type().is_symlink() {
                        continue;
                    }
                    if entry_meta.file_type().is_dir() {
                        walkdir_bounded(&path, depth + 1, files);
                    } else if entry_meta.file_type().is_file() {
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
    } else if meta.file_type().is_file() {
        files.push(dir.to_path_buf());
    }
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
        assert_eq!(rep.invalid_documents, 0);
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
        assert_eq!(rep.invalid_documents, 0);
        assert_eq!(rep.corrupt_documents, 0);
    }

    #[test]
    fn test_daemon_schema_invalid_blocks_promotion_and_skips_symlink_loops() {
        let temp_dir = std::env::temp_dir().join(format!(
            "mizan-daemon-invalid-symlink-{}",
            uuid::Uuid::new_v4()
        ));
        let cas_dir = temp_dir.join("isolated-cas");
        let watch_dir = temp_dir.join("workspace");
        std::fs::create_dir_all(&watch_dir).unwrap();

        // 1. Valid catalog + schema-invalid catalog (missing metadata & invalid uuid)
        std::fs::write(watch_dir.join("good.json"), SAMPLE_CATALOG).unwrap();
        let invalid_catalog = r#"{
            "catalog": {
                "uuid": "not-a-valid-uuid"
            }
        }"#;
        std::fs::write(watch_dir.join("invalid.yml"), invalid_catalog).unwrap();

        // 2. Symlink cycle inside workspace
        #[cfg(unix)]
        {
            let loop_link = watch_dir.join("loop_self");
            std::os::unix::fs::symlink(&watch_dir, &loop_link).unwrap();
        }

        let config = DaemonConfig {
            watch_dir: watch_dir.clone(),
            debounce_ms: 50,
            auto_fsm: true,
        };

        let mut daemon = ComplianceDaemon::new(config).with_cas(CasStore::new(cas_dir));
        let rep = daemon
            .scan_and_reconcile()
            .expect("scan should succeed despite symlink loop and invalid doc");

        assert_eq!(rep.files_scanned, 2);
        assert_eq!(rep.valid_documents, 1);
        assert_eq!(rep.invalid_documents, 1);
        assert_eq!(
            rep.cas_digests.len(),
            1,
            "invalid documents must NOT be cached in CAS"
        );
        assert_eq!(
            rep.current_fsm_state,
            ComplianceState::Draft,
            "Draft -> UnderReview promotion must be blocked when invalid_documents > 0"
        );

        // 3. When state is Approved or Operational, invalid_documents transitions -> Degraded
        *daemon.fsm_mut() =
            FsmRuntime::new(ComplianceState::Approved, EvidenceLevel::E3FedrampPassed);
        let rep_deg = daemon.scan_and_reconcile().unwrap();
        assert_eq!(rep_deg.current_fsm_state, ComplianceState::Degraded);

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
