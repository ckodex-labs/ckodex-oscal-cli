#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct CasCliArgs {
    #[command(subcommand)]
    pub action: CasCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CasCliAction {
    /// Show Content-Addressable Storage (CAS) statistics and object counts.
    Stats,
    /// Store a local file in Content-Addressable Storage and print its digest.
    Put {
        /// Path to file to store in CAS.
        file: PathBuf,
    },
    /// Retrieve a file from Content-Addressable Storage by its digest.
    Get {
        /// Cryptographic digest of the object (e.g. sha256:4a8f...).
        digest: String,
        /// Optional output file to write the retrieved content to.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Clear and prune local CAS object store cache.
    Prune,
}
