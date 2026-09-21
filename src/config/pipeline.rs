#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum PipelineAction {
    /// Run full compliance pipeline: Ingest SBOM -> Evaluate Policies -> OSCAL Assessment -> SLSA Provenance -> SARIF & GitLab Export.
    Run {
        /// Compliance jurisdiction (us, ca, eu, enterprise).
        #[arg(long, short = 'j', default_value = "us")]
        jurisdiction: String,
        /// Path to CycloneDX or SPDX SBOM JSON file.
        #[arg(long)]
        sbom: Option<PathBuf>,
        /// Path to Kubernetes or cloud workload manifest JSON/YAML file.
        #[arg(long, short = 'w')]
        workload: Option<PathBuf>,
        /// Specific rule IDs to evaluate (defaults to all built-in rules if omitted).
        #[arg(long = "rule")]
        rules: Vec<String>,
        /// Subject name for SLSA provenance (e.g. app:v1.0.0).
        #[arg(long, default_value = "workload-artifact")]
        subject: String,
        /// Subject SHA-256 digest for SLSA provenance.
        #[arg(
            long,
            default_value = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        )]
        digest: String,
        /// Output directory to store generated OSCAL, SLSA, SARIF, and GitLab artifacts.
        #[arg(long, short = 'o', default_value = "mizan-pipeline-output")]
        output_dir: PathBuf,
    },
}
