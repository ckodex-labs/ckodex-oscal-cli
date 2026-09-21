#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct DaemonCliArgs {
    /// Root directory to watch for OSCAL document changes.
    #[arg(long, short = 'd', default_value = ".")]
    pub dir: PathBuf,
    /// Debounce delay in milliseconds.
    #[arg(long, default_value_t = 500)]
    pub debounce_ms: u64,
    /// Number of reconciliation cycles to execute before exiting (default: infinite).
    #[arg(long)]
    pub max_cycles: Option<usize>,
}
