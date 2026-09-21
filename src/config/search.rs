#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct SearchArgs {
    /// Text query sent to the selected read-only search path.
    pub query: String,
    /// Use semantic governance search instead of standard model search.
    #[arg(long)]
    pub semantic: bool,
    /// Standard-search model types, comma-separated; unavailable in semantic mode.
    #[arg(long = "model", value_delimiter = ',')]
    pub model_types: Vec<String>,
    /// Semantic-search framework filter; available only with --semantic.
    #[arg(long)]
    pub framework: Option<String>,
    /// Result count for semantic search, or page size for standard search.
    #[arg(long, default_value_t = 10)]
    pub top_k: i32,
    /// Opaque continuation token returned by standard search.
    #[arg(long, default_value = "")]
    pub page_token: String,
}
