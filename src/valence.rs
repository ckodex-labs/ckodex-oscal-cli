//! Canonical Vector State Components: S(e,t) = < P, V, A, C, E, L, tau >

use serde::{Deserialize, Serialize};
use std::fmt;

/// P = Presence dimension
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Presence {
    Empty,
    Present,
    Unknown,
    Redacted,
}

impl fmt::Display for Presence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Present => write!(f, "present"),
            Self::Unknown => write!(f, "unknown"),
            Self::Redacted => write!(f, "redacted"),
        }
    }
}

/// V = Valence dimension (directional effect relative to proposition or invariant)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DirectionalValence {
    Positive,
    Negative,
    Neutral,
    Mixed,
    Unresolved,
}

impl fmt::Display for DirectionalValence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Positive => write!(f, "positive"),
            Self::Negative => write!(f, "negative"),
            Self::Neutral => write!(f, "neutral"),
            Self::Mixed => write!(f, "mixed"),
            Self::Unresolved => write!(f, "unresolved"),
        }
    }
}

/// A = Anti / structural conflict relation
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AntiRelation {
    None,
    Contradicts,
    Attacks,
    Invalidates,
}

impl fmt::Display for AntiRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Contradicts => write!(f, "contradicts"),
            Self::Attacks => write!(f, "attacks"),
            Self::Invalidates => write!(f, "invalidates"),
        }
    }
}

/// C = Coherence (integrity across representations of reality)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Coherence {
    Coherent,
    PartiallyCoherent,
    Decoherent,
    Reconciling,
}

impl fmt::Display for Coherence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coherent => write!(f, "coherent"),
            Self::PartiallyCoherent => write!(f, "partially_coherent"),
            Self::Decoherent => write!(f, "decoherent"),
            Self::Reconciling => write!(f, "reconciling"),
        }
    }
}

/// E = Epistemic evidence class and verification standing
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceClass {
    Claimed,
    Inferred,
    Observed,
    Verified,
    Attested,
    Quarantined,
}

impl fmt::Display for EvidenceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Claimed => write!(f, "claimed"),
            Self::Inferred => write!(f, "inferred"),
            Self::Observed => write!(f, "observed"),
            Self::Verified => write!(f, "verified"),
            Self::Attested => write!(f, "attested"),
            Self::Quarantined => write!(f, "quarantined"),
        }
    }
}

/// S(e,t) = < P, V, A, C, E, L, tau >
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VectorState {
    pub presence: Presence,
    pub valence: DirectionalValence,
    pub anti: AntiRelation,
    pub coherence: Coherence,
    pub evidence: EvidenceClass,
    pub lifecycle: String,
    pub epoch: u64,
}

impl VectorState {
    pub fn new(
        presence: Presence,
        valence: DirectionalValence,
        anti: AntiRelation,
        coherence: Coherence,
        evidence: EvidenceClass,
        lifecycle: impl Into<String>,
        epoch: u64,
    ) -> Self {
        Self {
            presence,
            valence,
            anti,
            coherence,
            evidence,
            lifecycle: lifecycle.into(),
            epoch,
        }
    }

    /// Verifies if a forbidden tuple is present (Rule 16 & Section 15 skill invariants)
    pub fn is_forbidden(&self) -> bool {
        // Anti violations dominate all scores
        if self.anti != AntiRelation::None {
            return true;
        }
        // Active lifecycle requires at least Verified evidence
        if self.lifecycle.to_uppercase() == "ACTIVE"
            && matches!(
                self.evidence,
                EvidenceClass::Claimed | EvidenceClass::Quarantined
            )
        {
            return true;
        }
        // Empty presence cannot be positively evaluated
        if self.presence == Presence::Empty && self.valence == DirectionalValence::Positive {
            return true;
        }
        false
    }

    /// Negative valence or decoherence blocks promotion
    pub fn can_promote(&self) -> bool {
        !self.is_forbidden()
            && self.valence == DirectionalValence::Positive
            && self.coherence == Coherence::Coherent
            && matches!(
                self.evidence,
                EvidenceClass::Verified | EvidenceClass::Attested
            )
    }

    /// Anti-relation or quarantined evidence blocks execution
    pub fn can_execute(&self) -> bool {
        self.anti == AntiRelation::None
            && self.coherence != Coherence::Decoherent
            && self.evidence != EvidenceClass::Quarantined
    }
}

