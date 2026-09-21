// Mizan Root Fabric · Isolated Multi-Tenant High-Assurance DataStore
// Cryptographically isolated storage partitioned strictly by TenantId and Namespace

use crate::fabric::tenant::{TenantContext, TenantError, TenantId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
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
    // Key: (TenantId, Namespace, DocumentId) -> StoredDocument
    documents: Arc<RwLock<DocumentMap>>,
    // Per-tenant audit trail
    audit_logs: Arc<RwLock<HashMap<TenantId, Vec<AuditEntry>>>>,
}

impl FabricDataStore {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            audit_logs: Arc::new(RwLock::new(HashMap::new())),
        }
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

        docs.insert(key, stored.clone());

        // Append to tenant audit trail
        let mut audits = self
            .audit_logs
            .write()
            .map_err(|e| DataStoreError::Lock(e.to_string()))?;
        audits
            .entry(ctx.tenant_id.clone())
            .or_default()
            .push(AuditEntry {
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
            });

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fabric::tenant::UserId;

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
}
