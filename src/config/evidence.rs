#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct EvidenceArgs {
    #[command(subcommand)]
    pub action: EvidenceAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum EvidenceAction {
    Get {
        /// Content-addressed evidence identifier.
        evidence_id: String,
    },
    Verify {
        /// Content-addressed evidence identifier.
        evidence_id: String,
        /// Fetch the referenced subject and verify its digest when supported.
        #[arg(long)]
        fetch_and_hash: bool,
    },
    /// Read the append-only audit history for external evidence fetches.
    FetchEvents {
        /// Maximum number of fetch events to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Upload {
        /// Path to the evidence JSON document.
        file: PathBuf,
        /// Optional binary blob to associate with the evidence.
        #[arg(long)]
        blob: Option<PathBuf>,
    },
    /// Generate a cryptographic proof Evidence Bundle from an OSCAL document.
    Bundle {
        /// Path to OSCAL document (catalog, ssp, assessment-results).
        file: PathBuf,
        /// Target evidence assurance level (e0, e1, e2, e3, e4, e5).
        #[arg(long, default_value = "e3")]
        level: String,
        /// Evaluator engine identification string.
        #[arg(long, default_value = "mizan-kernel-v1")]
        evaluator: String,
        /// Optional identity to sign the bundle with.
        #[arg(long)]
        sign_as: Option<String>,
        /// Output file path for generated evidence bundle JSON.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Verify the Merkle root and cryptographic signatures of an Evidence Bundle.
    VerifyBundle {
        /// Path to evidence bundle JSON file.
        bundle_file: PathBuf,
    },
}
