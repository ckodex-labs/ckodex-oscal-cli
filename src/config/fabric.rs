#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum FabricAction {
    /// Display overall Root Fabric health and configuration status.
    Status,
    /// Multi-tenant management and isolation commands.
    Tenant {
        #[command(subcommand)]
        action: FabricTenantAction,
    },
    /// SPIFFE / SPIRE workload identity validation.
    Spiffe {
        /// SPIFFE ID to validate (e.g. spiffe://meridian.runbase.io/ns/prod/sa/auditor).
        id: String,
    },
    /// OpenID Connect (OIDC) JWT token claims verification.
    Oidc {
        /// Raw JWT bearer token.
        token: String,
        /// Expected token issuer.
        #[arg(long, default_value = "https://auth.runbase.io")]
        issuer: String,
        /// Expected token audience.
        #[arg(long, default_value = "mizan-workbench")]
        audience: String,
    },
    /// Query or inspect the isolated high-assurance multi-tenant DataStore.
    Datastore {
        #[command(subcommand)]
        action: FabricDatastoreAction,
    },
}
