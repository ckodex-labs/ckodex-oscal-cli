use serde::{Deserialize, Serialize};
use std::path::Path;

use super::findings::{FindingDisposition, collect_catalog_controls, extract_findings};
use crate::{
    document::parser::OscalDocument,
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifReport {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifDriver {
    pub name: String,
    pub version: String,
    #[serde(rename = "informationUri")]
    pub information_uri: String,
    pub rules: Vec<SarifRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    #[serde(rename = "shortDescription")]
    pub short_description: SarifMessage,
    #[serde(rename = "fullDescription", skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
    #[serde(rename = "defaultConfiguration")]
    pub default_configuration: SarifConfiguration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifConfiguration {
    pub level: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifResult {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    /// SARIF 2.1.0 `kind`: `fail`, `pass`, or `review`.
    #[serde(default = "default_kind")]
    pub kind: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressions: Vec<SarifSuppression>,
}

fn default_kind() -> String {
    "fail".to_string()
}

/// A SARIF suppression. Used for waived findings: the result stays a failure,
/// the accepted derogation is recorded alongside it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifSuppression {
    pub kind: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justification: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub artifact_location: SarifArtifactLocation,
    pub region: Option<SarifRegion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    pub uri: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SarifRegion {
    #[serde(rename = "startLine")]
    pub start_line: usize,
    #[serde(rename = "startColumn")]
    pub start_column: usize,
}

pub struct SarifExporter;

impl SarifExporter {
    pub fn export_from_oscal(doc: &OscalDocument, source_path: &Path) -> Result<SarifReport> {
        if doc.root_object().is_none() {
            return Err(AppError::Configuration(
                "Document root is not an object".to_string(),
            ));
        }

        let doc_uri = source_path.to_string_lossy().to_string();
        let rule_key = |id: &str| format!("OSCAL-{}", id.to_uppercase());

        // Rule definitions: catalog controls (if any), then any rule id that a
        // finding references but no control defines.
        let mut rules: Vec<SarifRule> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (id, title) in collect_catalog_controls(doc) {
            if seen.insert(rule_key(&id)) {
                rules.push(SarifRule {
                    id: rule_key(&id),
                    name: id.clone(),
                    short_description: SarifMessage { text: title },
                    full_description: None,
                    default_configuration: SarifConfiguration {
                        level: "warning".to_string(),
                    },
                });
            }
        }

        let findings = extract_findings(doc);
        let mut results = Vec::with_capacity(findings.len());
        for f in findings {
            let key = rule_key(&f.rule_id);
            if seen.insert(key.clone()) {
                rules.push(SarifRule {
                    id: key.clone(),
                    name: f.rule_id.clone(),
                    short_description: SarifMessage {
                        text: Some(f.title.clone().unwrap_or_else(|| f.rule_id.clone())),
                    },
                    full_description: None,
                    default_configuration: SarifConfiguration {
                        level: "error".to_string(),
                    },
                });
            }
            let (kind, level, suppressions) = match f.disposition {
                FindingDisposition::Failed => ("fail", "error", Vec::new()),
                FindingDisposition::Waived { justification } => (
                    "fail",
                    "error",
                    vec![SarifSuppression {
                        kind: "external".to_string(),
                        status: "accepted".to_string(),
                        justification,
                    }],
                ),
                FindingDisposition::Satisfied => ("pass", "none", Vec::new()),
                FindingDisposition::Unknown => ("review", "none", Vec::new()),
            };
            results.push(SarifResult {
                rule_id: key,
                kind: kind.to_string(),
                level: level.to_string(),
                message: SarifMessage {
                    text: Some(f.message),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: doc_uri.clone(),
                        },
                        region: None,
                    },
                }],
                suppressions,
            });
        }

        Ok(SarifReport {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "mizan-compliance-engine".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        information_uri: "https://mizan.dev".to_string(),
                        rules,
                    },
                },
                results,
            }],
        })
    }

    pub fn save_to_file(report: &SarifReport, output_path: &Path) -> Result<()> {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Configuration(format!("Failed to create SARIF report dir: {e}"))
            })?;
        }
        let json_str = serde_json::to_string_pretty(report)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        std::fs::write(output_path, json_str)
            .map_err(|e| AppError::Configuration(format!("Failed to write SARIF report: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sarif_export() {
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
        let report = SarifExporter::export_from_oscal(&doc, Path::new("catalog.json")).unwrap();

        assert_eq!(report.version, "2.1.0");
        assert_eq!(report.runs.len(), 1);
        assert_eq!(report.runs[0].tool.driver.rules.len(), 1);
        assert_eq!(report.runs[0].tool.driver.rules[0].id, "OSCAL-AC-1");
        // A catalog defines controls; it contains no findings.
        assert_eq!(report.runs[0].results.len(), 0);
    }

    #[test]
    fn test_sarif_export_assessment_findings() {
        let doc = OscalDocument::from_value(
            serde_json::json!({
                "assessment-results": {
                    "uuid": "ar",
                    "metadata": { "title": "t" },
                    "results": [{ "findings": [
                        { "rule_id": "k8s-no-root", "status": "FAILED", "violation": "runs as root" },
                        { "rule_id": "k8s-limits", "status": "WAIVED", "violation": "no limits", "reason": "ticket-1" }
                    ]}]
                }
            }),
            None,
        )
        .unwrap();
        let report = SarifExporter::export_from_oscal(&doc, Path::new("ar.json")).unwrap();
        let run = &report.runs[0];
        assert_eq!(run.results.len(), 2);
        assert_eq!(run.tool.driver.rules.len(), 2);
        assert_eq!(run.results[0].kind, "fail");
        assert_eq!(run.results[0].level, "error");
        assert!(run.results[0].suppressions.is_empty());
        assert_eq!(run.results[1].kind, "fail");
        assert_eq!(run.results[1].suppressions.len(), 1);
        assert_eq!(
            run.results[1].suppressions[0].justification.as_deref(),
            Some("ticket-1")
        );
    }
}
