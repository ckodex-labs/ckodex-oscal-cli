#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct DiffArgs {
    /// Path to base OSCAL document.
    pub file_a: PathBuf,
    /// Path to revised OSCAL document.
    pub file_b: PathBuf,
}
