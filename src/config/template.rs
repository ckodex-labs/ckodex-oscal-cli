#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct TemplateArgs {
    /// Kind of OSCAL artifact (catalog, profile, ssp, component-definition).
    pub kind: String,
    /// Compliance standard baseline (e.g. fedramp-moderate, fedramp-high, nist-800-53-r5, iso-27001-2022, soc-2, cis-kubernetes).
    #[arg(long, default_value = "nist-800-53-r5")]
    pub standard: String,
    /// Custom document title.
    #[arg(long)]
    pub title: Option<String>,
    /// Path for the generated template file.
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
    /// Directly scaffold into a split Markdown workspace directory.
    #[arg(long = "split-to")]
    pub split_to: Option<PathBuf>,
}
