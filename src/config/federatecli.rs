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
    /// Ingest a SARIF or GitLab Security Report and generate/update an OSCAL POA&M remediation plan.
    FromSecurityReport {
        /// Path to input security report file (JSON or SARIF).
        #[arg(long, short = 'i')]
        report: PathBuf,
        /// Format of input report: "sarif", "gitlab-sast", "auto".
        #[arg(long = "input-format", default_value = "auto")]
        input_format: String,
        /// Title for generated POA&M.
        #[arg(long, default_value = "Automated Security Remediation POA&M")]
        title: String,
        /// Optional path to existing POA&M to append findings into.
        #[arg(long)]
        existing_poam: Option<PathBuf>,
        /// Output path for the resulting POA&M document.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
}
