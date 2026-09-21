#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct FrameworkArgs {
    #[command(subcommand)]
    pub action: FrameworkAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum FrameworkAction {
    List {
        /// Optional jurisdiction filter.
        #[arg(long)]
        jurisdiction: Option<String>,
        /// Maximum number of frameworks to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// Framework reference identifier.
        ref_id: String,
    },
    Ingest {
        /// Path to the raw requirements file to ingest.
        file: PathBuf,
        /// Input format, e.g. "eu-ai-act-json".
        #[arg(long)]
        format: String,
        /// Target framework identifier.
        #[arg(long)]
        framework: String,
    },
}
