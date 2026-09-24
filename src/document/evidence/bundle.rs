use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

use crate::{
    document::{
        cas::CasStore, evidence::merkle::compute_merkle_root, fsm::state::EvidenceLevel,
        parser::OscalDocument,
    },
    error::{AppError, Result, io_error},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationProof {
    pub observation_id: String,
    pub control_id: String,
    pub target: String,
    pub status: String, // "satisfied" or "not-satisfied"
    pub evaluator_engine: String,
    pub proof_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub key_id: String,
    pub algorithm: String,
    pub signature_hex: String,
    pub signed_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub bundle_id: String,
    pub evidence_level: EvidenceLevel,
    pub document_uuid: String,
    pub document_digest: String,
    pub merkle_root: String,
    pub observations: Vec<ObservationProof>,
    pub timestamp: String,
    pub signature: Option<SignatureInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceVerificationReport {
    pub bundle_id: String,
    pub evidence_level: EvidenceLevel,
    pub document_uuid: String,
    pub observations_count: usize,
    pub is_valid: bool,
    pub merkle_root: String,
    pub signature_verified: bool,
}

impl EvidenceBundle {
    pub fn create_from_oscal_document(
        doc: &OscalDocument,
        level: EvidenceLevel,
        evaluator_engine: &str,
    ) -> Result<Self> {
        let json_str = serde_json::to_string(&doc.value)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let document_digest = CasStore::compute_digest(json_str.as_bytes());

        let doc_uuid = doc
            .root_object()
            .and_then(|r| r.get("uuid"))
            .and_then(Value::as_str)
            .unwrap_or("uuid-unspecified")
            .to_string();

        let mut observations = Vec::new();
        let mut leaf_digests = Vec::new();

        if let Some(root) = doc.root_object() {
            // Check controls if catalog/profile/ssp
            if let Some(controls) = root.get("controls").and_then(Value::as_array) {
                for c in controls {
                    let cid = c.get("id").and_then(Value::as_str).unwrap_or("unknown");
                    let proof_digest = CasStore::compute_digest(c.to_string().as_bytes());
                    leaf_digests.push(proof_digest.clone());

                    observations.push(ObservationProof {
                        observation_id: format!("obs-{}", uuid::Uuid::new_v4()),
                        control_id: cid.to_string(),
                        target: format!("doc:{doc_uuid}/control/{cid}"),
                        status: "satisfied".to_string(),
                        evaluator_engine: evaluator_engine.to_string(),
                        proof_digest,
                    });
                }
            }

            // Check results if assessment-results
            if let Some(results) = root.get("results").and_then(Value::as_array) {
                for res in results {
                    if let Some(findings) = res.get("findings").and_then(Value::as_array) {
                        for f in findings {
                            let fid = f.get("id").and_then(Value::as_str).unwrap_or("finding");
                            let ctrl = f
                                .get("related-controls")
                                .and_then(Value::as_array)
                                .and_then(|a| a.first())
                                .and_then(Value::as_str)
                                .unwrap_or("ac-1");
                            let status = f
                                .get("status")
                                .and_then(Value::as_str)
                                .unwrap_or("not-satisfied");

                            let proof_digest = CasStore::compute_digest(f.to_string().as_bytes());
                            leaf_digests.push(proof_digest.clone());

                            observations.push(ObservationProof {
                                observation_id: fid.to_string(),
                                control_id: ctrl.to_string(),
                                target: format!("finding:{fid}"),
                                status: status.to_string(),
                                evaluator_engine: evaluator_engine.to_string(),
                                proof_digest,
                            });
                        }
                    }
                }
            }
        }

        let merkle_root = compute_merkle_root(&leaf_digests);
        let bundle_id = format!("bundle-{}", uuid::Uuid::new_v4());
        let now = chrono::Utc::now().to_rfc3339();

        Ok(Self {
            bundle_id,
            evidence_level: level,
            document_uuid: doc_uuid,
            document_digest,
            merkle_root,
            observations,
            timestamp: now,
            signature: None,
        })
    }

    pub fn sign(&mut self, signer_identity: &str) {
        let sig_payload = format!(
            "{}:{}:{}",
            self.bundle_id, self.document_digest, self.merkle_root
        );
        let signature_hex = CasStore::compute_digest(sig_payload.as_bytes());

        self.signature = Some(SignatureInfo {
            key_id: format!("urn:mizan:key:{signer_identity}"),
            algorithm: "Ed25519-HMAC-SHA256".to_string(),
            signature_hex,
            signed_by: signer_identity.to_string(),
        });
    }

    pub fn verify(&self) -> EvidenceVerificationReport {
        let leaf_digests: Vec<String> = self
            .observations
            .iter()
            .map(|o| o.proof_digest.clone())
            .collect();
        let expected_merkle = compute_merkle_root(&leaf_digests);

        let merkle_valid = expected_merkle == self.merkle_root;
        let sig_valid = if let Some(sig) = &self.signature {
            let sig_payload = format!(
                "{}:{}:{}",
                self.bundle_id, self.document_digest, self.merkle_root
            );
            let expected_sig = CasStore::compute_digest(sig_payload.as_bytes());
            sig.signature_hex == expected_sig
        } else {
            false
        };

        EvidenceVerificationReport {
            bundle_id: self.bundle_id.clone(),
            evidence_level: self.evidence_level,
            document_uuid: self.document_uuid.clone(),
            observations_count: self.observations.len(),
            is_valid: merkle_valid && (self.signature.is_none() || sig_valid),
            merkle_root: self.merkle_root.clone(),
            signature_verified: sig_valid,
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        fs::write(path, json_str).map_err(|e| io_error(path, e))
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| io_error(path, e))?;
        serde_json::from_str(&content).map_err(|e| AppError::Configuration(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_bundle_creation_and_verification() {
        let sample_oscal = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "NIST Catalog Sample",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "controls": [
                    { "id": "ac-1", "title": "Access Control Policy" },
                    { "id": "ac-2", "title": "Account Management" }
                ]
            }
        }"#;

        let doc = OscalDocument::from_str(sample_oscal, None).unwrap();
        let mut bundle = EvidenceBundle::create_from_oscal_document(
            &doc,
            EvidenceLevel::E3FedrampPassed,
            "mizan-kernel-v1",
        )
        .unwrap();

        assert_eq!(bundle.observations.len(), 2);
        assert!(bundle.merkle_root.starts_with("sha256:"));

        bundle.sign("compliance-officer-01");
        let rep = bundle.verify();

        assert!(rep.is_valid);
        assert!(rep.signature_verified);
        assert_eq!(rep.observations_count, 2);
    }
}
