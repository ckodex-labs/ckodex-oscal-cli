#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct PoamCliArgs {
    #[command(subcommand)]
    pub action: PoamCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum PoamCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
