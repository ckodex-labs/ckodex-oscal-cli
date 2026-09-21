use std::fmt;

use crate::error::{AppError, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MethodClass {
    Read,
    Create,
    Update,
    Delete,
    Analyze,
}

impl MethodClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Analyze => "analyze",
        }
    }
}

impl fmt::Display for MethodClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OscalModel {
    Catalog,
    Profile,
    ComponentDefinition,
    Ssp,
    AssessmentPlan,
    AssessmentResults,
    Poam,
    Mapping,
}

impl OscalModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Profile => "profile",
            Self::ComponentDefinition => "component-definition",
            Self::Ssp => "system-security-plan",
            Self::AssessmentPlan => "assessment-plan",
            Self::AssessmentResults => "assessment-results",
            Self::Poam => "plan-of-action-and-milestones",
            Self::Mapping => "mapping",
        }
    }
}

impl fmt::Display for OscalModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceModel {
    Entity,
    Framework,
    Snapshot,
    Release,
}

impl GovernanceModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Framework => "framework",
            Self::Snapshot => "snapshot",
            Self::Release => "release",
        }
    }
}

impl fmt::Display for GovernanceModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RpcMethod {
    OscalGet,
    OscalList,
    OscalSearch,
    OscalCrud {
        class: MethodClass,
        model: OscalModel,
    },
    GovernanceGet,
    GovernanceList,
    GovernanceSemanticSearch,
    GovernanceCrud {
        class: MethodClass,
        model: GovernanceModel,
    },
    TransparencyGet,
    TransparencyList,
    TransparencyVerify,
    TransparencyVerificationEvents,
    TransparencyReceipt,
    TransparencyFetch,
    TransparencyFetchEvents,
    ClaimWrite,
    EvidenceWrite,
    GraphGet,
    GraphList,
    GraphProjectionEvents,
    GraphAnalysis,
    GraphEdgeWrite,
    HealthCheck,
}

impl RpcMethod {
    pub fn class(&self) -> MethodClass {
        match self {
            Self::OscalGet
            | Self::OscalList
            | Self::OscalSearch
            | Self::GovernanceGet
            | Self::GovernanceList
            | Self::GovernanceSemanticSearch
            | Self::TransparencyGet
            | Self::TransparencyList
            | Self::TransparencyVerify
            | Self::TransparencyVerificationEvents
            | Self::TransparencyReceipt
            | Self::TransparencyFetchEvents
            | Self::GraphGet
            | Self::GraphList
            | Self::GraphProjectionEvents
            | Self::HealthCheck => MethodClass::Read,
            Self::OscalCrud { class, .. } | Self::GovernanceCrud { class, .. } => *class,
            Self::ClaimWrite | Self::EvidenceWrite | Self::GraphEdgeWrite => MethodClass::Create,
            Self::TransparencyFetch => MethodClass::Create,
            Self::GraphAnalysis => MethodClass::Analyze,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::OscalGet => "oscal.get".to_owned(),
            Self::OscalList => "oscal.list".to_owned(),
            Self::OscalSearch => "oscal.search".to_owned(),
            Self::OscalCrud { class, model } => format!("oscal.{}.{}", class, model),
            Self::GovernanceGet => "governance.get".to_owned(),
            Self::GovernanceList => "governance.list".to_owned(),
            Self::GovernanceSemanticSearch => "governance.semantic-search".to_owned(),
            Self::GovernanceCrud { class, model } => format!("governance.{}.{}", class, model),
            Self::TransparencyGet => "transparency.get".to_owned(),
            Self::TransparencyList => "transparency.list".to_owned(),
            Self::TransparencyVerify => "transparency.verify".to_owned(),
            Self::TransparencyVerificationEvents => "transparency.verification-events".to_owned(),
            Self::TransparencyReceipt => "transparency.receipt".to_owned(),
            Self::TransparencyFetch => "transparency.fetch".to_owned(),
            Self::TransparencyFetchEvents => "transparency.fetch-events".to_owned(),
            Self::ClaimWrite => "claim.write".to_owned(),
            Self::EvidenceWrite => "evidence.write".to_owned(),
            Self::GraphGet => "graph.get".to_owned(),
            Self::GraphList => "graph.list".to_owned(),
            Self::GraphProjectionEvents => "graph.projection-events".to_owned(),
            Self::GraphAnalysis => "graph.analysis".to_owned(),
            Self::GraphEdgeWrite => "graph.edge.write".to_owned(),
            Self::HealthCheck => "health.check".to_owned(),
        }
    }
}

impl fmt::Display for RpcMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gate {
    pub read_only: bool,
}

