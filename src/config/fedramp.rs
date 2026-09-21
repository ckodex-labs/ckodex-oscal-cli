#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum FedrampAction {
    /// Validate an SSP or Component Definition against FedRAMP PMO baseline rules.
    Validate {
        /// Path to SSP or Component Definition document.
        file: PathBuf,
        /// FedRAMP baseline level (low, moderate, high).
        #[arg(long, default_value = "moderate")]
        baseline: String,
    },
}