/// Epistemic transport valence for legacy RPC and FSM compatibility
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Valence {
    Observed,
    Inferred,
    Claimed,
    Attested,
    Contradicted,
    Quarantined,
}

impl fmt::Display for Valence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Observed => "observed",
            Self::Inferred => "inferred",
            Self::Claimed => "claimed",
            Self::Attested => "attested",
            Self::Contradicted => "contradicted",
            Self::Quarantined => "quarantined",
        };
        f.write_str(s)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CrudResult<T> {
    pub valence: Valence,
    pub data: T,
}

#[allow(dead_code)]
impl<T> CrudResult<T> {
    pub fn new(valence: Valence, data: T) -> Self {
        Self { valence, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_labels_verified_read() {
        assert_eq!(Valence::Observed.to_string(), "observed");
        let result = CrudResult::new(Valence::Observed, "catalog");
        assert_eq!(result.valence, Valence::Observed);
    }

    #[test]
    fn inferred_labels_graph_analysis() {
        assert_eq!(Valence::Inferred.to_string(), "inferred");
        let result = CrudResult::new(Valence::Inferred, 7_u8);
        assert_eq!(result.valence, Valence::Inferred);
    }

    #[test]
    fn claimed_labels_candidate_trust_state() {
        assert_eq!(Valence::Claimed.to_string(), "claimed");
        let result = CrudResult::new(Valence::Claimed, true);
        assert_eq!(result.valence, Valence::Claimed);
    }

    #[test]
    fn attested_labels_verify_success_or_receipt() {
        assert_eq!(Valence::Attested.to_string(), "attested");
        let result = CrudResult::new(Valence::Attested, 0xABu8);
        assert_eq!(result.valence, Valence::Attested);
    }

    #[test]
    fn contradicted_labels_verify_failure_or_invalid_chain() {
        assert_eq!(Valence::Contradicted.to_string(), "contradicted");
        let result = CrudResult::new(Valence::Contradicted, "chain_valid=false");
        assert_eq!(result.valence, Valence::Contradicted);
    }

    #[test]
    fn quarantined_labels_integrity_failure() {
        assert_eq!(Valence::Quarantined.to_string(), "quarantined");
        let result = CrudResult::new(Valence::Quarantined, ());
        assert_eq!(result.valence, Valence::Quarantined);
    }

    #[test]
    fn crud_result_serializes_and_deserializes() {
        let result = CrudResult::new(Valence::Attested, 42_u8);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("attested"));
        let restored: CrudResult<u8> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, restored);
    }

    #[test]
    fn vector_state_healthy_promotion_and_execution() {
        let state = VectorState::new(
            Presence::Present,
            DirectionalValence::Positive,
            AntiRelation::None,
            Coherence::Coherent,
            EvidenceClass::Verified,
            "ACTIVE",
            1790369200,
        );
        assert!(!state.is_forbidden());
        assert!(state.can_promote());
        assert!(state.can_execute());
        assert_eq!(state.presence.to_string(), "present");
        assert_eq!(state.valence.to_string(), "positive");
        assert_eq!(state.coherence.to_string(), "coherent");
    }

    #[test]
    fn vector_state_anti_relation_dominates_and_forbids() {
        let state = VectorState::new(
            Presence::Present,
            DirectionalValence::Positive,
            AntiRelation::Contradicts,
            Coherence::Coherent,
            EvidenceClass::Verified,
            "ACTIVE",
            1790369200,
        );
        assert!(state.is_forbidden());
        assert!(!state.can_promote());
        assert!(!state.can_execute());
    }

    #[test]
    fn vector_state_decoherence_blocks_promotion_and_execution() {
        let state = VectorState::new(
            Presence::Present,
            DirectionalValence::Positive,
            AntiRelation::None,
            Coherence::Decoherent,
            EvidenceClass::Verified,
            "UPDATING",
            1790369200,
        );
        assert!(!state.can_promote());
        assert!(!state.can_execute());
    }

    #[test]
    fn vector_state_active_with_claimed_evidence_is_forbidden() {
        let state = VectorState::new(
            Presence::Present,
            DirectionalValence::Neutral,
            AntiRelation::None,
            Coherence::Coherent,
            EvidenceClass::Claimed,
            "ACTIVE",
            1790369200,
        );
        assert!(state.is_forbidden());
    }
}
