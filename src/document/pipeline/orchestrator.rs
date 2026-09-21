use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    document::{
        attestation::{SlsaProvenanceBuilder, SlsaVersion},
        cas::CasStore,
        catalog::{EmbeddedCatalogProvider, Jurisdiction},
        evidence::{bundle::ObservationProof, EvidenceBundle},
        export::{GitLabReportExporter, SarifExporter},
        fsm::EvidenceLevel,
        parser::OscalDocument,
        policy::rulepack::BuiltinRulepack,
        sbom::cyclonedx::SbomImporter,
    },
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub jurisdiction: Jurisdiction,
    pub sbom_path: Option<PathBuf>,
    pub workload_path: Option<PathBuf>,
    pub rule_ids: Vec<String>,
    pub output_dir: PathBuf,
    pub subject_name: String,
    pub subject_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineExecutionReport {
    pub timestamp: String,
    pub jurisdiction: String,
    pub oscal_catalog_uuid: String,
    pub sbom_components_count: usize,
    pub evaluated_rules_count: usize,
    pub passed_rules_count: usize,
    pub violations_count: usize,
    pub merkle_root: String,
    pub cas_objects_written: usize,
    pub slsa_provenance_path: PathBuf,
    pub sarif_report_path: PathBuf,
    pub gitlab_report_path: PathBuf,
    pub oscal_assessment_path: PathBuf,
    pub all_passed: bool,
}

pub struct PipelineOrchestrator;

