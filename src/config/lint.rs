#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct LintArgs {
    /// Path to OSCAL file to lint.
    pub file: PathBuf,
    /// Automatically fix common lint issues (UUID generation, last-modified timestamp bump).
    #[arg(long)]
    pub fix: bool,
}
