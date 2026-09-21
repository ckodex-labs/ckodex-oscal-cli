use serde::Serialize;
use serde_json::{json, Map, Value};
use std::path::Path;

use crate::{
    document::{parser::OscalDocument, schema::DocumentKind},
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize)]
pub struct FederationChainReport {
    pub root_catalog_title: Option<String>,
    pub profile_title: Option<String>,
    pub ssp_title: Option<String>,
    pub total_controls_traced: usize,
    pub total_findings: usize,
    pub poam_items_created: usize,
}

pub struct ComplianceFederator;

impl ComplianceFederator {
    pub fn auto_generate_poam_from_findings(
        ssp_doc: &OscalDocument,
        assessment_results_doc: &OscalDocument,
        poam_title: &str,
        output_path: Option<&Path>,
    ) -> Result<(OscalDocument, FederationChainReport)> {
        let ssp_root = ssp_doc
            .root_object()
            .ok_or_else(|| AppError::Configuration("Invalid SSP root object".to_string()))?;
        let ssp_title = ssp_root
            .get("metadata")
            .and_then(|m| m.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("System Security Plan");

        let assessment_root = assessment_results_doc.root_object().ok_or_else(|| {
            AppError::Configuration("Invalid Assessment Results root object".to_string())
        })?;

        let mut poam_items = Vec::new();
        let mut total_findings = 0;

        if let Some(results) = assessment_root.get("results").and_then(Value::as_array) {
            for res in results {
                if let Some(findings) = res.get("findings").and_then(Value::as_array) {
                    for f in findings {
                        total_findings += 1;
                        let finding_id = f
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("finding-unknown");
                        let title = f
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or("Security Finding");
                        let desc = f
                            .get("description")
                            .and_then(Value::as_str)
                            .unwrap_or("Remediation required");
                        let related_ctrls = f
                            .get("related-controls")
                            .cloned()
                            .unwrap_or_else(|| json!(["ac-1"]));

                        let poam_item = json!({
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "title": format!("Remediate: {title}"),
                            "description": desc,
                            "related-findings": [finding_id],
                            "related-controls": related_ctrls,
                            "milestones": [
                                {
                                    "uuid": uuid::Uuid::new_v4().to_string(),
                                    "title": "Apply configuration patch",
                                    "status": "planned"
                                }
                            ]
                        });
                        poam_items.push(poam_item);
                    }
                }
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let doc_uuid = uuid::Uuid::new_v4().to_string();

        let mut metadata = Map::new();
        metadata.insert("title".to_string(), json!(poam_title));
        metadata.insert("published".to_string(), json!(now));
        metadata.insert("last-modified".to_string(), json!(now));
        metadata.insert("version".to_string(), json!("1.0.0"));
        metadata.insert("oscal-version".to_string(), json!("1.2.3"));

        let mut root_obj = Map::new();
        root_obj.insert("uuid".to_string(), json!(doc_uuid));
        root_obj.insert("metadata".to_string(), Value::Object(metadata));
        root_obj.insert("poam-items".to_string(), Value::Array(poam_items.clone()));

        let mut doc_json = Map::new();
        doc_json.insert(
            DocumentKind::Poam.root_key().to_string(),
            Value::Object(root_obj),
        );

        let poam_doc = OscalDocument::from_value(
            Value::Object(doc_json),
            output_path.map(|p| p.to_path_buf()),
        )?;

        if let Some(out_p) = output_path {
            if let Some(parent) = out_p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let json_str = serde_json::to_string_pretty(&poam_doc.value)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            std::fs::write(out_p, json_str).map_err(|e| crate::error::io_error(out_p, e))?;
        }

        let report = FederationChainReport {
            root_catalog_title: None,
            profile_title: None,
            ssp_title: Some(ssp_title.to_string()),
            total_controls_traced: 1,
            total_findings,
            poam_items_created: poam_items.len(),
        };

        Ok((poam_doc, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federation_poam_generation_from_findings() {
        let sample_ssp = r#"{
            "system-security-plan": {
                "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                "metadata": {
                    "title": "FedRAMP Moderate Cloud Platform",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                }
            }
        }"#;

        let sample_assessment = r#"{
            "assessment-results": {
                "uuid": "7b788647-767a-4ecb-ba3a-f2b7f719602b",
                "metadata": {
                    "title": "Kubernetes Audit",
                    "published": "2026-08-28T00:00:00Z",
                    "last-modified": "2026-08-28T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "results": [
                    {
                        "uuid": "11111111-767a-4ecb-ba3a-f2b7f719602c",
                        "title": "Live K8s Regorus Run",
                        "start": "2026-08-28T00:00:00Z",
                        "findings": [
                            {
                                "id": "finding-001",
                                "title": "Privilege Escalation Allowed",
                                "description": "Pod allows privilege escalation",
                                "related-controls": ["ac-6"]
                            }
                        ]
                    }
                ]
            }
        }"#;

        let ssp_doc = OscalDocument::from_str(sample_ssp, None).unwrap();
        let assessment_doc = OscalDocument::from_str(sample_assessment, None).unwrap();

        let (poam_doc, report) = ComplianceFederator::auto_generate_poam_from_findings(
            &ssp_doc,
            &assessment_doc,
            "Auto-Remediation POA&M",
            None,
        )
        .unwrap();

        assert_eq!(poam_doc.kind, DocumentKind::Poam);
        assert_eq!(report.total_findings, 1);
        assert_eq!(report.poam_items_created, 1);

        let root = poam_doc.root_object().unwrap();
        let items = root["poam-items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0]["title"]
            .as_str()
            .unwrap()
            .contains("Privilege Escalation Allowed"));
    }
}
