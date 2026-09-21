use std::fmt;

use crate::document::fsm::state::{ComplianceState, EvidenceLevel, FsmEvent};
use crate::document::fsm::transitions::FsmRuntime;
use crate::error::IncoherenceKind;
use crate::policy::{Gate, MethodClass};
use crate::valence::Valence;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemoteLifecycleState {
    Absent,
    Creating,
    Active {
        compliance: ComplianceState,
        evidence: EvidenceLevel,
        valence: Valence,
    },
    Updating,
    Deleting,
    Deleted,
    Failed {
        op: MethodClass,
        code: String,
        retryable: bool,
    },
    Rejected {
        reason: String,
    },
    Incoherent {
        kind: IncoherenceKind,
    },
}

impl RemoteLifecycleState {
    pub fn valence(&self) -> Option<Valence> {
        if let Self::Active { valence, .. } = self {
            Some(*valence)
        } else {
            None
        }
    }
}

impl fmt::Display for RemoteLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent => write!(f, "absent"),
            Self::Creating => write!(f, "creating"),
            Self::Active {
                compliance,
                evidence,
                valence,
            } => write!(f, "active({compliance},{evidence},{valence})"),
            Self::Updating => write!(f, "updating"),
            Self::Deleting => write!(f, "deleting"),
            Self::Deleted => write!(f, "deleted"),
            Self::Failed {
                op,
                code,
                retryable,
            } => write!(f, "failed({op},{code},{retryable})"),
            Self::Rejected { reason } => write!(f, "rejected({reason})"),
            Self::Incoherent { kind } => write!(f, "incoherent({kind})"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleEvent {
    Create,
    CreateSuccess {
        compliance: ComplianceState,
        evidence: EvidenceLevel,
        valence: Valence,
    },
    CreateFailure {
        op: MethodClass,
        code: String,
        retryable: bool,
    },
    Read(Valence),
    List(Valence),
    Update(FsmEvent),
    UpdateSuccess {
        compliance: ComplianceState,
        evidence: EvidenceLevel,
        valence: Valence,
    },
    UpdateFailure {
        op: MethodClass,
        code: String,
        retryable: bool,
    },
    Delete,
    DeleteSuccess,
    DeleteFailure {
        op: MethodClass,
        code: String,
        retryable: bool,
    },
    Contradiction(IncoherenceKind),
    Reset,
    Retry,
}

impl fmt::Display for LifecycleEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::CreateSuccess { .. } => write!(f, "create-success"),
            Self::CreateFailure { .. } => write!(f, "create-failure"),
            Self::Read(v) => write!(f, "read({v})"),
            Self::List(v) => write!(f, "list({v})"),
            Self::Update(ev) => write!(f, "update({ev})"),
            Self::UpdateSuccess { .. } => write!(f, "update-success"),
            Self::UpdateFailure { .. } => write!(f, "update-failure"),
            Self::Delete => write!(f, "delete"),
            Self::DeleteSuccess => write!(f, "delete-success"),
            Self::DeleteFailure { .. } => write!(f, "delete-failure"),
            Self::Contradiction(kind) => write!(f, "contradiction({kind})"),
            Self::Reset => write!(f, "reset"),
            Self::Retry => write!(f, "retry"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleRecord {
    pub from_state: RemoteLifecycleState,
    pub to_state: RemoteLifecycleState,
    pub event: LifecycleEvent,
    pub timestamp: String,
    pub actor: String,
    pub rationale: String,
    pub from_valence: Option<Valence>,
    pub to_valence: Option<Valence>,
}

#[derive(Clone, Debug)]
pub struct LifecycleHistory {
    pub current_state: RemoteLifecycleState,
    pub transitions: Vec<LifecycleRecord>,
}

pub struct LifecycleRuntime {
    history: LifecycleHistory,
}

impl Default for LifecycleRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleRuntime {
    pub fn new() -> Self {
        Self {
            history: LifecycleHistory {
                current_state: RemoteLifecycleState::Absent,
                transitions: Vec::new(),
            },
        }
    }

    pub fn state(&self) -> &RemoteLifecycleState {
        &self.history.current_state
    }

    pub fn history(&self) -> &LifecycleHistory {
        &self.history
    }

    fn is_mutating(event: &LifecycleEvent) -> bool {
        matches!(
            event,
            LifecycleEvent::Create | LifecycleEvent::Update(_) | LifecycleEvent::Delete
        )
    }

    pub fn apply(
        &mut self,
        event: LifecycleEvent,
        gate: &Gate,
        actor: &str,
        rationale: &str,
    ) -> &RemoteLifecycleState {
        let from = self.history.current_state.clone();

        if gate.read_only && Self::is_mutating(&event) {
            let to = RemoteLifecycleState::Rejected {
                reason: "read-only gate denies mutating lifecycle event".to_owned(),
            };
            self.record(&from, &event, &to, actor, rationale);
            return &self.history.current_state;
        }

        let to = match (&from, &event) {
            (RemoteLifecycleState::Absent, LifecycleEvent::Create) => {
                RemoteLifecycleState::Creating
            }
            (
                RemoteLifecycleState::Creating,
                LifecycleEvent::CreateSuccess {
                    compliance,
                    evidence,
                    valence,
                },
            ) => RemoteLifecycleState::Active {
                compliance: *compliance,
                evidence: *evidence,
                valence: *valence,
            },
            (
                RemoteLifecycleState::Creating,
                LifecycleEvent::CreateFailure {
                    op,
                    code,
                    retryable,
                },
            ) => RemoteLifecycleState::Failed {
                op: *op,
                code: code.clone(),
                retryable: *retryable,
            },
            (
                RemoteLifecycleState::Active {
                    compliance,
                    evidence,
                    ..
                },
                LifecycleEvent::Read(valence),
            ) => RemoteLifecycleState::Active {
                compliance: *compliance,
                evidence: *evidence,
                valence: *valence,
            },
            (
                RemoteLifecycleState::Active {
                    compliance,
                    evidence,
                    ..
                },
                LifecycleEvent::List(valence),
            ) => RemoteLifecycleState::Active {
                compliance: *compliance,
                evidence: *evidence,
                valence: *valence,
            },
            (
                RemoteLifecycleState::Active {
                    compliance,
                    evidence,
                    ..
                },
                LifecycleEvent::Update(fsm_event),
            ) => match FsmRuntime::new(*compliance, *evidence).transition(
                fsm_event.clone(),
                actor,
                rationale,
            ) {
                Ok(_) => RemoteLifecycleState::Updating,
                Err(e) => RemoteLifecycleState::Rejected {
                    reason: e.to_string(),
                },
            },
            (
                RemoteLifecycleState::Active {
                    compliance: ComplianceState::Archived,
                    ..
                },
                LifecycleEvent::Delete,
            ) => RemoteLifecycleState::Rejected {
                reason: "delete on archived".to_owned(),
            },
            (RemoteLifecycleState::Active { .. }, LifecycleEvent::Delete) => {
                RemoteLifecycleState::Deleting
            }
            (
                RemoteLifecycleState::Updating,
                LifecycleEvent::UpdateSuccess {
                    compliance,
                    evidence,
                    valence,
                },
            ) => RemoteLifecycleState::Active {
                compliance: *compliance,
                evidence: *evidence,
                valence: *valence,
            },
            (
                RemoteLifecycleState::Updating,
                LifecycleEvent::UpdateFailure {
                    op,
                    code,
                    retryable,
                },
            ) => RemoteLifecycleState::Failed {
                op: *op,
                code: code.clone(),
                retryable: *retryable,
            },
            (RemoteLifecycleState::Deleting, LifecycleEvent::DeleteSuccess) => {
                RemoteLifecycleState::Deleted
            }
            (
                RemoteLifecycleState::Deleting,
                LifecycleEvent::DeleteFailure {
                    op,
                    code,
                    retryable,
                },
            ) => RemoteLifecycleState::Failed {
                op: *op,
                code: code.clone(),
                retryable: *retryable,
            },
            (RemoteLifecycleState::Deleted, LifecycleEvent::Delete) => {
                RemoteLifecycleState::Rejected {
                    reason: "delete on deleted".to_owned(),
                }
            }
            (RemoteLifecycleState::Failed { .. }, LifecycleEvent::Retry) => {
                RemoteLifecycleState::Creating
            }
            (RemoteLifecycleState::Failed { .. }, LifecycleEvent::Reset) => {
                RemoteLifecycleState::Absent
            }
            (RemoteLifecycleState::Rejected { .. }, LifecycleEvent::Retry) => {
                RemoteLifecycleState::Creating
            }
            (RemoteLifecycleState::Rejected { .. }, LifecycleEvent::Reset) => {
                RemoteLifecycleState::Absent
            }
            (_, LifecycleEvent::Contradiction(kind)) => {
                RemoteLifecycleState::Incoherent { kind: *kind }
            }
            (s, e) => RemoteLifecycleState::Rejected {
                reason: format!("{e} not permitted from {s}"),
            },
        };

        self.record(&from, &event, &to, actor, rationale);
        &self.history.current_state
    }

    fn record(
        &mut self,
        from: &RemoteLifecycleState,
        event: &LifecycleEvent,
        to: &RemoteLifecycleState,
        actor: &str,
        rationale: &str,
    ) {
        let to_valence = if matches!(
            event,
            LifecycleEvent::Contradiction(
                IncoherenceKind::DigestMismatch | IncoherenceKind::ChainInvalid
            )
        ) {
            Some(Valence::Quarantined)
        } else {
            to.valence()
        };
        let record = LifecycleRecord {
            from_state: from.clone(),
            to_state: to.clone(),
            event: event.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor: actor.to_owned(),
            rationale: rationale.to_owned(),
            from_valence: from.valence(),
            to_valence,
        };
        self.history.current_state = to.clone();
        self.history.transitions.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act(c: ComplianceState, e: EvidenceLevel, v: Valence) -> LifecycleEvent {
        LifecycleEvent::CreateSuccess {
            compliance: c,
            evidence: e,
            valence: v,
        }
    }

    fn rw() -> Gate {
        Gate { read_only: false }
    }

    fn ro() -> Gate {
        Gate { read_only: true }
    }

    fn to_active(c: ComplianceState, e: EvidenceLevel, v: Valence) -> RemoteLifecycleState {
        RemoteLifecycleState::Active {
            compliance: c,
            evidence: e,
            valence: v,
        }
    }

    #[test]
    fn positive_absent_create_to_creating() {
        let mut rt = LifecycleRuntime::new();
        let s = rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        assert_eq!(*s, RemoteLifecycleState::Creating);
    }

    #[test]
    fn positive_creating_to_active() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        let s = rt.apply(
            act(
                ComplianceState::Operational,
                EvidenceLevel::E4AuditPassed,
                Valence::Observed,
            ),
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            to_active(
                ComplianceState::Operational,
                EvidenceLevel::E4AuditPassed,
                Valence::Observed
            )
        );
    }

    #[test]
    fn positive_update_delete_lifecycle() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Draft,
                EvidenceLevel::E1SchemaValid,
                Valence::Inferred,
            ),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(
            LifecycleEvent::Update(FsmEvent::SubmitForReview),
            &rw(),
            "t",
            "",
        );
        assert_eq!(*s, RemoteLifecycleState::Updating);
        let s = rt.apply(
            LifecycleEvent::UpdateSuccess {
                compliance: ComplianceState::UnderReview,
                evidence: EvidenceLevel::E1SchemaValid,
                valence: Valence::Attested,
            },
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            to_active(
                ComplianceState::UnderReview,
                EvidenceLevel::E1SchemaValid,
                Valence::Attested
            )
        );
        let s = rt.apply(LifecycleEvent::Delete, &rw(), "t", "");
        assert_eq!(*s, RemoteLifecycleState::Deleting);
        let s = rt.apply(LifecycleEvent::DeleteSuccess, &rw(), "t", "");
        assert_eq!(*s, RemoteLifecycleState::Deleted);
    }

    #[test]
    fn positive_read_and_list_update_valence() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Approved,
                EvidenceLevel::E3FedrampPassed,
                Valence::Inferred,
            ),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(LifecycleEvent::Read(Valence::Attested), &rw(), "t", "");
        assert_eq!(
            *s,
            to_active(
                ComplianceState::Approved,
                EvidenceLevel::E3FedrampPassed,
                Valence::Attested
            )
        );
        let s = rt.apply(LifecycleEvent::List(Valence::Claimed), &rw(), "t", "");
        assert_eq!(
            *s,
            to_active(
                ComplianceState::Approved,
                EvidenceLevel::E3FedrampPassed,
                Valence::Claimed
            )
        );
    }

    #[test]
    fn negative_create_failure() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        let s = rt.apply(
            LifecycleEvent::CreateFailure {
                op: MethodClass::Create,
                code: "503".to_owned(),
                retryable: true,
            },
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            RemoteLifecycleState::Failed {
                op: MethodClass::Create,
                code: "503".to_owned(),
                retryable: true
            }
        );
    }

    #[test]
    fn negative_update_failure() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Draft,
                EvidenceLevel::E1SchemaValid,
                Valence::Inferred,
            ),
            &rw(),
            "t",
            "",
        );
        rt.apply(
            LifecycleEvent::Update(FsmEvent::SubmitForReview),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(
            LifecycleEvent::UpdateFailure {
                op: MethodClass::Update,
                code: "500".to_owned(),
                retryable: false,
            },
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            RemoteLifecycleState::Failed {
                op: MethodClass::Update,
                code: "500".to_owned(),
                retryable: false
            }
        );
    }

    #[test]
    fn negative_delete_failure() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Operational,
                EvidenceLevel::E4AuditPassed,
                Valence::Observed,
            ),
            &rw(),
            "t",
            "",
        );
        rt.apply(LifecycleEvent::Delete, &rw(), "t", "");
        let s = rt.apply(
            LifecycleEvent::DeleteFailure {
                op: MethodClass::Delete,
                code: "409".to_owned(),
                retryable: false,
            },
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            RemoteLifecycleState::Failed {
                op: MethodClass::Delete,
                code: "409".to_owned(),
                retryable: false
            }
        );
    }

    #[test]
    fn anti_read_only_create_and_delete() {
        let mut rt = LifecycleRuntime::new();
        let _s = rt.apply(LifecycleEvent::Create, &ro(), "t", "");
        assert!(rt.state().to_string().contains("read-only"));
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Operational,
                EvidenceLevel::E4AuditPassed,
                Valence::Observed,
            ),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(LifecycleEvent::Delete, &ro(), "t", "");
        assert!(s.to_string().contains("read-only"));
    }

    #[test]
    fn anti_delete_on_archived_and_deleted() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Archived,
                EvidenceLevel::E5ContinuousMonitoring,
                Valence::Observed,
            ),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(LifecycleEvent::Delete, &rw(), "t", "");
        assert!(s.to_string().contains("delete on archived"));

        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Operational,
                EvidenceLevel::E4AuditPassed,
                Valence::Observed,
            ),
            &rw(),
            "t",
            "",
        );
        rt.apply(LifecycleEvent::Delete, &rw(), "t", "");
        rt.apply(LifecycleEvent::DeleteSuccess, &rw(), "t", "");
        let s = rt.apply(LifecycleEvent::Delete, &rw(), "t", "");
        assert!(s.to_string().contains("delete on deleted"));
    }

    #[test]
    fn anti_guard_rejected_update() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Draft,
                EvidenceLevel::E0Unverified,
                Valence::Inferred,
            ),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(
            LifecycleEvent::Update(FsmEvent::SubmitForReview),
            &rw(),
            "t",
            "",
        );
        assert!(s.to_string().contains("rejected"));
    }

    #[test]
    fn incoherent_partial_crud_from_updating() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Draft,
                EvidenceLevel::E1SchemaValid,
                Valence::Inferred,
            ),
            &rw(),
            "t",
            "",
        );
        rt.apply(
            LifecycleEvent::Update(FsmEvent::SubmitForReview),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(
            LifecycleEvent::Contradiction(IncoherenceKind::PartialCrud),
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            RemoteLifecycleState::Incoherent {
                kind: IncoherenceKind::PartialCrud
            }
        );
    }

    #[test]
    fn incoherent_digest_quarantines_valence_and_fsm() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            act(
                ComplianceState::Operational,
                EvidenceLevel::E4AuditPassed,
                Valence::Observed,
            ),
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(
            LifecycleEvent::Contradiction(IncoherenceKind::DigestMismatch),
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            RemoteLifecycleState::Incoherent {
                kind: IncoherenceKind::DigestMismatch
            }
        );
        let last = rt.history().transitions.last().expect("record");
        assert_eq!(last.to_valence, Some(Valence::Quarantined));

        let s = rt.apply(
            LifecycleEvent::Contradiction(IncoherenceKind::FsmContradiction),
            &rw(),
            "t",
            "",
        );
        assert_eq!(
            *s,
            RemoteLifecycleState::Incoherent {
                kind: IncoherenceKind::FsmContradiction
            }
        );
    }

    #[test]
    fn failed_refuses_mutations_and_reset_or_retry() {
        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            LifecycleEvent::CreateFailure {
                op: MethodClass::Create,
                code: "500".to_owned(),
                retryable: true,
            },
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(LifecycleEvent::Delete, &rw(), "t", "");
        assert!(s.to_string().contains("not permitted"));
        let s = rt.apply(LifecycleEvent::Retry, &rw(), "t", "");
        assert_eq!(*s, RemoteLifecycleState::Creating);

        let mut rt = LifecycleRuntime::new();
        rt.apply(LifecycleEvent::Create, &rw(), "t", "");
        rt.apply(
            LifecycleEvent::CreateFailure {
                op: MethodClass::Create,
                code: "500".to_owned(),
                retryable: true,
            },
            &rw(),
            "t",
            "",
        );
        let s = rt.apply(LifecycleEvent::Reset, &rw(), "t", "");
        assert_eq!(*s, RemoteLifecycleState::Absent);
    }
}
