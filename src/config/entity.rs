#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct EntityArgs {
    #[command(subcommand)]
    pub action: EntityAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum EntityAction {
    List {
        /// Optional entity-type filter.
        #[arg(long, default_value = "")]
        type_filter: String,
        /// Optional lifecycle-status filter.
        #[arg(long, default_value = "")]
        status_filter: String,
        /// Maximum number of entities to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from the previous response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Get {
        /// Entity URN.
        urn: String,
    },
    Create {
        /// Path to the entity JSON or YAML document.
        file: PathBuf,
    },
    Update {
        /// Path to the entity JSON or YAML document.
        file: PathBuf,
    },
}