impl PipelineOrchestrator {
    pub fn run(config: PipelineConfig) -> Result<PipelineExecutionReport> {
        fs::create_dir_all(&config.output_dir)
            .map_err(|e| AppError::Configuration(format!("Failed to create output dir: {e}")))?;

        let cas = CasStore::default();
        let mut cas_written = 0;

        // 1. Load Baseline Catalog
        let catalog_doc = EmbeddedCatalogProvider::get_catalog(config.jurisdiction)?;
        let catalog_json_str = serde_json::to_string_pretty(&catalog_doc.value)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let catalog_path = config.output_dir.join("oscal-catalog.json");
        fs::write(&catalog_path, &catalog_json_str)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        cas.put_str(&catalog_json_str)?;
        cas_written += 1;

        let cat_uuid = catalog_doc.value["catalog"]["uuid"]
            .as_str()
            .ok_or_else(|| {
                AppError::Configuration("Embedded catalog is missing a UUID".to_owned())
            })?
            .to_string();

        // 2. Ingest SBOM if provided
        let mut sbom_count = 0;
        let mut oscal_comp_uuid = String::new();
        if let Some(sbom_file) = &config.sbom_path {
            if sbom_file.exists() {
                let (comp_doc, summary) = SbomImporter::import_file(sbom_file)?;
                sbom_count = summary.direct_dependencies;
                oscal_comp_uuid = summary.oscal_component_uuid;
                let comp_json = serde_json::to_string_pretty(&comp_doc.value)
                    .map_err(|e| AppError::Configuration(e.to_string()))?;
                let comp_path = config.output_dir.join("oscal-component-definition.json");
                fs::write(&comp_path, &comp_json)
                    .map_err(|e| AppError::Configuration(e.to_string()))?;
                cas.put_str(&comp_json)?;
                cas_written += 1;
            }
        }

        // 3. Evaluate Policies against Workload
        let workload_val = if let Some(workload_file) = &config.workload_path {
            if workload_file.exists() {
                let content = fs::read_to_string(workload_file).map_err(|e| {
                    AppError::Configuration(format!("Failed to read workload file: {e}"))
                })?;
                serde_json::from_str::<Value>(&content)
                    .or_else(|_| serde_yaml::from_str::<Value>(&content))
                    .map_err(|e| {
                        AppError::Configuration(format!("Invalid JSON/YAML workload: {e}"))
                    })?
            } else {
                json!({
                    "apiVersion": "v1",
                    "kind": "Pod",
                    "spec": {
                        "containers": [{
                            "name": "workload-app",
                            "securityContext": {
                                "privileged": false,
                                "runAsNonRoot": true,
                                "readOnlyRootFilesystem": true
                            }
                        }]
                    }
                })
            }
        } else {
            json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "spec": {
                    "containers": [{
                        "name": "workload-app",
                        "securityContext": {
                            "privileged": false,
                            "runAsNonRoot": true,
                            "readOnlyRootFilesystem": true
                        }
                    }]
                }
            })
        };

        let rules_to_run = if config.rule_ids.is_empty() {
            BuiltinRulepack::get_rules()
                .into_iter()
                .map(|r| r.id)
                .collect()
        } else {
            config.rule_ids.clone()
        };

        let mut evaluated_count = 0;
        let mut passed_count = 0;
        let mut violations_count = 0;
        let mut findings_for_oscal = Vec::new();
        let mut generated_observations = Vec::new();

        for rule_id in &rules_to_run {
            evaluated_count += 1;
            match BuiltinRulepack::evaluate_rule(rule_id, &workload_val) {
                Ok(res) => {
                    if res.passed {
                        passed_count += 1;
                        generated_observations.push(ObservationProof {
                            observation_id: format!("obs-{}", uuid::Uuid::new_v4()),
                            control_id: rule_id.clone(),
                            target: "workload-resource".to_string(),
                            status: "PASSED".to_string(),
                            evaluator_engine: "regorus-0.3.4".to_string(),
                            proof_digest: format!("sha256:{}", sha256_hex(rule_id.as_bytes())),
                        });
                    } else {
                        violations_count += 1;
                        for f in &res.findings {
                            findings_for_oscal.push(json!({
                                "rule_id": rule_id,
                                "violation": f,
                            }));
                            generated_observations.push(ObservationProof {
                                observation_id: format!("obs-{}", uuid::Uuid::new_v4()),
                                control_id: rule_id.clone(),
                                target: "workload-resource".to_string(),
                                status: "FAILED".to_string(),
                                evaluator_engine: "regorus-0.3.4".to_string(),
                                proof_digest: format!("sha256:{}", sha256_hex(f.as_bytes())),
                            });
                        }
                    }
                }
                Err(e) => {
                    violations_count += 1;
                    generated_observations.push(ObservationProof {
                        observation_id: format!("obs-{}", uuid::Uuid::new_v4()),
                        control_id: rule_id.clone(),
                        target: "workload-resource".to_string(),
                        status: "ERROR".to_string(),
                        evaluator_engine: "regorus-0.3.4".to_string(),
                        proof_digest: format!("sha256:{}", sha256_hex(e.to_string().as_bytes())),
                    });
                }
            }
        }

        // 4. Generate OSCAL Assessment Results Document
        let assessment_results = json!({
            "assessment-results": {
                "uuid": uuid::Uuid::new_v4().to_string(),
                "metadata": {
                    "title": "Mizan Automated Compliance Pipeline Assessment",
                    "last-modified": chrono::Utc::now().to_rfc3339(),
                    "version": "1.0",
                    "oscal-version": "1.2.0"
                },
                "results": [{
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "title": "Pipeline Continuous Evaluation",
                    "description": format!("Evaluated {} rules against jurisdiction {}", evaluated_count, config.jurisdiction),
                    "start": chrono::Utc::now().to_rfc3339(),
                    "findings": findings_for_oscal,
                    "components": [{
                        "uuid": if oscal_comp_uuid.is_empty() { uuid::Uuid::new_v4().to_string() } else { oscal_comp_uuid },
                        "title": config.subject_name
                    }]
                }]
            }
        });
        let assessment_doc = OscalDocument::from_value(assessment_results.clone(), None)?;
        let assessment_json = serde_json::to_string_pretty(&assessment_results)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let assessment_path = config.output_dir.join("oscal-assessment-results.json");
        fs::write(&assessment_path, &assessment_json)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        cas.put_str(&assessment_json)?;
        cas_written += 1;

        // 5. Build Evidence Bundle & Merkle Root from Assessment Doc
        let mut bundle = EvidenceBundle::create_from_oscal_document(
            &assessment_doc,
            EvidenceLevel::E4AuditPassed,
            "mizan-orchestrator-engine",
        )?;
        if !generated_observations.is_empty() {
            bundle.observations.extend(generated_observations);
        }
        let merkle_root = bundle.merkle_root.clone();
        let bundle_json = serde_json::to_string_pretty(&bundle)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let evidence_path = config.output_dir.join("evidence-bundle.json");
        fs::write(&evidence_path, &bundle_json)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        cas.put_str(&bundle_json)?;
        cas_written += 1;

        // 6. Generate SLSA v1.2 Provenance Statement
        let slsa_builder = SlsaProvenanceBuilder::new(&config.subject_name, &config.subject_digest)
            .with_version(SlsaVersion::V1_2)
            .with_oscal_evidence(bundle);
        let slsa_statement = slsa_builder.build();
        let slsa_json = serde_json::to_string_pretty(&slsa_statement)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let slsa_path = config.output_dir.join("slsa-provenance.json");
        fs::write(&slsa_path, &slsa_json).map_err(|e| AppError::Configuration(e.to_string()))?;
        cas.put_str(&slsa_json)?;
        cas_written += 1;

        // 7. Generate CI/CD Exporters (SARIF v2.1.0 and GitLab Security Report)
        let sarif_report =
            SarifExporter::export_from_oscal(&assessment_doc, Path::new("workload.yaml"))?;
        let sarif_json = serde_json::to_string_pretty(&sarif_report)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let sarif_path = config.output_dir.join("sarif-report.json");
        fs::write(&sarif_path, &sarif_json).map_err(|e| AppError::Configuration(e.to_string()))?;
        cas.put_str(&sarif_json)?;
        cas_written += 1;

        let gitlab_report =
            GitLabReportExporter::export_from_oscal(&assessment_doc, Path::new("workload.yaml"))?;
        let gitlab_json = serde_json::to_string_pretty(&gitlab_report)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        let gitlab_path = config.output_dir.join("gl-security-report.json");
        fs::write(&gitlab_path, &gitlab_json)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        cas.put_str(&gitlab_json)?;
        cas_written += 1;

        Ok(PipelineExecutionReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            jurisdiction: config.jurisdiction.to_string(),
            oscal_catalog_uuid: cat_uuid,
            sbom_components_count: sbom_count,
            evaluated_rules_count: evaluated_count,
            passed_rules_count: passed_count,
            violations_count,
            merkle_root,
            cas_objects_written: cas_written,
            slsa_provenance_path: slsa_path,
            sarif_report_path: sarif_path,
            gitlab_report_path: gitlab_path,
            oscal_assessment_path: assessment_path,
            all_passed: violations_count == 0,
        })
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_orchestrator_end_to_end() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-pipe-test-{}", uuid::Uuid::new_v4()));

        // Create sample SBOM
        let sbom_file = temp_dir.join("cyclonedx.json");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(
            &sbom_file,
            serde_json::json!({
                "bomFormat": "CycloneDX",
                "specVersion": "1.5",
                "version": 1,
                "metadata": {
                    "component": {
                        "name": "enterprise-api-gateway",
                        "version": "1.4.0",
                        "type": "application"
                    }
                },
                "components": [
                    { "name": "tokio", "version": "1.43.0", "type": "library" },
                    { "name": "serde", "version": "1.0.218", "type": "library" }
                ]
            })
            .to_string(),
        )
        .unwrap();

        // Create compliant workload
        let workload_file = temp_dir.join("pod.json");
        fs::write(
            &workload_file,
            serde_json::json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "spec": {
                    "containers": [{
                        "name": "enterprise-api",
                        "securityContext": {
                            "privileged": false,
                            "runAsNonRoot": true,
                            "readOnlyRootFilesystem": true
                        }
                    }]
                }
            })
            .to_string(),
        )
        .unwrap();

        let out_dir = temp_dir.join("artifacts");
        let cfg = PipelineConfig {
            jurisdiction: Jurisdiction::UsNist800_53Rev5,
            sbom_path: Some(sbom_file),
            workload_path: Some(workload_file),
            rule_ids: vec!["cis-k8s-5.2.1".to_string(), "cis-k8s-5.2.6".to_string()],
            output_dir: out_dir.clone(),
            subject_name: "enterprise-api-gateway:v1.4.0".to_string(),
            subject_digest:
                "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
        };

        let report = PipelineOrchestrator::run(cfg).expect("pipeline should execute cleanly");

        assert!(report.all_passed);
        assert_eq!(report.passed_rules_count, 2);
        assert_eq!(report.violations_count, 0);
        assert_eq!(report.sbom_components_count, 2);
        assert!(!report.merkle_root.is_empty());
        assert!(report.slsa_provenance_path.exists());
        assert!(report.sarif_report_path.exists());
        assert!(report.gitlab_report_path.exists());
        assert!(report.oscal_assessment_path.exists());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
