#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct SbomCliArgs {
    #[command(subcommand)]
    pub action: SbomCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SbomCliAction {
    /// Ingest a CycloneDX v1.5/v1.6 SBOM and synthesize an OSCAL Component Definition.
    Import {
        /// Path to CycloneDX SBOM JSON file.
        #[arg(long, short = 'i')]
        input: PathBuf,
        /// Output file path for generated OSCAL component-definition.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
}
