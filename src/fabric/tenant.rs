// Mizan Root Fabric · Multi-Tenant / Multi-User Isolation Engine
// Enforces strict mathematical tenant boundaries, quotas, and contextual separation

use crate::fabric::spiffe::SpiffeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Invalid tenant ID format: {0}")]
    InvalidTenantId(String),
    #[error("Tenant not found: {0}")]
    TenantNotFound(String),
    #[error("Tenant {tenant_id} exceeded storage quota ({used_bytes} / {max_bytes} bytes)")]
    QuotaExceeded {
        tenant_id: String,
        used_bytes: u64,
        max_bytes: u64,
    },
    #[error(
        "Cross-tenant access violation: caller from {caller_tenant} attempted access to resource owned by {target_tenant}"
    )]
    CrossTenantViolation {
        caller_tenant: String,
        target_tenant: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(id: impl Into<String>) -> Result<Self, TenantError> {
        let s = id.into().trim().to_ascii_lowercase();
        if s.is_empty() || s.contains('/') || s.contains('\\') || s.contains(' ') {
            return Err(TenantError::InvalidTenantId(s));
        }
        Ok(Self(s))
    }

    /// Build a tenant id without validation. Reserved for statically known values.
    pub(crate) fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TenantId {
    type Err = TenantError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Active Tenant Context attached to all operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub spiffe_id: Option<SpiffeId>,
    pub roles: Vec<String>,
    pub default_jurisdiction: String,
}

impl TenantContext {
    pub fn new(tenant_id: TenantId, user_id: UserId) -> Self {
        Self {
            tenant_id,
            user_id,
            spiffe_id: None,
            roles: vec!["Viewer".to_string()],
            default_jurisdiction: "us".to_string(),
        }
    }

    pub fn with_spiffe_id(mut self, spiffe_id: SpiffeId) -> Self {
        self.spiffe_id = Some(spiffe_id);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_jurisdiction(mut self, jurisdiction: impl Into<String>) -> Self {
        self.default_jurisdiction = jurisdiction.into();
        self
    }

    /// Assert that a target resource belongs to this tenant
    pub fn assert_same_tenant(&self, target_tenant: &TenantId) -> Result<(), TenantError> {
        if &self.tenant_id != target_tenant {
            return Err(TenantError::CrossTenantViolation {
                caller_tenant: self.tenant_id.to_string(),
                target_tenant: target_tenant.to_string(),
            });
        }
        Ok(())
    }
}

impl Default for TenantContext {
    fn default() -> Self {
        Self::new(TenantId::new_unchecked("default"), UserId::new("system"))
    }
}

/// Tenant Profile and Quota Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMetadata {
    pub tenant_id: TenantId,
    pub display_name: String,
    pub tier: String,
    pub max_storage_bytes: u64,
    pub current_storage_bytes: u64,
    pub default_jurisdiction: String,
    pub created_at: String,
}

/// Thread-safe in-memory / persistent Tenant Registry
#[derive(Debug, Clone, Default)]
pub struct TenantManager {
    tenants: HashMap<TenantId, TenantMetadata>,
}

impl TenantManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            tenants: HashMap::new(),
        };

        // Bootstrap default system tenants
        let default_tenant = TenantMetadata {
            tenant_id: match TenantId::new("default") {
                Ok(id) => id,
                Err(_) => TenantId::new_unchecked("default"),
            },
            display_name: "Default Local Workspace".to_string(),
            tier: "Community".to_string(),
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            current_storage_bytes: 0,
            default_jurisdiction: "us".to_string(),
            created_at: "2026-08-29T00:00:00Z".to_string(),
        };

        let enterprise_tenant = TenantMetadata {
            tenant_id: match TenantId::new("meridian-corp") {
                Ok(id) => id,
                Err(_) => TenantId::new_unchecked("meridian-corp"),
            },
            display_name: "Meridian Enterprise Cloud".to_string(),
            tier: "Enterprise".to_string(),
            max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            current_storage_bytes: 0,
            default_jurisdiction: "us".to_string(),
            created_at: "2026-08-29T00:00:00Z".to_string(),
        };

        mgr.tenants
            .insert(default_tenant.tenant_id.clone(), default_tenant);
        mgr.tenants
            .insert(enterprise_tenant.tenant_id.clone(), enterprise_tenant);
        mgr
    }

    fn persistence_path() -> std::path::PathBuf {
        let dir = std::path::PathBuf::from(".mizan");
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        dir.join("tenants.json")
    }

    pub fn load_or_default() -> Self {
        let path = Self::persistence_path();
        if path.exists()
            && let Ok(data) = std::fs::read_to_string(&path)
            && let Ok(tenants_map) =
                serde_json::from_str::<HashMap<TenantId, TenantMetadata>>(&data)
            && !tenants_map.is_empty()
        {
            return Self {
                tenants: tenants_map,
            };
        }

        let mgr = Self::new();
        let _ = mgr.save();
        mgr
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::persistence_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&self.tenants)?;
        std::fs::write(path, json)
    }

    pub fn register_tenant(&mut self, metadata: TenantMetadata) {
        self.tenants.insert(metadata.tenant_id.clone(), metadata);
        let _ = self.save();
    }

    pub fn get_tenant(&self, tenant_id: &TenantId) -> Result<&TenantMetadata, TenantError> {
        self.tenants
            .get(tenant_id)
            .ok_or_else(|| TenantError::TenantNotFound(tenant_id.to_string()))
    }

    pub fn list_tenants(&self) -> Vec<&TenantMetadata> {
        self.tenants.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_context_and_isolation() {
        let t1 = TenantId::new("tenant-alpha").unwrap();
        let t2 = TenantId::new("tenant-beta").unwrap();
        let u1 = UserId::new("alice@alpha.io");

        let ctx = TenantContext::new(t1.clone(), u1);

        assert!(ctx.assert_same_tenant(&t1).is_ok());
        assert!(ctx.assert_same_tenant(&t2).is_err());
    }

    #[test]
    fn test_tenant_manager_listing() {
        let mgr = TenantManager::new();
        let list = mgr.list_tenants();
        assert!(list.len() >= 2);
    }
}
