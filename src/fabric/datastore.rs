// Mizan Root Fabric · Isolated Multi-Tenant High-Assurance DataStore
// Cryptographically isolated storage partitioned strictly by TenantId and Namespace

use crate::fabric::tenant::{TenantContext, TenantError, TenantId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

type DocumentKey = (TenantId, String, String);
type DocumentMap = HashMap<DocumentKey, StoredDocument>;

#[derive(Debug, thiserror::Error)]
pub enum DataStoreError {
    #[error("Tenant error: {0}")]
    Tenant(#[from] TenantError),
    #[error("Document not found: {id} in namespace {namespace}")]
    NotFound { id: String, namespace: String },
    #[error("Document already exists: {id} in namespace {namespace}")]
    AlreadyExists { id: String, namespace: String },
    #[error("Storage integrity error: {0}")]
    Integrity(String),
    #[error("Datastore lock poisoned: {0}")]
    Lock(String),
    #[error("Storage I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Storage serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Stored OSCAL Artifact Record with Cryptographic Lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocument {
    pub document_id: String,
    pub tenant_id: TenantId,
    pub namespace: String,
    pub resource_type: String,
    pub version: u64,
    pub sha256_digest: String,
    pub content_json: String,
    pub created_at: String,
    pub created_by: String,
}

/// Audit Log Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub tenant_id: TenantId,
    pub user_id: String,
    pub action: String,
    pub document_id: String,
    pub document_digest: String,
}

/// In-Memory / File-backed Isolated Multi-Tenant DataStore
#[derive(Debug, Clone, Default)]
pub struct FabricDataStore {
    // Optional base directory for persistent file-backed storage
    storage_dir: Option<PathBuf>,
    // Key: (TenantId, Namespace, DocumentId) -> StoredDocument
    documents: Arc<RwLock<DocumentMap>>,
    // Per-tenant audit trail
    audit_logs: Arc<RwLock<HashMap<TenantId, Vec<AuditEntry>>>>,
}

impl FabricDataStore {
    pub fn new() -> Self {
        Self {
            storage_dir: None,
            documents: Arc::new(RwLock::new(HashMap::new())),
            audit_logs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Construct a persistent datastore rooted at `storage_dir`, creating
    /// directories if needed and loading pre-existing documents and audit entries.
    pub fn new_persistent(storage_dir: PathBuf) -> Result<Self, DataStoreError> {
        std::fs::create_dir_all(&storage_dir)?;

        let mut doc_map = HashMap::new();
        let mut audit_map = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(&storage_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let tenant_name = match entry.file_name().into_string() {
                    Ok(name) => name,
                    Err(_) => continue,
                };

                let tenant_id = match TenantId::new(&tenant_name) {
                    Ok(id) => id,
                    Err(_) => continue,
                };

                // Load audit trail if audit.jsonl exists
                let audit_file = path.join("audit.jsonl");
                if audit_file.is_file() {
                    let file = std::fs::File::open(&audit_file)?;
                    let reader = std::io::BufReader::new(file);
                    let mut entries_vec = Vec::new();
                    for line_res in reader.lines() {
                        let line = line_res?;
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let audit_entry: AuditEntry = serde_json::from_str(trimmed)?;
                        entries_vec.push(audit_entry);
                    }
                    if !entries_vec.is_empty() {
                        audit_map.insert(tenant_id.clone(), entries_vec);
                    }
                }

                // Scan namespace directories within tenant partition
                if let Ok(ns_entries) = std::fs::read_dir(&path) {
                    for ns_entry in ns_entries {
                        let ns_entry = ns_entry?;
                        let ns_path = ns_entry.path();
                        if !ns_path.is_dir() {
                            continue;
                        }

                        let namespace = match ns_entry.file_name().into_string() {
                            Ok(name) => name,
                            Err(_) => continue,
                        };

                        if let Ok(doc_entries) = std::fs::read_dir(&ns_path) {
                            for doc_entry in doc_entries {
                                let doc_entry = doc_entry?;
                                let doc_path = doc_entry.path();
                                if !doc_path.is_file() {
                                    continue;
                                }

                                let filename = match doc_entry.file_name().into_string() {
                                    Ok(f) => f,
                                    Err(_) => continue,
                                };

                                if !filename.ends_with(".json") || filename.contains(".tmp") {
                                    continue;
                                }

                                let content = std::fs::read_to_string(&doc_path)?;
                                let doc: StoredDocument = serde_json::from_str(&content)?;

                                // Verify cryptographic integrity: digest must match content
                                let computed_digest =
                                    format!("{:x}", Sha256::digest(doc.content_json.as_bytes()));
                                if doc.sha256_digest != computed_digest {
                                    return Err(DataStoreError::Integrity(format!(
                                        "Integrity violation: document {} in tenant {} digest mismatch (recorded: {}, computed: {})",
                                        doc.document_id,
                                        tenant_id,
                                        doc.sha256_digest,
                                        computed_digest
                                    )));
                                }

                                let key = (
                                    tenant_id.clone(),
                                    namespace.clone(),
                                    doc.document_id.clone(),
                                );
                                doc_map.insert(key, doc);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            storage_dir: Some(storage_dir),
            documents: Arc::new(RwLock::new(doc_map)),
            audit_logs: Arc::new(RwLock::new(audit_map)),
        })
    }

    /// Returns the optional persistent storage directory path
    pub fn storage_dir(&self) -> Option<&Path> {
        self.storage_dir.as_deref()
    }

    fn validate_path_segment(segment: &str) -> Result<(), DataStoreError> {
        if segment.is_empty()
            || segment.contains('/')
            || segment.contains('\\')
            || segment == "."
            || segment == ".."
            || segment.contains("..")
        {
            return Err(DataStoreError::Integrity(format!(
                "Invalid path segment: '{segment}'"
            )));
        }
        Ok(())
    }

    fn persist_document_atomic(
        storage_dir: &Path,
        doc: &StoredDocument,
    ) -> Result<(), DataStoreError> {
        Self::validate_path_segment(&doc.namespace)?;
        Self::validate_path_segment(&doc.document_id)?;

        let dir = storage_dir
            .join(doc.tenant_id.as_str())
            .join(&doc.namespace);
        std::fs::create_dir_all(&dir)?;

        let target = dir.join(format!("{}.json", doc.document_id));
        let tmp = dir.join(format!(
            "{}.json.tmp.{}",
            doc.document_id,
            uuid::Uuid::new_v4()
        ));

        let content = serde_json::to_string_pretty(doc)?;
        if let Err(e) = std::fs::write(&tmp, content.as_bytes()) {
            let _ = std::fs::remove_file(&tmp);
            return Err(DataStoreError::Io(e));
        }

        if let Err(e) = std::fs::rename(&tmp, &target) {
            let _ = std::fs::remove_file(&tmp);
            return Err(DataStoreError::Io(e));
        }

        Ok(())
    }

    fn remove_document_file(
        storage_dir: &Path,
        tenant_id: &TenantId,
        namespace: &str,
        document_id: &str,
    ) -> Result<(), DataStoreError> {
        Self::validate_path_segment(namespace)?;
        Self::validate_path_segment(document_id)?;

        let target = storage_dir
            .join(tenant_id.as_str())
            .join(namespace)
            .join(format!("{}.json", document_id));

        if target.exists() {
            std::fs::remove_file(&target)?;
        }

        Ok(())
    }

    fn append_audit_log_file(
        storage_dir: &Path,
        tenant_id: &TenantId,
        entry: &AuditEntry,
    ) -> Result<(), DataStoreError> {
        let tenant_dir = storage_dir.join(tenant_id.as_str());
        std::fs::create_dir_all(&tenant_dir)?;
        let audit_path = tenant_dir.join("audit.jsonl");

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(audit_path)?;
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        file.flush()?;

        Ok(())
    }

    /// Insert or update a document strictly within the caller's tenant context
    pub fn put_document(
        &self,
        ctx: &TenantContext,
        namespace: &str,
        document_id: &str,
        resource_type: &str,
        content_json: &str,
    ) -> Result<StoredDocument, DataStoreError> {
        let digest = format!("{:x}", Sha256::digest(content_json.as_bytes()));
        let now = chrono::Utc::now().to_rfc3339();

        let key = (
            ctx.tenant_id.clone(),
            namespace.to_string(),
            document_id.to_string(),
        );

        let mut docs = self
            .documents
            .write()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;
        let version = if let Some(existing) = docs.get(&key) {
            existing.version + 1
        } else {
            1
        };

        let stored = StoredDocument {
            document_id: document_id.to_string(),
            tenant_id: ctx.tenant_id.clone(),
            namespace: namespace.to_string(),
            resource_type: resource_type.to_string(),
            version,
            sha256_digest: digest.clone(),
            content_json: content_json.to_string(),
            created_at: now.clone(),
            created_by: ctx.user_id.to_string(),
        };

        if let Some(base_dir) = &self.storage_dir {
            Self::persist_document_atomic(base_dir, &stored)?;
        }

        docs.insert(key, stored.clone());
        drop(docs);

        // Append to tenant audit trail
        let audit_entry = AuditEntry {
            timestamp: now,
            tenant_id: ctx.tenant_id.clone(),
            user_id: ctx.user_id.to_string(),
            action: if version == 1 {
                "CREATE".to_string()
            } else {
                "UPDATE".to_string()
            },
            document_id: document_id.to_string(),
            document_digest: digest,
        };

        let mut audits = self
            .audit_logs
            .write()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;

        if let Some(base_dir) = &self.storage_dir {
            Self::append_audit_log_file(base_dir, &ctx.tenant_id, &audit_entry)?;
        }

        audits
            .entry(ctx.tenant_id.clone())
            .or_default()
            .push(audit_entry);

        Ok(stored)
    }

    /// Retrieve a document strictly verified against the caller's tenant context
    pub fn get_document(
        &self,
        ctx: &TenantContext,
        namespace: &str,
        document_id: &str,
    ) -> Result<StoredDocument, DataStoreError> {
        let key = (
            ctx.tenant_id.clone(),
            namespace.to_string(),
            document_id.to_string(),
        );
        let docs = self
            .documents
            .read()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;

        docs.get(&key)
            .cloned()
            .ok_or_else(|| DataStoreError::NotFound {
                id: document_id.to_string(),
                namespace: namespace.to_string(),
            })
    }

    /// List all documents for the caller's tenant within an optional namespace
    pub fn list_documents(
        &self,
        ctx: &TenantContext,
        namespace_filter: Option<&str>,
    ) -> Result<Vec<StoredDocument>, DataStoreError> {
        let docs = self
            .documents
            .read()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;
        Ok(docs
            .iter()
            .filter(|((t_id, ns, _), _)| {
                t_id == &ctx.tenant_id && namespace_filter.is_none_or(|f| ns == f)
            })
            .map(|(_, doc)| doc.clone())
            .collect())
    }

    /// Retrieve audit logs for the caller's tenant
    pub fn get_audit_logs(&self, ctx: &TenantContext) -> Result<Vec<AuditEntry>, DataStoreError> {
        let audits = self
            .audit_logs
            .read()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;
        Ok(audits.get(&ctx.tenant_id).cloned().unwrap_or_default())
    }

    /// Delete a document strictly within the caller's tenant context
    pub fn delete_document(
        &self,
        ctx: &TenantContext,
        namespace: &str,
        document_id: &str,
    ) -> Result<StoredDocument, DataStoreError> {
        let key = (
            ctx.tenant_id.clone(),
            namespace.to_string(),
            document_id.to_string(),
        );
        let mut docs = self
            .documents
            .write()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;
        let stored = docs
            .get(&key)
            .cloned()
            .ok_or_else(|| DataStoreError::NotFound {
                id: document_id.to_string(),
                namespace: namespace.to_string(),
            })?;

        if let Some(base_dir) = &self.storage_dir {
            Self::remove_document_file(base_dir, &ctx.tenant_id, namespace, document_id)?;
        }

        docs.remove(&key);
        drop(docs);

        let audit_entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            tenant_id: ctx.tenant_id.clone(),
            user_id: ctx.user_id.to_string(),
            action: "DELETE".to_string(),
            document_id: document_id.to_string(),
            document_digest: stored.sha256_digest.clone(),
        };

        let mut audits = self
            .audit_logs
            .write()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;

        if let Some(base_dir) = &self.storage_dir {
            Self::append_audit_log_file(base_dir, &ctx.tenant_id, &audit_entry)?;
        }

        audits
            .entry(ctx.tenant_id.clone())
            .or_default()
            .push(audit_entry);

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fabric::tenant::UserId;

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "mizan-datastore-test-{}-{}",
                prefix,
                uuid::Uuid::new_v4()
            ));
            let _ = std::fs::create_dir_all(&path);
            Self { path }
        }

        fn path(&self) -> PathBuf {
            self.path.clone()
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_datastore_isolation_between_tenants() {
        let store = FabricDataStore::new();

        let t1 = TenantId::new("corp-a").unwrap();
        let ctx1 = TenantContext::new(t1, UserId::new("alice"));

        let t2 = TenantId::new("corp-b").unwrap();
        let ctx2 = TenantContext::new(t2, UserId::new("bob"));

        // Tenant 1 writes a sensitive SSP
        store
            .put_document(
                &ctx1,
                "production",
                "ssp-01",
                "ssp",
                r#"{"title":"Corp A Secret"}"#,
            )
            .expect("put doc");

        // Tenant 1 can read it
        let doc1 = store
            .get_document(&ctx1, "production", "ssp-01")
            .expect("get doc");
        assert_eq!(doc1.document_id, "ssp-01");

        // Tenant 2 CANNOT read Tenant 1's document (returns NotFound because it's partitioned)
        assert!(store.get_document(&ctx2, "production", "ssp-01").is_err());
    }

    #[test]
    fn test_datastore_versioning_and_audit() {
        let store = FabricDataStore::new();
        let t = TenantId::new("corp-a").unwrap();
        let ctx = TenantContext::new(t, UserId::new("alice"));

        let doc_v1 = store
            .put_document(&ctx, "prod", "doc-1", "catalog", "{}")
            .unwrap();
        assert_eq!(doc_v1.version, 1);

        let doc_v2 = store
            .put_document(&ctx, "prod", "doc-1", "catalog", "{\"modified\":true}")
            .unwrap();
        assert_eq!(doc_v2.version, 2);

        let audits = store.get_audit_logs(&ctx).unwrap();
        assert_eq!(audits.len(), 2);
        assert_eq!(audits[0].action, "CREATE");
        assert_eq!(audits[1].action, "UPDATE");
    }

    #[test]
    fn test_persistent_datastore_lifecycle_and_recovery() {
        let guard = TempDirGuard::new("lifecycle");
        let storage_path = guard.path();

        let t1 = TenantId::new("tenant-one").unwrap();
        let ctx1 = TenantContext::new(t1.clone(), UserId::new("alice"));

        let t2 = TenantId::new("tenant-two").unwrap();
        let ctx2 = TenantContext::new(t2.clone(), UserId::new("bob"));

        // Phase 1: Initialize persistent datastore and write documents
        {
            let store = FabricDataStore::new_persistent(storage_path.clone())
                .expect("initialize persistent datastore");
            assert_eq!(store.storage_dir(), Some(storage_path.as_path()));

            // Tenant 1: put doc-1 (v1), doc-2 (v1), then update doc-1 (v2)
            let d1_v1 = store
                .put_document(&ctx1, "ns-a", "doc-1", "ssp", r#"{"val":1}"#)
                .unwrap();
            assert_eq!(d1_v1.version, 1);

            let d2_v1 = store
                .put_document(&ctx1, "ns-b", "doc-2", "catalog", r#"{"val":2}"#)
                .unwrap();
            assert_eq!(d2_v1.version, 1);

            let d1_v2 = store
                .put_document(&ctx1, "ns-a", "doc-1", "ssp", r#"{"val":1,"updated":true}"#)
                .unwrap();
            assert_eq!(d1_v2.version, 2);

            // Tenant 2: put doc-alpha (v1)
            let da_v1 = store
                .put_document(&ctx2, "ns-a", "doc-alpha", "profile", r#"{"val":"alpha"}"#)
                .unwrap();
            assert_eq!(da_v1.version, 1);

            // Verify files exist on disk
            let doc1_file = storage_path.join("tenant-one/ns-a/doc-1.json");
            let doc2_file = storage_path.join("tenant-one/ns-b/doc-2.json");
            let doca_file = storage_path.join("tenant-two/ns-a/doc-alpha.json");
            let audit1_file = storage_path.join("tenant-one/audit.jsonl");
            let audit2_file = storage_path.join("tenant-two/audit.jsonl");

            assert!(doc1_file.is_file(), "doc-1 file must exist on disk");
            assert!(doc2_file.is_file(), "doc-2 file must exist on disk");
            assert!(doca_file.is_file(), "doc-alpha file must exist on disk");
            assert!(audit1_file.is_file(), "audit log 1 must exist on disk");
            assert!(audit2_file.is_file(), "audit log 2 must exist on disk");
        } // store drops here, simulating process restart

        // Phase 2: Restart from disk and verify crash recovery
        {
            let store = FabricDataStore::new_persistent(storage_path.clone())
                .expect("reload persistent datastore from disk");

            // Verify Tenant 1 documents reloaded
            let d1 = store.get_document(&ctx1, "ns-a", "doc-1").unwrap();
            assert_eq!(d1.version, 2);
            assert_eq!(d1.content_json, r#"{"val":1,"updated":true}"#);

            let d2 = store.get_document(&ctx1, "ns-b", "doc-2").unwrap();
            assert_eq!(d2.version, 1);
            assert_eq!(d2.content_json, r#"{"val":2}"#);

            // Verify Tenant 2 documents reloaded
            let da = store.get_document(&ctx2, "ns-a", "doc-alpha").unwrap();
            assert_eq!(da.version, 1);
            assert_eq!(da.content_json, r#"{"val":"alpha"}"#);

            // Verify audit logs reloaded
            let audits1 = store.get_audit_logs(&ctx1).unwrap();
            assert_eq!(audits1.len(), 3);
            assert_eq!(audits1[0].action, "CREATE");
            assert_eq!(audits1[0].document_id, "doc-1");
            assert_eq!(audits1[1].action, "CREATE");
            assert_eq!(audits1[1].document_id, "doc-2");
            assert_eq!(audits1[2].action, "UPDATE");
            assert_eq!(audits1[2].document_id, "doc-1");

            let audits2 = store.get_audit_logs(&ctx2).unwrap();
            assert_eq!(audits2.len(), 1);
            assert_eq!(audits2[0].action, "CREATE");
            assert_eq!(audits2[0].document_id, "doc-alpha");

            // Phase 3: Update doc-1 again, version must advance to 3
            let d1_v3 = store
                .put_document(&ctx1, "ns-a", "doc-1", "ssp", r#"{"val":1,"v":3}"#)
                .unwrap();
            assert_eq!(d1_v3.version, 3);
        }
    }

    #[test]
    fn test_persistent_datastore_deletion_and_recovery() {
        let guard = TempDirGuard::new("deletion");
        let storage_path = guard.path();

        let t = TenantId::new("tenant-del").unwrap();
        let ctx = TenantContext::new(t, UserId::new("operator"));

        let doc_file = storage_path.join("tenant-del/sec/policy-01.json");

        {
            let store = FabricDataStore::new_persistent(storage_path.clone()).unwrap();
            store
                .put_document(&ctx, "sec", "policy-01", "policy", r#"{"allow":true}"#)
                .unwrap();
            assert!(doc_file.is_file());

            let deleted = store.delete_document(&ctx, "sec", "policy-01").unwrap();
            assert_eq!(deleted.document_id, "policy-01");
            assert!(!doc_file.exists(), "file must be removed after deletion");

            let audits = store.get_audit_logs(&ctx).unwrap();
            assert_eq!(audits.len(), 2);
            assert_eq!(audits[1].action, "DELETE");
        }

        // Reopen from disk: deleted document must NOT exist
        {
            let store = FabricDataStore::new_persistent(storage_path).unwrap();
            assert!(store.get_document(&ctx, "sec", "policy-01").is_err());

            let audits = store.get_audit_logs(&ctx).unwrap();
            assert_eq!(audits.len(), 2);
            assert_eq!(audits[0].action, "CREATE");
            assert_eq!(audits[1].action, "DELETE");
        }
    }

    #[test]
    fn test_persistent_datastore_tenant_directory_isolation() {
        let guard = TempDirGuard::new("isolation");
        let storage_path = guard.path();

        let t_a = TenantId::new("tenant-alpha").unwrap();
        let ctx_a = TenantContext::new(t_a, UserId::new("alice"));

        let t_b = TenantId::new("tenant-beta").unwrap();
        let ctx_b = TenantContext::new(t_b, UserId::new("bob"));

        let store = FabricDataStore::new_persistent(storage_path.clone()).unwrap();

        store
            .put_document(&ctx_a, "confidential", "doc-a", "ssp", r#"{"secret":"a"}"#)
            .unwrap();
        store
            .put_document(&ctx_b, "confidential", "doc-b", "ssp", r#"{"secret":"b"}"#)
            .unwrap();

        // Inspect directory structure
        let dir_a = storage_path.join("tenant-alpha");
        let dir_b = storage_path.join("tenant-beta");

        assert!(dir_a.is_dir());
        assert!(dir_b.is_dir());

        assert!(dir_a.join("confidential/doc-a.json").is_file());
        assert!(!dir_a.join("confidential/doc-b.json").exists());

        assert!(dir_b.join("confidential/doc-b.json").is_file());
        assert!(!dir_b.join("confidential/doc-a.json").exists());

        // Cross-tenant in-memory access fails
        assert!(store.get_document(&ctx_a, "confidential", "doc-b").is_err());
        assert!(store.get_document(&ctx_b, "confidential", "doc-a").is_err());
    }

    #[test]
    fn test_persistent_datastore_integrity_tamper_detection() {
        let guard = TempDirGuard::new("tamper");
        let storage_path = guard.path();

        let t = TenantId::new("tenant-tamper").unwrap();
        let ctx = TenantContext::new(t, UserId::new("admin"));

        {
            let store = FabricDataStore::new_persistent(storage_path.clone()).unwrap();
            store
                .put_document(&ctx, "baseline", "doc-1", "catalog", r#"{"original":true}"#)
                .unwrap();
        }

        // Tamper directly with the file on disk (corrupt content_json without valid sha256_digest)
        let doc_file = storage_path.join("tenant-tamper/baseline/doc-1.json");
        assert!(doc_file.is_file());

        let mut tampered_doc: StoredDocument =
            serde_json::from_str(&std::fs::read_to_string(&doc_file).unwrap()).unwrap();
        tampered_doc.content_json = r#"{"original":false,"backdoor":true}"#.to_string();
        std::fs::write(&doc_file, serde_json::to_string(&tampered_doc).unwrap()).unwrap();

        // Reloading should fail with DataStoreError::Integrity
        let load_res = FabricDataStore::new_persistent(storage_path);
        match load_res {
            Err(DataStoreError::Integrity(msg)) => {
                assert!(msg.contains("Integrity violation"));
            }
            Ok(_) => panic!("Expected tamper detection to return Integrity error, but succeeded"),
            Err(e) => panic!("Expected Integrity error, got: {e:?}"),
        }
    }
}
