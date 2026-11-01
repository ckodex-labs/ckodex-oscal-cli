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
    #[arg(long, alias = "justification")]
    pub reason: Option<String>,
    /// Time-to-live for the derogation lease (e.g. 30m, 24h, 7d, 30d).
    #[arg(long, default_value = "7d", alias = "duration-days")]
    pub ttl: String,
    /// Target scope or resource (file path, workload name, or '*' for global).
    #[arg(long, default_value = "*", alias = "resource")]
    pub scope: String,
    /// Author or engineer approving the waiver.
    #[arg(long)]
    pub author: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum WaiveCliAction {
    /// Create a time-bounded cryptographic derogation waiver lease.
    Create {
        /// Rule ID to waive (e.g. cis-k8s-5.2.1).
        #[arg(long, short = 'r')]
        rule: String,
        /// Mandatory justification reason for the derogation lease.
        #[arg(long, short = 'j', alias = "justification")]
        reason: Option<String>,
        /// Time-to-live for the derogation lease (e.g. 30m, 24h, 7d, 30d).
        #[arg(long, default_value = "7d", alias = "duration-days")]
        ttl: String,
        /// Target scope or resource (file path, workload name, or '*' for global).
        #[arg(long, default_value = "*", alias = "resource")]
        scope: String,
        /// Author or engineer approving the waiver.
        #[arg(long)]
        author: Option<String>,
    },
    /// List all active, expired, and revoked derogation waivers.
    List,
    /// Revoke an active derogation waiver lease by ID.
    Revoke {
        /// Waiver ID to revoke (e.g. waiver-a1b2c3d4).
        #[arg(long, short = 'i')]
        id: String,
    },
}
