#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct DedupArgs {
    /// Path to input OSCAL document.
    pub file: PathBuf,
    /// Output file path for deduplicated document (defaults to stdout in dry-run mode).
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
    /// Perform a dry-run report without saving modifications.
    #[arg(long)]
    pub dry_run: bool,
}
