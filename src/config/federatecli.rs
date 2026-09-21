#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct FederateCliArgs {
    #[command(subcommand)]
    pub action: FederateCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum FederateCliAction {
    /// Auto-generate a POA&M remediation plan from live Kubernetes assessment findings.
    AutoPoam {
        /// Path to System Security Plan (SSP) document.
        #[arg(long, short = 's')]
        ssp: PathBuf,
        /// Path to Assessment Results document.
        #[arg(long, short = 'a')]
        assessment: PathBuf,
        /// Title for the generated POA&M document.
        #[arg(long, default_value = "Auto-Remediation POA&M")]
        title: String,
        /// Output file path for generated POA&M document.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
}
