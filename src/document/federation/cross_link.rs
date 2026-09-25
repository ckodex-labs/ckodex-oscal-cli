use serde::Serialize;
use serde_json::{Map, Value, json};
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

    pub fn generate_poam_from_security_report(
        report_path: &Path,
        format_hint: &str,
        poam_title: &str,
        existing_poam_path: Option<&Path>,
        output_path: Option<&Path>,
    ) -> Result<(OscalDocument, SecurityReportPoamReport)> {
        let content = std::fs::read_to_string(report_path)
            .map_err(|e| crate::error::io_error(report_path, e))?;
        let json_val: Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Configuration(format!("Failed to parse security report JSON: {e}")))?;

        let mut poam_items = Vec::new();
        let mut total_findings_read = 0;
        let mut high_critical_count = 0;
        let detected_format: String;

        let is_sarif = format_hint.eq_ignore_ascii_case("sarif")
            || (format_hint.eq_ignore_ascii_case("auto") && json_val.get("runs").is_some());
        let is_gitlab = format_hint.eq_ignore_ascii_case("gitlab")
            || format_hint.eq_ignore_ascii_case("gitlab-sast")
            || (format_hint.eq_ignore_ascii_case("auto") && json_val.get("vulnerabilities").is_some());

        if is_sarif {
            detected_format = "OASIS SARIF v2.1.0".to_string();
            if let Some(runs) = json_val.get("runs").and_then(Value::as_array) {
                for run in runs {
                    if let Some(results) = run.get("results").and_then(Value::as_array) {
                        for res in results {
                            total_findings_read += 1;
                            let rule_id = res
                                .get("ruleId")
                                .and_then(Value::as_str)
                                .unwrap_or("rule-unknown");
                            let msg = res
                                .get("message")
                                .and_then(|m| m.get("text"))
                                .and_then(Value::as_str)
                                .unwrap_or("Security flaw detected");
                            let level = res
                                .get("level")
                                .and_then(Value::as_str)
                                .unwrap_or("warning");

                            if level.eq_ignore_ascii_case("error") {
                                high_critical_count += 1;
                            }

                            let mut loc_display = String::new();
                            if let Some(locations) = res.get("locations").and_then(Value::as_array) {
                                if let Some(first_loc) = locations.first() {
                                    if let Some(phys) = first_loc.get("physicalLocation") {
                                        let uri = phys
                                            .get("artifactLocation")
                                            .and_then(|a| a.get("uri"))
                                            .and_then(Value::as_str)
                                            .unwrap_or("file");
                                        let line = phys
                                            .get("region")
                                            .and_then(|r| r.get("startLine"))
                                            .and_then(Value::as_u64)
                                            .unwrap_or(1);
                                        loc_display = format!("{uri}:{line}");
                                    }
                                }
                            }

                            let mapped_ctrls = map_text_to_controls(&format!("{rule_id} {msg}"));
                            let poam_item = json!({
                                "uuid": uuid::Uuid::new_v4().to_string(),
                                "title": format!("[{level}] {rule_id}"),
                                "description": if loc_display.is_empty() {
                                    msg.to_string()
                                } else {
                                    format!("{msg} (at {loc_display})")
                                },
                                "related-findings": [rule_id],
                                "related-controls": mapped_ctrls,
                                "milestones": [
                                    {
                                        "uuid": uuid::Uuid::new_v4().to_string(),
                                        "title": format!("Patch flaw {rule_id}"),
                                        "status": "planned"
                                    }
                                ]
                            });
                            poam_items.push(poam_item);
                        }
                    }
                }
            }
        } else if is_gitlab {
            detected_format = "GitLab Security Report".to_string();
            if let Some(vulns) = json_val.get("vulnerabilities").and_then(Value::as_array) {
                for v in vulns {
                    total_findings_read += 1;
                    let id = v.get("id").and_then(Value::as_str).unwrap_or("vuln-unknown");
                    let name = v
                        .get("name")
                        .or_else(|| v.get("message"))
                        .and_then(Value::as_str)
                        .unwrap_or("Vulnerability");
                    let severity = v
                        .get("severity")
                        .and_then(Value::as_str)
                        .unwrap_or("Medium");
                    let desc = v
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("Remediation required");

                    let sev_lower = severity.to_lowercase();
                    if sev_lower == "critical" || sev_lower == "high" {
                        high_critical_count += 1;
                    }

                    let file = v
                        .get("location")
                        .and_then(|l| l.get("file"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let line = v
                        .get("location")
                        .and_then(|l| l.get("start_line"))
                        .and_then(Value::as_u64)
                        .unwrap_or(1);

                    let mapped_ctrls = map_text_to_controls(&format!("{id} {name} {desc}"));
                    let poam_item = json!({
                        "uuid": uuid::Uuid::new_v4().to_string(),
                        "title": format!("[{severity}] {name}"),
                        "description": format!("{desc} (Location: {file}:{line})"),
                        "related-findings": [id],
                        "related-controls": mapped_ctrls,
                        "milestones": [
                            {
                                "uuid": uuid::Uuid::new_v4().to_string(),
                                "title": format!("Remediate vulnerability {id}"),
                                "status": "planned"
                            }
                        ]
                    });
                    poam_items.push(poam_item);
                }
            }
        } else {
            return Err(AppError::Configuration(
                "Unsupported security report format. Expected SARIF or GitLab security report."
                    .to_string(),
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let (poam_doc, poam_items_total) = if let Some(existing_p) = existing_poam_path {
            if existing_p.exists() {
                let mut existing_doc = OscalDocument::from_file(existing_p)?;
                let root_key = DocumentKind::Poam.root_key();
                if let Some(root_obj) = existing_doc.value.get_mut(root_key).and_then(Value::as_object_mut) {
                    if let Some(m) = root_obj.get_mut("metadata").and_then(Value::as_object_mut) {
                        m.insert("last-modified".to_string(), json!(now));
                    }
                    if let Some(items) = root_obj.get_mut("poam-items").and_then(Value::as_array_mut) {
                        items.extend(poam_items.clone());
                        let total = items.len();
                        (existing_doc, total)
                    } else {
                        root_obj.insert("poam-items".to_string(), Value::Array(poam_items.clone()));
                        let total = poam_items.len();
                        (existing_doc, total)
                    }
                } else {
                    return Err(AppError::Configuration(
                        "Existing document does not contain a valid plan-of-action-and-milestones model"
                            .to_string(),
                    ));
                }
            } else {
                return Err(AppError::Configuration(format!(
                    "Existing POA&M path not found: {}",
                    existing_p.display()
                )));
            }
        } else {
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

            let created_doc = OscalDocument::from_value(
                Value::Object(doc_json),
                output_path.map(|p| p.to_path_buf()),
            )?;
            let total = poam_items.len();
            (created_doc, total)
        };

        if let Some(out_p) = output_path {
            if let Some(parent) = out_p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let json_str = serde_json::to_string_pretty(&poam_doc.value)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            std::fs::write(out_p, json_str).map_err(|e| crate::error::io_error(out_p, e))?;
        }

        let report = SecurityReportPoamReport {
            input_format: detected_format,
            total_findings_read,
            poam_items_created: poam_items.len(),
            poam_items_total,
            high_critical_count,
            output_file: output_path.map(|p| p.display().to_string()),
        };

        Ok((poam_doc, report))
    }
}

fn map_text_to_controls(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut ctrls = Vec::new();
    if lower.contains("inject")
        || lower.contains("xss")
        || lower.contains("cwe-79")
        || lower.contains("cwe-89")
        || lower.contains("sqli")
    {
        ctrls.push("si-10".to_string());
        ctrls.push("si-3".to_string());
    }
    if lower.contains("auth")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("cwe-287")
    {
        ctrls.push("ia-2".to_string());
        ctrls.push("ia-5".to_string());
    }
    if lower.contains("crypto")
        || lower.contains("tls")
        || lower.contains("ssl")
        || lower.contains("cipher")
        || lower.contains("cwe-327")
    {
        ctrls.push("sc-8".to_string());
        ctrls.push("sc-13".to_string());
    }
    if lower.contains("secret") || lower.contains("leak") || lower.contains("key") {
        ctrls.push("ia-5".to_string());
        ctrls.push("sc-28".to_string());
    }
    if lower.contains("privilege")
        || lower.contains("root")
        || lower.contains("sudo")
        || lower.contains("cwe-250")
    {
        ctrls.push("ac-6".to_string());
    }
    if ctrls.is_empty() {
        ctrls.push("ra-5".to_string());
        ctrls.push("si-2".to_string());
    }
    ctrls.sort();
    ctrls.dedup();
    ctrls
}

#[derive(Clone, Debug, Serialize)]
pub struct SecurityReportPoamReport {
    pub input_format: String,
    pub total_findings_read: usize,
    pub poam_items_created: usize,
    pub poam_items_total: usize,
    pub high_critical_count: usize,
    pub output_file: Option<String>,
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
        assert!(
            items[0]["title"]
                .as_str()
                .unwrap()
                .contains("Privilege Escalation Allowed")
        );
    }

    #[test]
    fn test_poam_generation_from_sarif_report() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-poam-sarif-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let sarif_path = temp_dir.join("report.sarif");
        let sarif_content = r#"{
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "Semgrep",
                            "version": "1.0.0",
                            "informationUri": "https://semgrep.dev",
                            "rules": []
                        }
                    },
                    "results": [
                        {
                            "ruleId": "CWE-89-SQLI",
                            "level": "error",
                            "message": { "text": "SQL injection detected in query builder" },
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": { "uri": "src/db/query.rs" },
                                        "region": { "startLine": 42, "startColumn": 5 }
                                    }
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;
        std::fs::write(&sarif_path, sarif_content).unwrap();

        let out_poam = temp_dir.join("sarif-poam.json");
        let (poam_doc, report) = ComplianceFederator::generate_poam_from_security_report(
            &sarif_path,
            "auto",
            "SARIF Remediation POA&M",
            None,
            Some(&out_poam),
        )
        .unwrap();

        assert_eq!(poam_doc.kind, DocumentKind::Poam);
        assert_eq!(report.input_format, "OASIS SARIF v2.1.0");
        assert_eq!(report.total_findings_read, 1);
        assert_eq!(report.high_critical_count, 1);
        assert_eq!(report.poam_items_created, 1);
        assert!(out_poam.exists());

        let root = poam_doc.root_object().unwrap();
        let items = root["poam-items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0]["title"].as_str().unwrap().contains("CWE-89-SQLI"));
        let ctrls = items[0]["related-controls"].as_array().unwrap();
        assert!(ctrls.iter().any(|c| c.as_str() == Some("si-10")));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_poam_generation_from_gitlab_report() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-poam-gitlab-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let gitlab_path = temp_dir.join("gl-sast-report.json");
        let gitlab_content = r#"{
            "version": "15.0.0",
            "scan": {
                "scanner": {
                    "id": "semgrep",
                    "name": "Semgrep",
                    "version": "1.0.0",
                    "vendor": { "name": "GitLab" }
                },
                "status": "success",
                "start_time": "2026-08-28T00:00:00Z",
                "end_time": "2026-08-28T00:01:00Z",
                "type": "sast"
            },
            "vulnerabilities": [
                {
                    "id": "c7a8b9-secret-leak",
                    "category": "sast",
                    "name": "Hardcoded Secret Key Found",
                    "description": "High entropy secret detected in configuration",
                    "severity": "Critical",
                    "confidence": "High",
                    "location": {
                        "file": "config/secrets.env",
                        "start_line": 12
                    },
                    "identifiers": []
                }
            ]
        }"#;
        std::fs::write(&gitlab_path, gitlab_content).unwrap();

        let out_poam = temp_dir.join("gitlab-poam.json");
        let (poam_doc, report) = ComplianceFederator::generate_poam_from_security_report(
            &gitlab_path,
            "gitlab",
            "GitLab SAST Remediation POA&M",
            None,
            Some(&out_poam),
        )
        .unwrap();

        assert_eq!(poam_doc.kind, DocumentKind::Poam);
        assert_eq!(report.input_format, "GitLab Security Report");
        assert_eq!(report.total_findings_read, 1);
        assert_eq!(report.high_critical_count, 1);
        assert_eq!(report.poam_items_created, 1);

        let root = poam_doc.root_object().unwrap();
        let items = root["poam-items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0]["title"].as_str().unwrap().contains("Critical"));
        let ctrls = items[0]["related-controls"].as_array().unwrap();
        assert!(ctrls.iter().any(|c| c.as_str() == Some("ia-5") || c.as_str() == Some("sc-28")));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
