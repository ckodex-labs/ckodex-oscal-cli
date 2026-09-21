#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct GraphArgs {
    #[command(subcommand)]
    pub action: GraphAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum GraphAction {
    Node {
        #[command(subcommand)]
        action: GraphNodeAction,
    },
    Edge {
        #[command(subcommand)]
        action: GraphEdgeAction,
    },
    /// Inspect the append-only graph projection audit chain.
    ProjectionEvents {
        #[arg(long, help = "Filter events by claim ID")]
        claim_id: Option<String>,
        #[arg(long, help = "Filter events by edge ID")]
        edge_id: Option<String>,
    },
    Traverse {
        /// Starting graph node identifier.
        start_node: String,
        /// Comma-separated relation allow-list.
        #[arg(long = "relation", value_delimiter = ',')]
        relations: Vec<String>,
        /// Maximum traversal depth.
        #[arg(long, default_value_t = 3)]
        max_depth: i32,
        /// Minimum observed trust state.
        #[arg(long, default_value = "")]
        min_trust_state: String,
        /// Traversal algorithm accepted by the server.
        #[arg(long, default_value = "bfs")]
        traversal_mode: String,
    },
    Path {
        /// Starting graph node identifier.
        from_node: String,
        /// Destination graph node identifier.
        to_node: String,
        /// Minimum observed trust state.
        #[arg(long, default_value = "")]
        min_trust_state: String,
        /// Comma-separated relation allow-list.
        #[arg(long = "relation", value_delimiter = ',')]
        allowed_relations: Vec<String>,
    },
    Impact {
        /// Node whose downstream impact is read.
        node: String,
        /// Maximum impact depth.
        #[arg(long, default_value_t = 3)]
        max_depth: i32,
        /// Minimum observed trust state.
        #[arg(long, default_value = "")]
        min_trust_state: String,
        /// Comma-separated relation allow-list.
        #[arg(long = "relation", value_delimiter = ',')]
        relations: Vec<String>,
    },
    Explain {
        /// Claim identifier to explain.
        claim_id: String,
        /// Optional edge identifier to narrow the explanation.
        #[arg(long)]
        edge_id: Option<String>,
    },
    Trust {
        /// Claim identifier whose observed trust is read.
        claim_id: String,
        /// Optional edge identifier to narrow the trust result.
        #[arg(long)]
        edge_id: Option<String>,
    },
    Closure {
        /// Subject node identifier.
        subject_node: String,
        /// Closure purpose or policy context.
        purpose: String,
    },
    ProjectEdge {
        /// Claim identifier to project as an edge.
        claim_id: String,
        /// Optional edge identifier; auto-generated if omitted.
        #[arg(long)]
        edge_id: Option<String>,
    },
    DeleteEdge {
        /// Edge identifier to delete.
        edge_id: String,
    },
}
