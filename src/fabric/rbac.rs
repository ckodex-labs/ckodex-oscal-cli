// Mizan Root Fabric · Role-Based & Attribute-Based Access Control (RBAC/ABAC)
// Formally verified permission matrix for high-assurance governance

use crate::fabric::tenant::TenantContext;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum RbacError {
    #[error(
        "Access denied: user {user} in tenant {tenant} lacks permission for action {action:?} on resource {resource:?}"
    )]
    AccessDenied {
        user: String,
        tenant: String,
        action: Action,
        resource: ResourceType,
    },
    #[error("Unknown role name: {0}")]
    UnknownRole(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    TenantAdmin,
    ComplianceArchitect,
    Auditor,
    SecOpsOperator,
    Viewer,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TenantAdmin => "TenantAdmin",
            Self::ComplianceArchitect => "ComplianceArchitect",
            Self::Auditor => "Auditor",
            Self::SecOpsOperator => "SecOpsOperator",
            Self::Viewer => "Viewer",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Role {
    type Err = RbacError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "tenantadmin" | "admin" => Ok(Self::TenantAdmin),
            "compliancearchitect" | "architect" => Ok(Self::ComplianceArchitect),
            "auditor" | "assessor" => Ok(Self::Auditor),
            "secopsoperator" | "operator" | "secops" => Ok(Self::SecOpsOperator),
            "viewer" | "reader" => Ok(Self::Viewer),
            _ => Err(RbacError::UnknownRole(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    ReadDocument,
    CreateDocument,
    UpdateDocument,
    DeleteDocument,
    PublishBaseline,
    SignAttestation,
    PromoteEnvironment,
    ManageTenants,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Catalog,
    Profile,
    Ssp,
    AssessmentResults,
    Poam,
    Attestation,
    TenantConfig,
}

/// RBAC Evaluation Engine
#[derive(Debug, Clone, Default)]
pub struct RbacEngine;

impl RbacEngine {
    pub fn new() -> Self {
        Self
    }

    /// Check if context has permission to execute action on resource
    pub fn authorize(
        &self,
        ctx: &TenantContext,
        action: Action,
        resource: ResourceType,
    ) -> Result<(), RbacError> {
        let mut roles: Vec<Role> = Vec::with_capacity(ctx.roles.len());
        for role in &ctx.roles {
            roles.push(role.parse::<Role>()?);
        }

        for role in &roles {
            if Self::role_allows(*role, action, resource) {
                return Ok(());
            }
        }

        Err(RbacError::AccessDenied {
            user: ctx.user_id.to_string(),
            tenant: ctx.tenant_id.to_string(),
            action,
            resource,
        })
    }

    fn role_allows(role: Role, action: Action, resource: ResourceType) -> bool {
        match role {
            Role::TenantAdmin => true, // Superuser within tenant boundary
            Role::ComplianceArchitect => match action {
                Action::ReadDocument
                | Action::CreateDocument
                | Action::UpdateDocument
                | Action::PublishBaseline
                | Action::SignAttestation
                | Action::PromoteEnvironment => true,
                Action::DeleteDocument => resource != ResourceType::TenantConfig,
                Action::ManageTenants => false,
            },
            Role::Auditor => match action {
                Action::ReadDocument => true,
                Action::CreateDocument => {
                    resource == ResourceType::AssessmentResults || resource == ResourceType::Poam
                }
                Action::SignAttestation => true,
                Action::UpdateDocument
                | Action::DeleteDocument
                | Action::PublishBaseline
                | Action::PromoteEnvironment
                | Action::ManageTenants => false,
            },
            Role::SecOpsOperator => match action {
                Action::ReadDocument => true,
                Action::CreateDocument | Action::UpdateDocument => {
                    resource == ResourceType::AssessmentResults || resource == ResourceType::Poam
                }
                Action::SignAttestation => true,
                Action::DeleteDocument
                | Action::PublishBaseline
                | Action::PromoteEnvironment
                | Action::ManageTenants => false,
            },
            Role::Viewer => action == Action::ReadDocument,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fabric::tenant::{TenantId, UserId};

    #[test]
    fn test_rbac_architect_permissions() {
        let t = TenantId::new("meridian").unwrap();
        let u = UserId::new("architect@meridian.io");
        let ctx = TenantContext::new(t, u).with_roles(vec!["ComplianceArchitect".to_string()]);

        let engine = RbacEngine::new();

        // Architect can create SSP and publish baseline
        assert!(
            engine
                .authorize(&ctx, Action::CreateDocument, ResourceType::Ssp)
                .is_ok()
        );
        assert!(
            engine
                .authorize(&ctx, Action::PublishBaseline, ResourceType::Profile)
                .is_ok()
        );

        // Architect cannot manage global tenant settings
        assert!(
            engine
                .authorize(&ctx, Action::ManageTenants, ResourceType::TenantConfig)
                .is_err()
        );
    }

    #[test]
    fn test_rbac_auditor_permissions() {
        let t = TenantId::new("meridian").unwrap();
        let u = UserId::new("auditor@meridian.io");
        let ctx = TenantContext::new(t, u).with_roles(vec!["Auditor".to_string()]);

        let engine = RbacEngine::new();

        // Auditor can read documents and emit assessment results
        assert!(
            engine
                .authorize(&ctx, Action::ReadDocument, ResourceType::Catalog)
                .is_ok()
        );
        assert!(
            engine
                .authorize(
                    &ctx,
                    Action::CreateDocument,
                    ResourceType::AssessmentResults
                )
                .is_ok()
        );

        // Auditor CANNOT delete catalogs or promote environments
        assert!(
            engine
                .authorize(&ctx, Action::DeleteDocument, ResourceType::Catalog)
                .is_err()
        );
        assert!(
            engine
                .authorize(&ctx, Action::PromoteEnvironment, ResourceType::Profile)
                .is_err()
        );
    }
}
