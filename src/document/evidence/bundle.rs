use ring::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    document::{
        cas::CasStore,
        evidence::merkle::compute_merkle_root,
        fsm::state::EvidenceLevel,
        io_safe::{MAX_OSCAL_FILE_BYTES, atomic_write_bytes, read_bounded_utf8_file},
        parser::OscalDocument,
    },
    error::{AppError, Result},
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
    #[serde(default)]
    pub public_key_hex: String,
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
                            let fid = f
                                .get("uuid")
                                .and_then(Value::as_str)
                                .or_else(|| f.get("id").and_then(Value::as_str))
                                .unwrap_or("finding");
                            let ctrl = f
                                .get("target")
                                .and_then(|t| t.get("target-id"))
                                .and_then(Value::as_str)
                                .or_else(|| {
                                    f.get("related-controls")
                                        .and_then(Value::as_array)
                                        .and_then(|a| a.first())
                                        .and_then(Value::as_str)
                                })
                                .or_else(|| {
                                    f.get("props").and_then(Value::as_array).and_then(|props| {
                                        props.iter().find_map(|p| {
                                            if p.get("name").and_then(Value::as_str)
                                                == Some("rule-id")
                                            {
                                                p.get("value").and_then(Value::as_str)
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                })
                                .unwrap_or("ac-1");
                            let status = f
                                .get("target")
                                .and_then(|t| t.get("status"))
                                .and_then(|s| s.get("state"))
                                .and_then(Value::as_str)
                                .or_else(|| {
                                    f.get("props").and_then(Value::as_array).and_then(|props| {
                                        props.iter().find_map(|p| {
                                            if p.get("name").and_then(Value::as_str)
                                                == Some("status")
                                            {
                                                p.get("value").and_then(Value::as_str)
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                })
                                .or_else(|| f.get("status").and_then(Value::as_str))
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

    /// Recomputes `merkle_root` from the current `observations` leaf digests.
    pub fn recompute_merkle_root(&mut self) {
        let leaf_digests: Vec<String> = self
            .observations
            .iter()
            .map(|o| o.proof_digest.clone())
            .collect();
        self.merkle_root = compute_merkle_root(&leaf_digests);
    }

    fn canonical_signing_payload(&self, signer_identity: &str) -> String {
        format!(
            "mizan-evidence-v1:{}:{}:{}:{}",
            self.bundle_id, self.document_digest, self.merkle_root, signer_identity
        )
    }

    /// Signs the evidence bundle with an explicit 32-byte Ed25519 private seed (RFC 8032).
    pub fn sign_with_seed(&mut self, signer_identity: &str, seed: &[u8; 32]) -> Result<()> {
        let key_pair = Ed25519KeyPair::from_seed_unchecked(seed).map_err(|_| {
            AppError::Configuration("Failed to initialize Ed25519 keypair from seed".to_string())
        })?;
        let payload = self.canonical_signing_payload(signer_identity);
        let sig = key_pair.sign(payload.as_bytes());
        let pk_hex = hex::encode(key_pair.public_key().as_ref());
        let sig_hex = hex::encode(sig.as_ref());

        self.signature = Some(SignatureInfo {
            key_id: format!("urn:mizan:ed25519:{signer_identity}:{}", &pk_hex[..16]),
            algorithm: "Ed25519".to_string(),
            signature_hex: sig_hex,
            public_key_hex: pk_hex,
            signed_by: signer_identity.to_string(),
        });
        Ok(())
    }

    /// Signs the evidence bundle using `MIZAN_SIGNING_KEY_HEX` (64-char hex Ed25519 seed)
    /// when present in the environment, or an identity-derived Ed25519 keypair for local attestation.
    pub fn sign(&mut self, signer_identity: &str) {
        let seed = if let Ok(hex_seed) = std::env::var("MIZAN_SIGNING_KEY_HEX")
            && let Ok(bytes) = hex::decode(hex_seed.trim())
            && let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice())
        {
            arr
        } else {
            let mut hasher = Sha256::new();
            hasher.update(b"urn:mizan:ed25519:identity-seed:v1:");
            hasher.update(signer_identity.as_bytes());
            let digest = hasher.finalize();
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&digest);
            arr
        };

        let _ = self.sign_with_seed(signer_identity, &seed);
    }

    pub fn verify_with_public_key(
        &self,
        expected_pub_key_hex: Option<&str>,
    ) -> EvidenceVerificationReport {
        let leaf_digests: Vec<String> = self
            .observations
            .iter()
            .map(|o| o.proof_digest.clone())
            .collect();
        let expected_merkle = compute_merkle_root(&leaf_digests);

        let merkle_valid = expected_merkle == self.merkle_root;
        let pinned_key = expected_pub_key_hex
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let sig_valid = if let Some(sig) = &self.signature {
            let key_hex_to_use = pinned_key.unwrap_or(&sig.public_key_hex);
            let key_matches_pinned = pinned_key.is_none_or(|pinned| {
                sig.public_key_hex.is_empty() || sig.public_key_hex.eq_ignore_ascii_case(pinned)
            });
            if key_matches_pinned
                && sig.algorithm == "Ed25519"
                && let Ok(pk_bytes) = hex::decode(key_hex_to_use)
                && let Ok(sig_bytes) = hex::decode(&sig.signature_hex)
                && pk_bytes.len() == 32
                && sig_bytes.len() == 64
            {
                let payload = self.canonical_signing_payload(&sig.signed_by);
                let public_key = UnparsedPublicKey::new(&ED25519, pk_bytes);
                public_key.verify(payload.as_bytes(), &sig_bytes).is_ok()
            } else {
                false
            }
        } else {
            false
        };

        let is_valid = if pinned_key.is_some() {
            merkle_valid && sig_valid
        } else {
            merkle_valid && (self.signature.is_none() || sig_valid)
        };

        EvidenceVerificationReport {
            bundle_id: self.bundle_id.clone(),
            evidence_level: self.evidence_level,
            document_uuid: self.document_uuid.clone(),
            observations_count: self.observations.len(),
            is_valid,
            merkle_root: self.merkle_root.clone(),
            signature_verified: sig_valid,
        }
    }

    pub fn verify_strict(
        &self,
        expected_pub_key_hex: Option<&str>,
    ) -> Result<EvidenceVerificationReport> {
        let report = self.verify_with_public_key(expected_pub_key_hex);
        if !report.is_valid {
            return Err(AppError::Incoherent {
                kind: crate::error::IncoherenceKind::EvidenceConflict,
                detail: "Evidence bundle cryptographic integrity or signature verification failed"
                    .to_string(),
            });
        }
        Ok(report)
    }

    pub fn verify(&self) -> EvidenceVerificationReport {
        let env_key = std::env::var("MIZAN_VERIFY_PUBLIC_KEY_HEX")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self.verify_with_public_key(env_key.as_deref())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        atomic_write_bytes(path, json_str.as_bytes())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = read_bounded_utf8_file(path, MAX_OSCAL_FILE_BYTES)?;
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

        // Unsigned bundle fails when a pinned public key is required
        assert!(
            bundle
                .verify_strict(Some("00".repeat(32).as_str()))
                .is_err()
        );

        bundle.sign("compliance-officer-01");
        let rep = bundle.verify();

        assert!(rep.is_valid);
        assert!(rep.signature_verified);
        assert_eq!(rep.observations_count, 2);

        // Pinned public key verification succeeds with matching key and fails with wrong key
        let actual_pk = bundle.signature.as_ref().unwrap().public_key_hex.clone();
        assert!(bundle.verify_strict(Some(&actual_pk)).is_ok());
        assert!(bundle.verify_strict(Some(&"ff".repeat(32))).is_err());

        // Adversarial anti-test: tampering with signed_by or signature_hex invalidates Ed25519 verification
        let mut forged = bundle.clone();
        forged.signature.as_mut().unwrap().signed_by = "attacker-identity".to_string();
        let forged_rep = forged.verify();
        assert!(!forged_rep.signature_verified);
        assert!(!forged_rep.is_valid);

        // Adversarial anti-test: unkeyed SHA-256 digest masquerading as signature is rejected
        let mut fake_hash_sig = bundle.clone();
        fake_hash_sig.signature.as_mut().unwrap().signature_hex =
            CasStore::compute_digest(b"fake-unkeyed-digest");
        let fake_rep = fake_hash_sig.verify();
        assert!(!fake_rep.signature_verified);
        assert!(!fake_rep.is_valid);
    }

    #[test]
    fn test_evidence_bundle_oscal_v123_assessment_results_findings() {
        let sample_ar = r##"{
            "assessment-results": {
                "uuid": "ar-1111",
                "metadata": {
                    "title": "OSCAL v1.2.3 Assessment Results",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "import-ap": { "href": "#ap-1" },
                "results": [{
                    "uuid": "res-1",
                    "title": "Run 1",
                    "description": "Continuous scan",
                    "start": "2026-08-28T00:00:00Z",
                    "findings": [{
                        "uuid": "finding-uuid-001",
                        "title": "Violation for cis-k8s-5.2.1",
                        "description": "Privileged container detected",
                        "target": {
                            "type": "statement-id",
                            "target-id": "cis-k8s-5.2.1",
                            "status": {
                                "state": "not-satisfied",
                                "reason": "fail"
                            }
                        }
                    }]
                }]
            }
        }"##;
        let doc = OscalDocument::from_str(sample_ar, None).unwrap();
        let bundle = EvidenceBundle::create_from_oscal_document(
            &doc,
            EvidenceLevel::E1SchemaValid,
            "mizan-test",
        )
        .unwrap();
        assert_eq!(bundle.observations.len(), 1);
        assert_eq!(bundle.observations[0].observation_id, "finding-uuid-001");
        assert_eq!(bundle.observations[0].control_id, "cis-k8s-5.2.1");
        assert_eq!(bundle.observations[0].status, "not-satisfied");
    }
}
