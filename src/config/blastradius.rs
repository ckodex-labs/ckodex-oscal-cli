#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct BlastRadiusArgs {
    /// Primary OSCAL document (Catalog, Profile, SSP, Assessment Results, or POA&M).
    pub file: PathBuf,
    /// Target identifier (Control ID, Parameter ID, Component UUID, Finding ID).
    #[arg(long)]
    pub target: String,
    /// Optional context OSCAL documents for multi-model dependency tracing.
    #[arg(long = "context", value_name = "FILE")]
    pub context_files: Vec<PathBuf>,
    /// Maximum traversal depth across dependency relationships.
    #[arg(long, default_value_t = 4)]
    pub depth: usize,
}
