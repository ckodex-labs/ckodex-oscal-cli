#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct InitCliArgs {
    /// Repository root directory to scan and auto-discover compliance controls from.
    #[arg(long = "from-repo", default_value = ".")]
    pub from_repo: PathBuf,
    /// Output directory for generated governance assets.
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
}
