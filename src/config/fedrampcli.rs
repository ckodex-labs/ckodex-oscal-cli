#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct FedrampCliArgs {
    #[command(subcommand)]
    pub action: FedrampAction,
}
