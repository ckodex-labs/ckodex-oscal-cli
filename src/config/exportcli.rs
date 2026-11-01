#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ExportCliArgs {
    #[command(subcommand)]
    pub action: ExportCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ExportCliAction {
    /// Export an OSCAL document to OASIS SARIF v2.1.0 report format for GitHub Actions / Azure DevOps.
    Sarif {
        /// Path to input OSCAL document (positional or via -i/--input).
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,
        /// Alternative flag for input file path.
        #[arg(long, short = 'i', value_name = "INPUT_FILE")]
        input_file: Option<PathBuf>,
        /// Output path for SARIF JSON report.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Export an OSCAL document to GitLab Security Scanner Report JSON format.
    Gitlab {
        /// Path to input OSCAL document (positional or via -i/--input).
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,
        /// Alternative flag for input file path.
        #[arg(long, short = 'i', value_name = "INPUT_FILE")]
        input_file: Option<PathBuf>,
        /// Output path for GitLab Security Report JSON.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Export a self-contained offline HTML evidence capsule with embedded WebCrypto Merkle verification.
    Capsule {
        /// Path to input OSCAL assessment results JSON (positional or via -a/--assessment).
        #[arg(value_name = "ASSESSMENT")]
        assessment: Option<PathBuf>,
        /// Alternative flag for assessment results path.
        #[arg(long, short = 'a', value_name = "ASSESSMENT_FILE")]
        assessment_file: Option<PathBuf>,
        /// Optional path to evidence bundle JSON.
        #[arg(long, short = 'b')]
        bundle: Option<PathBuf>,
        /// Output path for standalone capsule HTML file.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Export a shieldcn-compatible SVG badge URL, Markdown, or HTML snippet.
    Badge {
        /// Badge label (e.g. FedRAMP, SLSA, Invariant).
        #[arg(long, short = 'l', default_value = "compliance")]
        label: String,
        /// Badge message/value (e.g. Moderate, Level 3, passing).
        #[arg(long, short = 'm', default_value = "verified")]
        message: String,
        /// Badge color (e.g. green, blue, emerald, orange, red, slate).
        #[arg(long, short = 'c', default_value = "green")]
        color: String,
        /// Visual variant (default, secondary, outline, ghost, destructive, branded).
        #[arg(long, short = 'v', default_value = "secondary")]
        variant: String,
        /// WCAG 3.0 APCA contrast compliance mode (shade-800 colors).
        #[arg(long, default_value_t = 3)]
        wcag: u8,
        /// Optional icon slug (e.g. rust, nextdotjs, shield).
        #[arg(long)]
        logo: Option<String>,
        /// Optional target link when rendering Markdown.
        #[arg(long)]
        link: Option<String>,
        /// Optional input document to infer badge status (e.g. assessment.json or catalog.json).
        #[arg(long)]
        from_document: Option<PathBuf>,
        /// Optional destination output file.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
}
