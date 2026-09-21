#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct AssessmentPlanCliArgs {
    #[command(subcommand)]
    pub action: AssessmentPlanCliAction,
}

#[derive(Clone, Debug, Subcommand)]
pub enum AssessmentPlanCliAction {
    Validate(ValidateArgs),
    Convert(ConvertArgs),
    Inspect(InspectArgs),
    #[command(flatten)]
    Crud(CrudAction),
}
