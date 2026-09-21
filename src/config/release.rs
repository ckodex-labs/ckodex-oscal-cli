#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    pub action: ReleaseAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ReleaseAction {
    List {
        /// Maximum number of releases to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Create {
        /// Release name.
        name: String,
        /// Snapshot name to release.
        snapshot_name: String,
    },
}
