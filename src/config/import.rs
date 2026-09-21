#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ImportArgs {
    #[command(subcommand)]
    pub action: ImportAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ImportAction {
    Preflight {
        /// Path to a JSON or YAML document containing the import records.
        file: PathBuf,
    },
    Batch {
        /// Path to a JSON or YAML document containing the import records.
        file: PathBuf,
        /// Abort the entire batch if any record fails validation.
        #[arg(long)]
        all_or_nothing: bool,
    },
}
