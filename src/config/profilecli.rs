#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct ProfileCliArgs {
    #[command(subcommand)]
    pub action: ProfileCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum ProfileCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Resolve(ResolveArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
