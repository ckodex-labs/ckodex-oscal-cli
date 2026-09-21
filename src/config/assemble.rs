#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct AssembleArgs {
    /// Path to the split Markdown workspace directory.
    pub input_dir: PathBuf,
    /// Path for the assembled OSCAL output document (defaults to stdout in dry-run mode).
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
}
