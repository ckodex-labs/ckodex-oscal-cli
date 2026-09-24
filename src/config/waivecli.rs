#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct WaiveCliArgs {
    #[command(subcommand)]
    pub action: Option<WaiveCliAction>,
    /// Rule ID to waive (e.g. cis-k8s-5.2.1) when invoked directly.
    #[arg(long, short = 'r')]
    pub rule: Option<String>,
    /// Mandatory justification reason for the derogation lease.
    #[arg(long)]
    pub reason: Option<String>,
    /// Time-to-live for the derogation lease (e.g. 30m, 24h, 7d, 30d).
    #[arg(long, default_value = "7d")]
    pub ttl: String,
    /// Target scope (file path, workload name, or '*' for global).
    #[arg(long, default_value = "*")]
    pub scope: String,
    /// Author or engineer approving the waiver.
    #[arg(long)]
    pub author: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum WaiveCliAction {
    /// List all active, expired, and revoked derogation waivers.
    List,
    /// Revoke an active derogation waiver lease by ID.
    Revoke {
        /// Waiver ID to revoke (e.g. waiver-a1b2c3d4).
        #[arg(long, short = 'i')]
        id: String,
    },
}
