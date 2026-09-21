#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct AuditArgs {
    /// Target Kubernetes namespace to audit (default: all or active namespace).
    #[arg(long, short = 'n')]
    pub namespace: Option<String>,
    /// Optional directory containing custom compiled OSCAL Rego rules.
    #[arg(long)]
    pub rules_dir: Option<PathBuf>,
    /// Output file path for the generated OSCAL assessment-results document.
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
}
