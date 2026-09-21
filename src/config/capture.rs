#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct CaptureArgs {
    #[command(subcommand)]
    pub action: CaptureAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CaptureAction {
    List,
    Verify {
        /// Optional capture identifier; omit to verify all captures.
        id: Option<String>,
    },
    Prune {
        /// Candidate age threshold in days.
        #[arg(long, value_name = "DAYS")]
        older_than_days: u64,
        #[arg(long, help = "Delete verified local captures; omit for a dry run")]
        confirm: bool,
    },
    Show {
        /// Capture identifier.
        id: String,
    },
    Export {
        /// Capture identifier to export as verified evidence.
        id: String,
    },
}
