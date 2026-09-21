use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::path::Path;

use crate::{
    document::evidence::EvidenceBundle,
    error::{io_error, AppError, Result},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SlsaVersion {
    /// SLSA v1.2 Provenance Specification
    V1_2,
    /// SLSA v1.0 Provenance Specification (Backward Compatible)
    V1_0,
}

impl SlsaVersion {
    pub fn predicate_type(&self) -> &'static str {
        match self {
            Self::V1_2 => "https://slsa.dev/provenance/v1.2",
            Self::V1_0 => "https://slsa.dev/provenance/v1",
        }
    }

    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "v1.2" | "1.2" | "v12" => Some(Self::V1_2),
            "v1.0" | "1.0" | "v1" | "v10" => Some(Self::V1_0),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlsaVerificationReport {
    pub is_valid: bool,
    pub slsa_version: SlsaVersion,
    pub subject_name: String,
    pub subject_sha256: String,
    pub builder_id: String,
    pub has_oscal_evidence: bool,
    pub merkle_root: Option<String>,
}

pub struct SlsaProvenanceBuilder {
    version: SlsaVersion,
    subject_name: String,
    subject_digest: String,
    builder_id: String,
    invocation_id: String,
    external_parameters: Map<String, Value>,
    resolved_dependencies: Vec<Value>,
    oscal_evidence_bundle: Option<EvidenceBundle>,
}

impl SlsaProvenanceBuilder {
    pub fn new(subject_name: &str, subject_sha256: &str) -> Self {
        let clean_digest = subject_sha256
            .strip_prefix("sha256:")
            .unwrap_or(subject_sha256)
            .to_string();

        Self {
            version: SlsaVersion::V1_2,
            subject_name: subject_name.to_string(),
            subject_digest: clean_digest,
            builder_id: "https://mizan.dev/compliance-kernel/v0.1.0".to_string(),
            invocation_id: format!("mizan-run-{}", uuid::Uuid::new_v4()),
            external_parameters: Map::new(),
            resolved_dependencies: Vec::new(),
            oscal_evidence_bundle: None,
        }
    }

    pub fn with_version(mut self, version: SlsaVersion) -> Self {
        self.version = version;
        self
    }

    pub fn with_builder_id(mut self, builder_id: &str) -> Self {
        self.builder_id = builder_id.to_string();
        self
    }

    pub fn add_external_parameter(mut self, key: &str, value: Value) -> Self {
        self.external_parameters.insert(key.to_string(), value);
        self
    }

    pub fn add_dependency(mut self, uri: &str, digest_sha256: &str) -> Self {
        let clean_digest = digest_sha256
            .strip_prefix("sha256:")
            .unwrap_or(digest_sha256);
        self.resolved_dependencies.push(json!({
            "uri": uri,
            "digest": {
                "sha256": clean_digest
            }
        }));
        self
    }

    pub fn with_oscal_evidence(mut self, bundle: EvidenceBundle) -> Self {
        self.oscal_evidence_bundle = Some(bundle);
        self
    }

    pub fn build(self) -> Value {
        let now = chrono::Utc::now().to_rfc3339();

        let mut byproducts = Vec::new();
        let mut oscal_ext = json!({});

        if let Some(bundle) = &self.oscal_evidence_bundle {
            byproducts.push(json!({
                "name": format!("oscal-evidence-{}", bundle.bundle_id),
                "digest": {
                    "sha256": bundle.document_digest.strip_prefix("sha256:").unwrap_or(&bundle.document_digest)
                }
            }));

            oscal_ext = json!({
                "bundle_id": bundle.bundle_id,
                "evidence_level": bundle.evidence_level,
                "document_uuid": bundle.document_uuid,
                "merkle_root": bundle.merkle_root,
                "observations_count": bundle.observations.len()
            });
        }

        let mut predicate = json!({
            "buildDefinition": {
                "buildType": "https://mizan.dev/attestation/v1",
                "externalParameters": self.external_parameters,
                "internalParameters": {
                    "oscal_compliance_extension": oscal_ext
                },
                "resolvedDependencies": self.resolved_dependencies
            },
            "runDetails": {
                "builder": {
                    "id": self.builder_id
                },
                "metadata": {
                    "invocationId": self.invocation_id,
                    "startedOn": now,
                    "finishedOn": now
                },
                "byproducts": byproducts
            }
        });

        // SLSA v1.2 vs v1.0 backward compatibility adjustment
        if self.version == SlsaVersion::V1_0 {
            predicate["builder"] = json!({ "id": self.builder_id });
        }

        json!({
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [
                {
                    "name": self.subject_name,
                    "digest": {
                        "sha256": self.subject_digest
                    }
                }
            ],
            "predicateType": self.version.predicate_type(),
            "predicate": predicate
        })
    }

    pub fn verify(statement: &Value) -> Result<SlsaVerificationReport> {
        let stmt_type = statement.get("_type").and_then(Value::as_str);
        if stmt_type != Some("https://in-toto.io/Statement/v1") {
            return Err(AppError::Configuration(
                "Invalid in-toto statement header".to_string(),
            ));
        }

        let pred_type = statement
            .get("predicateType")
            .and_then(Value::as_str)
            .unwrap_or("");
        let slsa_version = if pred_type == SlsaVersion::V1_2.predicate_type() {
            SlsaVersion::V1_2
        } else if pred_type == SlsaVersion::V1_0.predicate_type() {
            SlsaVersion::V1_0
        } else {
            return Err(AppError::Configuration(format!(
                "Unsupported SLSA predicate type: {pred_type}"
            )));
        };

        let subjects = statement
            .get("subject")
            .and_then(Value::as_array)
            .ok_or_else(|| AppError::Configuration("Missing subject array".to_string()))?;
        let first_subject = subjects
            .first()
            .ok_or_else(|| AppError::Configuration("Empty subject array".to_string()))?;

        let subject_name = first_subject
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let subject_sha256 = first_subject
            .get("digest")
            .and_then(|d| d.get("sha256"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let predicate = statement
            .get("predicate")
            .ok_or_else(|| AppError::Configuration("Missing predicate block".to_string()))?;

        let builder_id = predicate
            .get("runDetails")
            .and_then(|r| r.get("builder"))
            .and_then(|b| b.get("id"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let oscal_ext = predicate
            .get("buildDefinition")
            .and_then(|b| b.get("internalParameters"))
            .and_then(|i| i.get("oscal_compliance_extension"));

        let has_oscal_evidence = oscal_ext.is_some();
        let merkle_root = oscal_ext
            .and_then(|o| o.get("merkle_root"))
            .and_then(Value::as_str)
            .map(str::to_string);

        Ok(SlsaVerificationReport {
            is_valid: true,
            slsa_version,
            subject_name,
            subject_sha256,
            builder_id,
            has_oscal_evidence,
            merkle_root,
        })
    }

    pub fn save_to_file(statement: &Value, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json_str = serde_json::to_string_pretty(statement)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        std::fs::write(path, json_str).map_err(|e| io_error(path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{fsm::EvidenceLevel, parser::OscalDocument};

    #[test]
    fn test_slsa_v1_2_and_v1_0_generation_and_verification() {
        let sample_oscal = r#"{
            "catalog": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "SLSA Provenance Catalog",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "controls": [
                    { "id": "ac-1", "title": "Access Control" }
                ]
            }
        }"#;
        let doc = OscalDocument::from_str(sample_oscal, None).unwrap();
        let bundle = EvidenceBundle::create_from_oscal_document(
            &doc,
            EvidenceLevel::E4AuditPassed,
            "mizan-kernel",
        )
        .unwrap();

        // 1. Build SLSA v1.2
        let slsa12_stmt =
            SlsaProvenanceBuilder::new("mizan-container:latest", "sha256:1122334455667788")
                .with_version(SlsaVersion::V1_2)
                .add_external_parameter("git_commit", json!("a1b2c3d4e5"))
                .add_dependency("docker.io/library/alpine:latest", "sha256:aabbccddeeff")
                .with_oscal_evidence(bundle)
                .build();

        assert_eq!(
            slsa12_stmt["predicateType"],
            "https://slsa.dev/provenance/v1.2"
        );

        let rep12 =
            SlsaProvenanceBuilder::verify(&slsa12_stmt).expect("verification should succeed");
        assert!(rep12.is_valid);
        assert_eq!(rep12.slsa_version, SlsaVersion::V1_2);
        assert!(rep12.has_oscal_evidence);
        assert!(rep12.merkle_root.is_some());
    }
}
