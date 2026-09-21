#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ExportCliArgs {
    #[command(subcommand)]
    pub action: ExportCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ExportCliAction {
    /// Export an OSCAL document to OASIS SARIF v2.1.0 report format for GitHub Actions / Azure DevOps.
    Sarif {
        /// Path to input OSCAL document.
        #[arg(long, short = 'i')]
        input: PathBuf,
        /// Output path for SARIF JSON report.
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
    /// Export an OSCAL document to GitLab Security Scanner Report JSON format.
    Gitlab {
        /// Path to input OSCAL document.
        #[arg(long, short = 'i')]
        input: PathBuf,
        /// Output path for GitLab Security Report JSON.
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
}
