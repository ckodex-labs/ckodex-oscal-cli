#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ClaimArgs {
    #[command(subcommand)]
    pub action: ClaimAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ClaimAction {
    List {
        /// Filter by subject digest.
        #[arg(long)]
        subject_digest: Option<String>,
        /// Filter by BOM kind.
        #[arg(long)]
        bom_kind: Option<String>,
        /// Filter by graph relation.
        #[arg(long)]
        relation: Option<String>,
        /// Filter by observed trust state.
        #[arg(long)]
        trust_state: Option<String>,
        /// Maximum number of claims to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// Claim identifier.
        claim_id: String,
    },
    Events {
        /// Claim identifier whose append-only events are read.
        claim_id: String,
    },
    Receipt {
        /// Claim identifier whose deterministic receipt is read.
        claim_id: String,
    },
    Create {
        /// Path to a JSON or YAML claim document.
        file: PathBuf,
    },
    Verify {
        /// Claim identifier to verify.
        claim_id: String,
        /// Named checks to run; empty runs all.
        #[arg(long = "check", value_delimiter = ',')]
        checks: Vec<String>,
    },
    Sync {
        /// Peer endpoint to sync claims from.
        #[arg(long, default_value = "")]
        peer_endpoint: String,
        /// Filter by subject digest.
        #[arg(long, default_value = "")]
        filter_subject: String,
        /// Filter by BOM kind.
        #[arg(long, default_value = "")]
        filter_bom_kind: String,
    },
}
