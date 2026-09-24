#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct FixCliArgs {
    /// Rule ID to remediate (e.g. cis-k8s-5.2.1, cis-k8s-5.2.2, cis-k8s-5.2.5, cis-k8s-5.2.6, cis-k8s-5.2.7).
    #[arg(long, short = 'r')]
    pub rule: String,
    /// Target Kubernetes manifest (YAML/JSON) or Dockerfile to remediate.
    #[arg(long, short = 'f')]
    pub file: PathBuf,
    /// Perform a dry-run preview and output the diff without writing to disk.
    #[arg(long)]
    pub dry_run: bool,
}
