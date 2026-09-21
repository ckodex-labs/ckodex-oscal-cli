use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

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
        let root = doc
            .root_object()
            .ok_or_else(|| AppError::Configuration("Document root is not an object".to_string()))?;

        let mut vulnerabilities = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();
        let file_str = source_path.to_string_lossy().to_string();

        if let Some(controls) = root.get("controls").and_then(Value::as_array) {
            for (idx, ctrl) in controls.iter().enumerate() {
                let id = ctrl
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::Configuration("Control missing 'id'".to_owned()))?;
                let title = ctrl.get("title").and_then(Value::as_str);
                let desc = ctrl.get("description").and_then(Value::as_str).or(title);

                vulnerabilities.push(GitLabVulnerability {
                    id: format!("mizan-vuln-{id}-{idx}"),
                    category: "compliance".to_string(),
                    name: title.map(|t| format!("NIST Control {id}: {t}")),
                    message: Some(format!("Compliance verification item for {id}")),
                    description: desc.map(String::from),
                    severity: "Medium".to_string(),
                    confidence: "Confirmed".to_string(),
                    location: GitLabLocation {
                        file: file_str.clone(),
                        start_line: 1,
                    },
                    identifiers: vec![
                        GitLabIdentifier {
                            id_type: "oscal_control_id".to_string(),
                            name: format!("OSCAL Control {id}"),
                            value: id.to_string(),
                            url: Some(format!("https://csrc.nist.gov/projects/cprt/catalog#/cprt/framework/version/SP_800_53_5_1_0/home?element={id}")),
                        }
                    ],
                });
            }
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
        assert_eq!(report.vulnerabilities.len(), 1);
        assert_eq!(report.vulnerabilities[0].identifiers[0].value, "ac-1");
        assert_eq!(report.scan.scanner.id, "mizan-compliance-scanner");
    }
}
