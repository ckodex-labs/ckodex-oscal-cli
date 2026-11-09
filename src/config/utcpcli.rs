#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;
use clap::{Args, Subcommand};

#[derive(Clone, Debug, Args)]
pub struct UtcpCliArgs {
    #[command(subcommand)]
    pub action: UtcpCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum UtcpCliAction {
    /// Generate the Universal Tool Calling Protocol (UTCP v1.0.1) Manual for Mizan.
    Manual {
        /// Primary UTCP transport / call_template_type (cli, http, grpc, mcp, sse, text).
        #[arg(long, short = 't', default_value = "cli")]
        transport: String,
        /// Base URL for HTTP/gRPC endpoints.
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,
        /// Optional output file path for the UTCP manual JSON (e.g. mizan.tool or utcp-manual.json).
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Generate a unified `.utcp_config.json` compatible with `rs-utcp`, `typescript-utcp`, `go-utcp`, `python-utcp`, and `utcp-mcp`.
    Config {
        /// Base URL for HTTP/gRPC endpoints.
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,
        /// Optional output file path (defaults to stdout or `.utcp_config.json` when specified).
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Validate a UTCP Manual JSON file against `utcp-specification` and CKODEX transport invariants.
    Validate {
        /// Path to UTCP manual JSON file.
        file: PathBuf,
        /// Exit with non-zero status code when validation errors are detected.
        #[arg(long)]
        strict: bool,
    },
    /// Invoke a Mizan tool directly via the Universal Tool Calling Protocol and emit a SHA-256 execution receipt.
    Call {
        /// Name of the UTCP tool to invoke (e.g. fabric_status, inspect_document, validate_fedramp).
        tool: String,
        /// JSON object string containing tool input arguments.
        #[arg(long, short = 'a', default_value = "{}")]
        args: String,
    },
    /// Generate an official UTCP SDK client integration snippet (`rs-utcp`, `typescript-utcp`, `go-utcp`, `python-utcp`, `utcp-mcp`).
    Sdk {
        /// Target SDK language (rust, typescript, go, python, mcp).
        #[arg(long, short = 'l', default_value = "rust")]
        lang: String,
        /// Base URL for HTTP/gRPC endpoints.
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,
        /// Optional output file path.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Search UTCP tools by tag, name, or description word match (`tag_and_description_word_match`).
    Search {
        /// Search query string matching tool name, tags, or description.
        #[arg(default_value = "")]
        query: String,
        /// Maximum number of matching tools to return.
        #[arg(long, short = 'l', default_value_t = 10)]
        limit: usize,
    },
}
