#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ComponentCliArgs {
    #[command(subcommand)]
    pub action: ComponentCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ComponentCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
