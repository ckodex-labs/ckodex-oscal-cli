#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ModelArgs {
    #[command(subcommand)]
    pub action: ModelAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ModelAction {
    List {
        /// OSCAL model kind to list.
        #[arg(value_enum)]
        model: ModelKind,
        /// Maximum number of models to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Server-side model filter.
        #[arg(long, default_value = "")]
        filter: String,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// OSCAL model kind to inspect.
        #[arg(value_enum)]
        model: ModelKind,
        /// Model UUID.
        uuid: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ModelKind {
    Catalog,
    Profile,
    ComponentDefinition,
    Ssp,
    AssessmentPlan,
    AssessmentResults,
    Poam,
    Mapping,
}
