// Mizan Root Fabric · Unified Enterprise Identity, Multi-Tenancy & DataStore Engine

pub mod datastore;
pub mod oidc;
pub mod rbac;
pub mod spiffe;
pub mod tenant;

pub use datastore::{AuditEntry, DataStoreError, FabricDataStore, StoredDocument};
pub use oidc::{OidcClaims, OidcError, OidcProviderConfig, OidcTokenValidator};
pub use rbac::{Action, RbacEngine, RbacError, ResourceType, Role};
pub use spiffe::{JwtSvid, SpiffeError, SpiffeId, SpireWorkloadAttestor, TrustDomain, X509Svid};
pub use tenant::{TenantContext, TenantError, TenantId, TenantManager, TenantMetadata, UserId};

/// Comprehensive Root Fabric Engine coordinating identity, governance, and partitioned storage
#[derive(Debug, Clone)]
pub struct RootFabricEngine {
    pub tenant_manager: TenantManager,
    pub rbac: RbacEngine,
    pub datastore: FabricDataStore,
    pub oidc_validator: Option<OidcTokenValidator>,
    pub spire_attestor: Option<SpireWorkloadAttestor>,
}

impl Default for RootFabricEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RootFabricEngine {
    pub fn new() -> Self {
        Self {
            tenant_manager: TenantManager::new(),
            rbac: RbacEngine::new(),
            datastore: FabricDataStore::new(),
            oidc_validator: None,
            spire_attestor: None,
        }
    }

    pub fn with_oidc(mut self, config: OidcProviderConfig) -> Self {
        self.oidc_validator = Some(OidcTokenValidator::new(config));
        self
    }

    pub fn with_spiffe(mut self, trust_domain: TrustDomain) -> Self {
        self.spire_attestor = Some(SpireWorkloadAttestor::new(trust_domain));
        self
    }

    /// Authenticate an incoming OIDC JWT and produce an authorized TenantContext
    pub fn authenticate_oidc(
        &self,
        token: &str,
        now_timestamp: i64,
    ) -> Result<TenantContext, OidcError> {
        let validator = self.oidc_validator.as_ref().ok_or_else(|| {
            OidcError::ClaimsDeserialization(
                "OIDC provider not configured in Root Fabric".to_string(),
            )
        })?;

        let claims = validator.decode_and_validate(token, now_timestamp)?;
        let tenant_id_str = claims.tenant_id.as_deref().unwrap_or("default");
        let tenant_id = TenantId::new(tenant_id_str)
            .map_err(|e| OidcError::ClaimsDeserialization(e.to_string()))?;

        let user_id = UserId::new(claims.sub);
        let ctx = TenantContext::new(tenant_id, user_id).with_roles(claims.roles);
        Ok(ctx)
    }

    /// Authenticate an incoming SPIFFE Workload SVID
    pub fn authenticate_spiffe(
        &self,
        svid: &JwtSvid,
        now_timestamp: i64,
    ) -> Result<TenantContext, SpiffeError> {
        let attestor =
            self.spire_attestor
                .as_ref()
                .ok_or_else(|| SpiffeError::TrustDomainMismatch {
                    expected: "Configured TrustDomain".to_string(),
                    actual: "Unconfigured".to_string(),
                })?;

        attestor.verify_jwt_svid(svid, now_timestamp)?;

        let ns = svid.spiffe_id.namespace().unwrap_or("default");
        let sa = svid.spiffe_id.service_account().unwrap_or("workload");

        let default_tenant = match TenantId::new("default") {
            Ok(id) => id,
            Err(_) => TenantId::new_unchecked("default"),
        };
        let tenant_id = match TenantId::new(ns) {
            Ok(id) => id,
            Err(_) => default_tenant,
        };
        let user_id = UserId::new(format!("sa:{}", sa));

        let ctx = TenantContext::new(tenant_id, user_id)
            .with_spiffe_id(svid.spiffe_id.clone())
            .with_roles(vec!["SecOpsOperator".to_string()]);

        Ok(ctx)
    }

    /// Guarded document creation with RBAC enforcement and isolated datastore write
    pub fn store_document_guarded(
        &self,
        ctx: &TenantContext,
        namespace: &str,
        document_id: &str,
        resource_type: ResourceType,
        content_json: &str,
    ) -> Result<StoredDocument, String> {
        self.rbac
            .authorize(ctx, Action::CreateDocument, resource_type)
            .map_err(|e| e.to_string())?;

        let res_type_str = format!("{:?}", resource_type);
        self.datastore
            .put_document(ctx, namespace, document_id, &res_type_str, content_json)
            .map_err(|e| e.to_string())
    }
}
