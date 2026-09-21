use serde::{Deserialize, Serialize};

use crate::{
    document::fsm::state::{ComplianceState, EvidenceLevel, FsmEvent},
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionRecord {
    pub from_state: ComplianceState,
    pub to_state: ComplianceState,
    pub event: FsmEvent,
    pub evidence_level: EvidenceLevel,
    pub timestamp: String,
    pub actor: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FsmHistory {
    pub current_state: ComplianceState,
    pub current_evidence: EvidenceLevel,
    pub transitions: Vec<TransitionRecord>,
}

pub struct FsmRuntime {
    history: FsmHistory,
}

impl FsmRuntime {
    pub fn new(initial_state: ComplianceState, initial_evidence: EvidenceLevel) -> Self {
        Self {
            history: FsmHistory {
                current_state: initial_state,
                current_evidence: initial_evidence,
                transitions: Vec::new(),
            },
        }
    }

    pub fn state(&self) -> ComplianceState {
        self.history.current_state
    }

    pub fn evidence(&self) -> EvidenceLevel {
        self.history.current_evidence
    }

    pub fn history(&self) -> &FsmHistory {
        &self.history
    }

    pub fn update_evidence(&mut self, evidence: EvidenceLevel) {
        self.history.current_evidence = evidence;
    }

    pub fn transition(
        &mut self,
        event: FsmEvent,
        actor: &str,
        rationale: &str,
    ) -> Result<ComplianceState> {
        let from_state = self.history.current_state;
        let evidence = self.history.current_evidence;

        let to_state = match (from_state, &event) {
            (ComplianceState::Draft, FsmEvent::SubmitForReview) => {
                if evidence < EvidenceLevel::E1SchemaValid {
                    return Err(AppError::Configuration(format!(
                        "Cannot transition DRAFT -> UNDER_REVIEW: Requires at least E1 schema validation (current: {evidence})"
                    )));
                }
                ComplianceState::UnderReview
            }
            (ComplianceState::UnderReview, FsmEvent::Approve) => {
                if evidence < EvidenceLevel::E3FedrampPassed {
                    return Err(AppError::Configuration(format!(
                        "Cannot transition UNDER_REVIEW -> APPROVED: Requires at least E3 FedRAMP PMO baseline validation (current: {evidence})"
                    )));
                }
                ComplianceState::Approved
            }
            (ComplianceState::Approved, FsmEvent::DeployToCluster) => {
                if evidence < EvidenceLevel::E4AuditPassed {
                    return Err(AppError::Configuration(format!(
                        "Cannot transition APPROVED -> OPERATIONAL: Requires E4 live cluster audit verification (current: {evidence})"
                    )));
                }
                ComplianceState::Operational
            }
            (
                ComplianceState::Operational,
                FsmEvent::AuditViolationDetected { violation_count },
            ) => {
                if *violation_count == 0 {
                    return Err(AppError::Configuration(
                        "Cannot transition OPERATIONAL -> DEGRADED with zero violations"
                            .to_string(),
                    ));
                }
                ComplianceState::Degraded
            }
            (ComplianceState::Degraded, FsmEvent::OpenPoam { .. }) => ComplianceState::Remediating,
            (ComplianceState::Remediating, FsmEvent::ResolveRemediation) => {
                if evidence < EvidenceLevel::E4AuditPassed {
                    return Err(AppError::Configuration(format!(
                        "Cannot resolve remediation to OPERATIONAL: Clean live audit E4 proof required (current: {evidence})"
                    )));
                }
                ComplianceState::Operational
            }
            (_, FsmEvent::Archive) => ComplianceState::Archived,
            (state, ev) => {
                return Err(AppError::Configuration(format!(
                    "Illegal FSM state transition: Cannot trigger event '{ev}' while in state '{state}'"
                )));
            }
        };

        let now = chrono::Utc::now().to_rfc3339();
        let record = TransitionRecord {
            from_state,
            to_state,
            event,
            evidence_level: evidence,
            timestamp: now,
            actor: actor.to_string(),
            rationale: rationale.to_string(),
        };

        self.history.current_state = to_state;
        self.history.transitions.push(record);

        Ok(to_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsm_lifecycle_happy_path() {
        let mut fsm = FsmRuntime::new(ComplianceState::Draft, EvidenceLevel::E0Unverified);

        // Cannot submit unverified draft
        assert!(fsm
            .transition(FsmEvent::SubmitForReview, "auditor", "Initial draft")
            .is_err());

        // Elevate to E1 (Schema Valid) and submit
        fsm.update_evidence(EvidenceLevel::E1SchemaValid);
        assert_eq!(
            fsm.transition(FsmEvent::SubmitForReview, "author", "Ready for review")
                .unwrap(),
            ComplianceState::UnderReview
        );

        // Cannot approve without FedRAMP E3
        assert!(fsm
            .transition(FsmEvent::Approve, "ciso", "Looks good")
            .is_err());

        // Elevate to E3 and approve
        fsm.update_evidence(EvidenceLevel::E3FedrampPassed);
        assert_eq!(
            fsm.transition(FsmEvent::Approve, "ciso", "FedRAMP Moderate satisfied")
                .unwrap(),
            ComplianceState::Approved
        );

        // Deploy to cluster with E4 live audit passed
        fsm.update_evidence(EvidenceLevel::E4AuditPassed);
        assert_eq!(
            fsm.transition(
                FsmEvent::DeployToCluster,
                "devops",
                "Deployed to production"
            )
            .unwrap(),
            ComplianceState::Operational
        );

        // Violation detected by live scanner
        assert_eq!(
            fsm.transition(
                FsmEvent::AuditViolationDetected { violation_count: 2 },
                "kube_auditor",
                "Root container violation detected"
            )
            .unwrap(),
            ComplianceState::Degraded
        );

        // Open POA&M ticket
        assert_eq!(
            fsm.transition(
                FsmEvent::OpenPoam {
                    poam_id: "POAM-2026-001".to_string()
                },
                "security_lead",
                "Assigned to platform engineering"
            )
            .unwrap(),
            ComplianceState::Remediating
        );

        // Resolve remediation
        assert_eq!(
            fsm.transition(
                FsmEvent::ResolveRemediation,
                "platform_lead",
                "Applied readOnlyRootFilesystem in Helm chart"
            )
            .unwrap(),
            ComplianceState::Operational
        );
        assert_eq!(fsm.history().transitions.len(), 6);
    }
}
