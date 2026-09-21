#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct TxCliArgs {
    #[command(subcommand)]
    pub action: TxCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum TxCliAction {
    /// Inspect or create an atomic compliance transaction with Write-Ahead Logging.
    Begin {
        /// Optional custom transaction identifier.
        #[arg(long)]
        tx_id: Option<String>,
    },
}
