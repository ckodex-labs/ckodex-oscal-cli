#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum FabricTenantAction {
    /// List all registered tenants in the Root Fabric.
    List,
    /// Create or register a new tenant.
    Create {
        /// Unique alphanumeric tenant identifier (e.g. corp-alpha).
        id: String,
        /// Human-readable display name.
        #[arg(long)]
        name: String,
        /// Service tier (Community, Pro, Enterprise).
        #[arg(long, default_value = "Enterprise")]
        tier: String,
        /// Maximum storage quota in gigabytes.
        #[arg(long, default_value_t = 50)]
        quota_gb: u64,
        /// Default compliance jurisdiction baseline (us, ca, eu).
        #[arg(long, default_value = "us")]
        jurisdiction: String,
    },
    /// Inspect details and quotas for a specific tenant.
    Inspect {
        /// Tenant ID.
        id: String,
    },
}
