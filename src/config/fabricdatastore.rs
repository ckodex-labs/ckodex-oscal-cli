#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum FabricDatastoreAction {
    /// Show aggregate DataStore partition statistics.
    Stats,
    /// List stored documents for a tenant.
    List {
        /// Tenant ID to scope queries within.
        #[arg(long, default_value = "default")]
        tenant: String,
        /// Optional namespace filter.
        #[arg(long)]
        namespace: Option<String>,
    },
}
