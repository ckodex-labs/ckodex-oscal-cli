use jsonschema::Validator;
use serde_json::Value;
use std::sync::OnceLock;

use crate::error::{AppError, Result};

pub const SCHEMA_VERSION: &str = "1.2.3";

const CATALOG_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_catalog_schema.json");
const PROFILE_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_profile_schema.json");
const SSP_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_ssp_schema.json");
const COMPONENT_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_component_schema.json");
const ASSESSMENT_PLAN_SCHEMA: &str =
    include_str!("../../schemas/v1.2.3/oscal_assessment-plan_schema.json");
const ASSESSMENT_RESULTS_SCHEMA: &str =
    include_str!("../../schemas/v1.2.3/oscal_assessment-results_schema.json");
const POAM_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_poam_schema.json");
const MAPPING_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_mapping_schema.json");
const COMPLETE_SCHEMA: &str = include_str!("../../schemas/v1.2.3/oscal_complete_schema.json");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DocumentKind {
    Catalog,
    Profile,
    Ssp,
    ComponentDefinition,
    AssessmentPlan,
    AssessmentResults,
    Poam,
    Mapping,
    Complete,
}

impl DocumentKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Profile => "profile",
            Self::Ssp => "system-security-plan",
            Self::ComponentDefinition => "component-definition",
            Self::AssessmentPlan => "assessment-plan",
            Self::AssessmentResults => "assessment-results",
            Self::Poam => "plan-of-action-and-milestones",
            Self::Mapping => "mapping-collection",
            Self::Complete => "complete",
        }
    }

    pub fn root_key(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Profile => "profile",
            Self::Ssp => "system-security-plan",
            Self::ComponentDefinition => "component-definition",
            Self::AssessmentPlan => "assessment-plan",
            Self::AssessmentResults => "assessment-results",
            Self::Poam => "plan-of-action-and-milestones",
            Self::Mapping => "mapping-collection",
            Self::Complete => "complete",
        }
    }

    pub fn from_root_key(key: &str) -> Option<Self> {
        match key {
            "catalog" => Some(Self::Catalog),
            "profile" => Some(Self::Profile),
            "system-security-plan" | "ssp" => Some(Self::Ssp),
            "component-definition" | "component" => Some(Self::ComponentDefinition),
            "assessment-plan" | "ap" => Some(Self::AssessmentPlan),
            "assessment-results" | "ar" => Some(Self::AssessmentResults),
            "plan-of-action-and-milestones" | "poam" => Some(Self::Poam),
            "mapping-collection" | "mapping" => Some(Self::Mapping),
            _ => None,
        }
    }

    pub fn raw_schema_str(&self) -> &'static str {
        match self {
            Self::Catalog => CATALOG_SCHEMA,
            Self::Profile => PROFILE_SCHEMA,
            Self::Ssp => SSP_SCHEMA,
            Self::ComponentDefinition => COMPONENT_SCHEMA,
            Self::AssessmentPlan => ASSESSMENT_PLAN_SCHEMA,
            Self::AssessmentResults => ASSESSMENT_RESULTS_SCHEMA,
            Self::Poam => POAM_SCHEMA,
            Self::Mapping => MAPPING_SCHEMA,
            Self::Complete => COMPLETE_SCHEMA,
        }
    }
}

pub struct SchemaRegistry {
    catalog: Validator,
    profile: Validator,
    ssp: Validator,
    component: Validator,
    assessment_plan: Validator,
    assessment_results: Validator,
    poam: Validator,
    mapping: Validator,
    complete: Validator,
}

static REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();

impl SchemaRegistry {
    pub fn global() -> Result<&'static SchemaRegistry> {
        if let Some(reg) = REGISTRY.get() {
            return Ok(reg);
        }
        let reg = Self::build()?;
        let _ = REGISTRY.set(reg);
        Ok(REGISTRY.get().expect("registry is initialized"))
    }

    fn build() -> Result<Self> {
        let compile = |raw: &'static str, name: &str| -> Result<Validator> {
            let schema_val: Value = serde_json::from_str(raw).map_err(|e| {
                AppError::Configuration(format!("Failed to parse schema for {name}: {e}"))
            })?;
            Validator::new(&schema_val).map_err(|e| {
                AppError::Configuration(format!("Failed to compile schema for {name}: {e}"))
            })
        };

        Ok(Self {
            catalog: compile(CATALOG_SCHEMA, "catalog")?,
            profile: compile(PROFILE_SCHEMA, "profile")?,
            ssp: compile(SSP_SCHEMA, "ssp")?,
            component: compile(COMPONENT_SCHEMA, "component-definition")?,
            assessment_plan: compile(ASSESSMENT_PLAN_SCHEMA, "assessment-plan")?,
            assessment_results: compile(ASSESSMENT_RESULTS_SCHEMA, "assessment-results")?,
            poam: compile(POAM_SCHEMA, "poam")?,
            mapping: compile(MAPPING_SCHEMA, "mapping")?,
            complete: compile(COMPLETE_SCHEMA, "complete")?,
        })
    }

    pub fn validator_for(&self, kind: DocumentKind) -> &Validator {
        match kind {
            DocumentKind::Catalog => &self.catalog,
            DocumentKind::Profile => &self.profile,
            DocumentKind::Ssp => &self.ssp,
            DocumentKind::ComponentDefinition => &self.component,
            DocumentKind::AssessmentPlan => &self.assessment_plan,
            DocumentKind::AssessmentResults => &self.assessment_results,
            DocumentKind::Poam => &self.poam,
            DocumentKind::Mapping => &self.mapping,
            DocumentKind::Complete => &self.complete,
        }
    }
}
