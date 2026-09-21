#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct SplitArgs {
    /// Path to the source OSCAL document.
    pub file: PathBuf,
    /// Destination directory for the split Markdown workspace.
    #[arg(long, short = 'o')]
    pub output_dir: PathBuf,
}