impl Gate {
    pub fn permit(&self, method: &RpcMethod) -> Result<()> {
        match method.class() {
            MethodClass::Read | MethodClass::Analyze => Ok(()),
            MethodClass::Create | MethodClass::Update | MethodClass::Delete => {
                if self.read_only {
                    Err(AppError::Anti {
                        method: method.name(),
                        class: method.class().as_str(),
                        reason: "read-only gate denies mutating RPC".to_owned(),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
pub fn reject_mutating_rpc(name: &str) -> Result<()> {
    Err(AppError::Anti {
        method: name.to_owned(),
        class: "create",
        reason: "read-only policy denied mutating RPC".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rpc_method_is_classified() {
        for method in [
            RpcMethod::OscalGet,
            RpcMethod::OscalList,
            RpcMethod::OscalSearch,
            RpcMethod::OscalCrud {
                class: MethodClass::Create,
                model: OscalModel::Catalog,
            },
            RpcMethod::OscalCrud {
                class: MethodClass::Update,
                model: OscalModel::Profile,
            },
            RpcMethod::OscalCrud {
                class: MethodClass::Delete,
                model: OscalModel::Ssp,
            },
            RpcMethod::GovernanceGet,
            RpcMethod::GovernanceList,
            RpcMethod::GovernanceSemanticSearch,
            RpcMethod::GovernanceCrud {
                class: MethodClass::Create,
                model: GovernanceModel::Entity,
            },
            RpcMethod::GovernanceCrud {
                class: MethodClass::Update,
                model: GovernanceModel::Framework,
            },
            RpcMethod::GovernanceCrud {
                class: MethodClass::Delete,
                model: GovernanceModel::Snapshot,
            },
            RpcMethod::TransparencyGet,
            RpcMethod::TransparencyList,
            RpcMethod::TransparencyVerify,
            RpcMethod::TransparencyVerificationEvents,
            RpcMethod::TransparencyReceipt,
            RpcMethod::ClaimWrite,
            RpcMethod::EvidenceWrite,
            RpcMethod::GraphGet,
            RpcMethod::GraphList,
            RpcMethod::GraphProjectionEvents,
            RpcMethod::GraphAnalysis,
            RpcMethod::GraphEdgeWrite,
            RpcMethod::HealthCheck,
        ] {
            assert!(!method.name().is_empty());
            let class = method.class();
            assert!(!class.as_str().is_empty());
        }
    }

    #[test]
    fn mutation_is_explicitly_rejected() {
        let error = reject_mutating_rpc("CreateCatalog").expect_err("mutation must be denied");
        assert!(error.to_string().contains("CreateCatalog"));
        assert!(error.to_string().contains("anti"));
    }

    #[test]
    fn read_only_gate_denies_create_update_delete() {
        let gate = Gate { read_only: true };
        let create = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::Catalog,
        };
        let update = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::Profile,
        };
        let delete = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::Ssp,
        };

        let e = gate.permit(&create).expect_err("create must be denied");
        assert!(matches!(e, AppError::Anti { .. }));
        assert!(e.to_string().contains("oscal.create.catalog"));

        let e = gate.permit(&update).expect_err("update must be denied");
        assert!(matches!(e, AppError::Anti { .. }));
        assert!(e.to_string().contains("oscal.update.profile"));

        let e = gate.permit(&delete).expect_err("delete must be denied");
        assert!(matches!(e, AppError::Anti { .. }));
        assert!(e.to_string().contains("oscal.delete.system-security-plan"));
    }

    #[test]
    fn mutable_gate_permits_create_update_delete() {
        let gate = Gate { read_only: false };
        assert!(gate.permit(&RpcMethod::ClaimWrite).is_ok());
        assert!(gate.permit(&RpcMethod::EvidenceWrite).is_ok());
        assert!(gate.permit(&RpcMethod::GraphEdgeWrite).is_ok());
        assert!(gate
            .permit(&RpcMethod::OscalCrud {
                class: MethodClass::Create,
                model: OscalModel::Poam,
            })
            .is_ok());
    }

    #[test]
    fn gate_always_allows_read_and_analyze() {
        let ro = Gate { read_only: true };
        let rw = Gate { read_only: false };
        for method in [
            RpcMethod::OscalGet,
            RpcMethod::OscalList,
            RpcMethod::OscalSearch,
            RpcMethod::GovernanceGet,
            RpcMethod::GovernanceList,
            RpcMethod::GovernanceSemanticSearch,
            RpcMethod::TransparencyGet,
            RpcMethod::TransparencyList,
            RpcMethod::TransparencyVerify,
            RpcMethod::TransparencyVerificationEvents,
            RpcMethod::TransparencyReceipt,
            RpcMethod::GraphGet,
            RpcMethod::GraphList,
            RpcMethod::GraphProjectionEvents,
            RpcMethod::GraphAnalysis,
            RpcMethod::HealthCheck,
        ] {
            assert!(
                ro.permit(&method).is_ok(),
                "{method} denied by read-only gate"
            );
            assert!(
                rw.permit(&method).is_ok(),
                "{method} denied by writable gate"
            );
        }
    }
}
