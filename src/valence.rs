use serde::{Deserialize, Serialize};
use std::fmt;

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
}
