#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum GraphEdgeAction {
    List {
        /// Optional source node filter.
        #[arg(long, default_value = "")]
        from_node: String,
        /// Optional destination node filter.
        #[arg(long, default_value = "")]
        to_node: String,
        /// Optional relation filter.
        #[arg(long, default_value = "")]
        relation: String,
        /// Optional observed trust-state filter.
        #[arg(long, default_value = "")]
        trust_state: String,
        /// Maximum number of edges to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// Graph edge identifier.
        edge_id: String,
    },
}
