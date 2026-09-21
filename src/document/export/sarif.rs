use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

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
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
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
        let root = doc
            .root_object()
            .ok_or_else(|| AppError::Configuration("Document root is not an object".to_string()))?;

        let mut rules = Vec::new();
        let mut results = Vec::new();

        let doc_uri = source_path.to_string_lossy().to_string();

        if let Some(controls) = root.get("controls").and_then(Value::as_array) {
            for ctrl in controls {
                let id = ctrl
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| AppError::Configuration("Control missing 'id'".to_owned()))?;
                let title = ctrl.get("title").and_then(Value::as_str);
                let desc = ctrl.get("description").and_then(Value::as_str).or(title);

                rules.push(SarifRule {
                    id: format!("OSCAL-{}", id.to_uppercase()),
                    name: id.to_string(),
                    short_description: SarifMessage {
                        text: title.map(String::from),
                    },
                    full_description: desc.map(|d| SarifMessage {
                        text: Some(d.to_string()),
                    }),
                    default_configuration: SarifConfiguration {
                        level: "warning".to_string(),
                    },
                });

                results.push(SarifResult {
                    rule_id: format!("OSCAL-{}", id.to_uppercase()),
                    level: "note".to_string(),
                    message: SarifMessage {
                        text: title.map(|t| {
                            format!("Control {id} ({t}) evaluated across compliance baseline.")
                        }),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: doc_uri.clone(),
                            },
                            region: Some(SarifRegion {
                                start_line: 1,
                                start_column: 1,
                            }),
                        },
                    }],
                });
            }
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
        assert_eq!(report.runs[0].results.len(), 1);
    }
}
