#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct SyncArgs {
    /// Path to base common ancestor OSCAL document.
    #[arg(long)]
    pub base: Option<PathBuf>,
    /// Path to updated upstream OSCAL document.
    #[arg(long)]
    pub upstream: Option<PathBuf>,
    /// Path to local modified OSCAL document.
    #[arg(long)]
    pub local: Option<PathBuf>,
    /// Synchronize an auditor-edited CSV matrix back into the local OSCAL document.
    #[arg(long)]
    pub matrix: Option<PathBuf>,
    /// 3-way merge conflict resolution strategy (manual, ours, theirs).
    #[arg(long, default_value = "manual")]
    pub strategy: String,
    /// Path for merged output OSCAL document.
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
    /// Directly scaffold merged result into a split Markdown workspace directory.
    #[arg(long = "split-to")]
    pub split_to: Option<PathBuf>,
}
