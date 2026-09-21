#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct AssessmentResultsCliArgs {
    #[command(subcommand)]
    pub action: AssessmentResultsCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum AssessmentResultsCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
