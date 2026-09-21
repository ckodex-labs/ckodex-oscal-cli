#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ValidateArgs {
    /// Path to OSCAL JSON or YAML file to validate.
    pub file: PathBuf,
    /// Enforce strict Metaschema constraint checking.
    #[arg(long)]
    pub strict: bool,
    /// Quiet output; exits with status code only.
    #[arg(long, short)]
    pub quiet: bool,
}
