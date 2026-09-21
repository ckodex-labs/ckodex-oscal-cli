#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct FabricCliArgs {
    #[command(subcommand)]
    pub action: FabricAction,
}
