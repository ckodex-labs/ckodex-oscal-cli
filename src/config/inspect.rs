#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct InspectArgs {
    /// Path to OSCAL file to inspect.
    pub file: PathBuf,
}
