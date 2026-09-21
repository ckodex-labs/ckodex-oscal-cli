#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ResolveArgs {
    /// Path to OSCAL Profile (JSON or YAML).
    pub file: PathBuf,
    /// Output file path for resolved catalog (defaults to stdout).
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
}
