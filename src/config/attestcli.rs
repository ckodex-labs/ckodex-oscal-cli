#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct AttestCliArgs {
    #[command(subcommand)]
    pub action: AttestCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum AttestCliAction {
    /// Generate a SLSA v1.2 / v1.0 supply chain provenance attestation statement.
    Slsa {
        /// Name of the artifact subject (e.g. container image, binary, package).
        #[arg(long, short = 's')]
        subject: String,
        /// SHA-256 digest of the artifact.
        #[arg(long)]
        digest: String,
        /// SLSA version (v1.2 or v1.0).
        #[arg(long, default_value = "v1.2")]
        version: String,
        /// Optional path to OSCAL evidence bundle JSON to embed in provenance.
        #[arg(long, short = 'e')]
        evidence: Option<PathBuf>,
        /// Output file path for generated in-toto statement JSON.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Verify a SLSA supply chain provenance attestation statement.
    Verify {
        /// Path to SLSA in-toto statement JSON file.
        statement_file: PathBuf,
    },
}
