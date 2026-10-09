use serde::{Deserialize, Serialize};
use std::path::Path;

use super::findings::{extract_findings, FindingDisposition};
use crate::{
    document::parser::OscalDocument,
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabSecurityReport {
    pub version: String,
    pub vulnerabilities: Vec<GitLabVulnerability>,
    pub scan: GitLabScanMeta,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabScanMeta {
    pub scanner: GitLabScannerInfo,
    pub status: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(rename = "type")]
    pub scan_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabScannerInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub vendor: GitLabVendor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabVendor {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabVulnerability {
    pub id: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub severity: String,
    pub confidence: String,
    pub location: GitLabLocation,
    pub identifiers: Vec<GitLabIdentifier>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabLocation {
    pub file: String,
    pub start_line: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitLabIdentifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub name: String,
    pub value: String,
    pub url: Option<String>,
}

pub struct GitLabReportExporter;

impl GitLabReportExporter {
    pub fn export_from_oscal(
        doc: &OscalDocument,
        source_path: &Path,
    ) -> Result<GitLabSecurityReport> {
        if doc.root_object().is_none() {
            return Err(AppError::Configuration(
                "Document root is not an object".to_string(),
            ));
        }

        let mut vulnerabilities = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();
        let file_str = source_path.to_string_lossy().to_string();

        // Only unsatisfied, unwaived findings are vulnerabilities. The GitLab
        // schema has no suppression field, so waived findings are omitted here
        // and remain visible in the SARIF report and the assessment results.
        for (idx, f) in extract_findings(doc)
            .into_iter()
            .filter(|f| f.disposition == FindingDisposition::Failed)
            .enumerate()
        {
            let id = f.rule_id;
            vulnerabilities.push(GitLabVulnerability {
                id: format!("mizan-finding-{id}-{idx}"),
                category: "compliance".to_string(),
                name: Some(f.title.unwrap_or_else(|| format!("Rule {id}"))),
                message: Some(f.message.clone()),
                description: Some(f.message),
                severity: "Unknown".to_string(),
                confidence: "Confirmed".to_string(),
                location: GitLabLocation {
                    file: file_str.clone(),
                    start_line: 1,
                },
                identifiers: vec![GitLabIdentifier {
                    id_type: "mizan_rule_id".to_string(),
                    name: format!("Mizan rule {id}"),
                    value: id,
                    url: None,
                }],
            });
        }

        Ok(GitLabSecurityReport {
            version: "15.0.0".to_string(),
            vulnerabilities,
            scan: GitLabScanMeta {
                scanner: GitLabScannerInfo {
                    id: "mizan-compliance-scanner".to_string(),
                    name: "Mizan Compliance Engine".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    vendor: GitLabVendor {
                        name: "Mizan Core".to_string(),
                    },
                },
                status: "success".to_string(),
                start_time: now.clone(),
                end_time: now,
                scan_type: "sast".to_string(),
            },
        })
    }

    pub fn save_to_file(report: &GitLabSecurityReport, output_path: &Path) -> Result<()> {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Configuration(format!("Failed to create GitLab report dir: {e}"))
            })?;
        }
        let json_str = serde_json::to_string_pretty(report)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        std::fs::write(output_path, json_str)
            .map_err(|e| AppError::Configuration(format!("Failed to write GitLab report: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_report_export() {
        let doc = OscalDocument::from_value(
            serde_json::json!({
                "catalog": {
                    "uuid": "test-cat-123",
                    "metadata": { "title": "Test Catalog", "version": "1.0" },
                    "controls": [
                        {
                            "id": "ac-1",
                            "title": "Access Control Policy",
                            "description": "Establish access control policy"
                        }
                    ]
                }
            }),
            None,
        )
        .unwrap();
        let report =
            GitLabReportExporter::export_from_oscal(&doc, Path::new("catalog.json")).unwrap();

        assert_eq!(report.version, "15.0.0");
        // Catalog controls are definitions, not vulnerabilities.
        assert_eq!(report.vulnerabilities.len(), 0);
        assert_eq!(report.scan.scanner.id, "mizan-compliance-scanner");
    }

    #[test]
    fn test_gitlab_report_failed_findings_only() {
        let doc = OscalDocument::from_value(
            serde_json::json!({
                "assessment-results": {
                    "uuid": "ar",
                    "metadata": { "title": "t" },
                    "results": [{ "findings": [
                        { "rule_id": "k8s-no-root", "status": "FAILED", "violation": "runs as root" },
                        { "rule_id": "k8s-limits", "status": "WAIVED", "violation": "no limits" }
                    ]}]
                }
            }),
            None,
        )
        .unwrap();
        let report = GitLabReportExporter::export_from_oscal(&doc, Path::new("ar.json")).unwrap();
        assert_eq!(report.vulnerabilities.len(), 1);
        assert_eq!(report.vulnerabilities[0].identifiers[0].value, "k8s-no-root");
    }
}
