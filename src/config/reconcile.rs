#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ReconcileArgs {
    /// Path to declared System Security Plan (SSP) document.
    #[arg(long)]
    pub ssp: Option<PathBuf>,
    /// Path to observed machine inventory or SBOM (JSON format).
    #[arg(long)]
    pub inventory: Option<PathBuf>,
    /// Path to Assessment Results document.
    #[arg(long)]
    pub results: Option<PathBuf>,
    /// Path to Plan of Action & Milestones (POA&M) document.
    #[arg(long)]
    pub poam: Option<PathBuf>,
    /// Fail with non-zero exit code if any drift or unmitigated findings are detected.
    #[arg(long)]
    pub strict: bool,
}
