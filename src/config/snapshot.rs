#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    pub action: SnapshotAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SnapshotAction {
    List {
        /// Maximum number of snapshots to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// Snapshot name.
        name: String,
    },
    Create {
        /// Snapshot name.
        name: String,
    },
}
