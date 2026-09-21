#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ConvertArgs {
    /// Path to input OSCAL file (JSON or YAML).
    pub file: PathBuf,
    /// Target serialization format.
    #[arg(long, value_enum, default_value_t = TargetFormat::Json)]
    pub to: TargetFormat,
    /// Output file path (defaults to stdout).
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
}
