use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::path::Path;

use crate::{
    document::{
        parser::{FileFormat, OscalDocument},
        schema::DocumentKind,
    },
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SbomSummary {
    pub format: String,
    pub spec_version: String,
    pub component_count: usize,
    pub direct_dependencies: usize,
    pub oscal_component_uuid: String,
}

pub struct SbomImporter;

impl SbomImporter {
    pub fn import_cyclonedx(sbom_json: &Value) -> Result<(OscalDocument, SbomSummary)> {
        let spec_version = sbom_json
            .get("specVersion")
            .and_then(Value::as_str)
            .unwrap_or("1.5")
            .to_string();

        let raw_components = sbom_json
            .get("components")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut oscal_components = Vec::new();

        for comp in &raw_components {
            let name = comp
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown-component");
            let version = comp
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("latest");
            let comp_type = comp
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("library");
            let purl = comp.get("purl").and_then(Value::as_str).unwrap_or("");
            let desc = comp
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("Imported from CycloneDX Software Bill of Materials (SBOM)");

            let mut protocols = Vec::new();
            if !purl.is_empty() {
                protocols.push(json!({
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "name": "purl",
                    "title": purl
                }));
            }

            oscal_components.push(json!({
                "uuid": uuid::Uuid::new_v4().to_string(),
                "type": comp_type,
                "title": format!("{name}@{version}"),
                "description": desc,
                "purpose": "Supply chain third-party dependency (NIST SP 800-53 SA-11 / SR-3)",
                "protocols": protocols,
                "control-implementations": [
                    {
                        "uuid": uuid::Uuid::new_v4().to_string(),
                        "source": "https://csrc.nist.gov/publications/detail/sp/800-53/rev-5/final",
                        "description": "Component inventory and supply chain vulnerability tracking",
                        "implemented-requirements": [
                            {
                                "uuid": uuid::Uuid::new_v4().to_string(),
                                "control-id": "sa-11",
                                "description": format!("SBOM tracking for package {name} version {version}")
                            },
                            {
                                "uuid": uuid::Uuid::new_v4().to_string(),
                                "control-id": "si-2",
                                "description": "Continuous vulnerability remediation tracking for dependency"
                            }
                        ]
                    }
                ]
            }));
        }

        let comp_uuid = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let oscal_doc_value = json!({
            "component-definition": {
                "uuid": comp_uuid,
                "metadata": {
                    "title": format!("OSCAL Component Definition from CycloneDX v{spec_version} SBOM"),
                    "last-modified": now,
                    "version": "1.0.0",
                    "oscal-version": "1.2.3",
                    "remarks": "Synthesized automatically by Mizan SBOM Ingestor"
                },
                "components": oscal_components
            }
        });

        let summary = SbomSummary {
            format: "CycloneDX".to_string(),
            spec_version,
            component_count: raw_components.len(),
            direct_dependencies: raw_components.len(),
            oscal_component_uuid: comp_uuid,
        };

        let mut root_map = Map::new();
        root_map.insert(
            "component-definition".to_string(),
            oscal_doc_value["component-definition"].clone(),
        );

        let doc = OscalDocument {
            path: None,
            format: FileFormat::Json,
            kind: DocumentKind::ComponentDefinition,
            value: Value::Object(root_map),
        };

        Ok((doc, summary))
    }

    pub fn import_file(path: &Path) -> Result<(OscalDocument, SbomSummary)> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Configuration(format!("Failed to read SBOM file: {e}")))?;
        let val: Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Configuration(format!("Invalid SBOM JSON: {e}")))?;

        if val.get("bomFormat").and_then(Value::as_str) == Some("CycloneDX")
            || val.get("components").is_some()
        {
            Self::import_cyclonedx(&val)
        } else {
            Err(AppError::Configuration(
                "Unsupported SBOM format: expected CycloneDX JSON".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclonedx_sbom_import() {
        let sbom = json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.6",
            "version": 1,
            "components": [
                {
                    "type": "library",
                    "name": "tokio",
                    "version": "1.43.0",
                    "purl": "pkg:cargo/tokio@1.43.0",
                    "description": "An event-driven, non-blocking I/O platform for writing asynchronous applications."
                },
                {
                    "type": "library",
                    "name": "regorus",
                    "version": "0.4.0",
                    "purl": "pkg:cargo/regorus@0.4.0",
                    "description": "A pure Rust Open Policy Agent (OPA) / Rego interpreter."
                }
            ]
        });

        let (doc, summary) = SbomImporter::import_cyclonedx(&sbom).unwrap();
        assert_eq!(summary.component_count, 2);
        assert_eq!(summary.format, "CycloneDX");

        let root = doc.root_object().unwrap();
        let comps = root.get("components").and_then(Value::as_array).unwrap();
        assert_eq!(comps.len(), 2);
        assert!(comps[0]
            .get("title")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("tokio@1.43.0"));
    }
}
