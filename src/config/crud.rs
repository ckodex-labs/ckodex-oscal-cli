#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum CrudAction {
    Create {
        /// Path to the OSCAL document (JSON or YAML).
        file: PathBuf,
    },
    Get {
        /// UUID of the document to retrieve.
        uuid: String,
    },
    List {
        /// Maximum number of items to return.
        #[arg(long, default_value_t = 25)]
        page_size: i32,
        /// Opaque continuation token from a previous list response.
        #[arg(long, default_value = "")]
        page_token: String,
    },
    Update {
        /// UUID of the document to update.
        uuid: String,
        /// Path to the updated OSCAL document (JSON or YAML).
        file: PathBuf,
    },
    Delete {
        /// UUID of the document to delete.
        uuid: String,
    },
}
