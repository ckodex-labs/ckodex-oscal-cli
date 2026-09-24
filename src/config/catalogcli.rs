#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct CatalogCliArgs {
    #[command(subcommand)]
    pub action: CatalogCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CatalogCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    /// List built-in tri-jurisdictional compliance catalogs (US NIST 800-53 r5, Canadian ITSG-33, EU EUCS/ISO 27001).
    List,
    /// CRUD actions against the remote catalog service.
    #[command(subcommand)]
    Crud(CrudAction),
    /// Export a built-in catalog to a file.
    Export {
        /// Jurisdiction to export (us, ca, eu, enterprise).
        #[arg(long, short = 'j', default_value = "us")]
        jurisdiction: String,
        /// Output file path.
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
    /// Create or extend an enterprise baseline catalog with custom company controls.
    Extend {
        /// Base jurisdiction to inherit from (us, ca, eu).
        #[arg(long, short = 'b', default_value = "us")]
        base: String,
        /// Title for the new enterprise catalog.
        #[arg(long, default_value = "Enterprise Compliance Baseline")]
        title: String,
        /// Custom control ID to append (e.g. corp-sec-01).
        #[arg(long)]
        add_control_id: Option<String>,
        /// Custom control title.
        #[arg(long)]
        add_control_title: Option<String>,
        /// Custom control description.
        #[arg(long)]
        add_control_desc: Option<String>,
        /// Output file path for generated enterprise catalog JSON.
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
    /// Export an OSCAL catalog or profile to an auditor-friendly spreadsheet CSV matrix.
    ExportMatrix {
        /// Jurisdiction to export (us, ca, eu, enterprise).
        #[arg(long, short = 'j', default_value = "us")]
        jurisdiction: String,
        /// Output CSV file path.
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
}
