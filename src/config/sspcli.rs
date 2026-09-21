#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct SspCliArgs {
    #[command(subcommand)]
    pub action: SspCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SspCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
