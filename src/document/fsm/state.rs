use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    E0Unverified,
    E1SchemaValid,
    E2LinterPassed,
    E3FedrampPassed,
    E4AuditPassed,
    E5ContinuousMonitoring,
}

impl fmt::Display for EvidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::E0Unverified => write!(f, "E0 (Unverified Draft)"),
            Self::E1SchemaValid => write!(f, "E1 (Schema Validated)"),
            Self::E2LinterPassed => write!(f, "E2 (Linter Passed)"),
            Self::E3FedrampPassed => write!(f, "E3 (FedRAMP PMO Validated)"),
            Self::E4AuditPassed => write!(f, "E4 (Live Audit Passed)"),
            Self::E5ContinuousMonitoring => write!(f, "E5 (Continuous Monitoring)"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceState {
    Draft,
    UnderReview,
    Approved,
    Operational,
    Degraded,
    Remediating,
    Archived,
}

impl fmt::Display for ComplianceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "DRAFT"),
            Self::UnderReview => write!(f, "UNDER_REVIEW"),
            Self::Approved => write!(f, "APPROVED"),
            Self::Operational => write!(f, "OPERATIONAL"),
            Self::Degraded => write!(f, "DEGRADED"),
            Self::Remediating => write!(f, "REMEDIATING"),
            Self::Archived => write!(f, "ARCHIVED"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmEvent {
    SubmitForReview,
    Approve,
    DeployToCluster,
    AuditViolationDetected { violation_count: usize },
    OpenPoam { poam_id: String },
    ResolveRemediation,
    Archive,
}

impl fmt::Display for FsmEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubmitForReview => write!(f, "submit_for_review"),
            Self::Approve => write!(f, "approve"),
            Self::DeployToCluster => write!(f, "deploy_to_cluster"),
            Self::AuditViolationDetected { violation_count } => {
                write!(f, "audit_violation_detected (count: {violation_count})")
            }
            Self::OpenPoam { poam_id } => write!(f, "open_poam ({poam_id})"),
            Self::ResolveRemediation => write!(f, "resolve_remediation"),
            Self::Archive => write!(f, "archive"),
        }
    }
}
