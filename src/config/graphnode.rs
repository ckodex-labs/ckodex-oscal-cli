#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum GraphNodeAction {
    List {
        /// Optional node-kind filter.
        #[arg(long, default_value = "")]
        kind: String,
        /// Optional label substring filter.
        #[arg(long, default_value = "")]
        label_filter: String,
        /// Maximum number of nodes to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// Graph node identifier.
        node_id: String,
    },
}
