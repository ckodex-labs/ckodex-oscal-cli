#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct FsmCliArgs {
    #[command(subcommand)]
    pub action: FsmCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum FsmCliAction {
    /// Inspect compliance state and required evidence level.
    Status {
        /// Current state (draft, under_review, approved, operational, degraded, remediating, archived).
        #[arg(long, default_value = "draft")]
        state: String,
        /// Current evidence level (e0, e1, e2, e3, e4, e5).
        #[arg(long, default_value = "e0")]
        evidence: String,
    },
    /// Evaluate a verified state transition against evidence gates.
    Transition {
        /// Origin state.
        #[arg(long, default_value = "draft")]
        from: String,
        /// Current evidence level.
        #[arg(long, default_value = "e1")]
        evidence: String,
        /// Trigger event (submit_for_review, approve, deploy_to_cluster, resolve_remediation, archive).
        #[arg(long)]
        event: String,
        /// Actor initiating transition.
        #[arg(long, default_value = "security-engineer")]
        actor: String,
        /// Audit rationale for transition.
        #[arg(long, default_value = "Standard promotion gate")]
        rationale: String,
    },
}
