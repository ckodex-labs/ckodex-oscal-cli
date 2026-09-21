#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct MappingCliArgs {
    #[command(subcommand)]
    pub action: MappingCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum MappingCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
