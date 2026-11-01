use serde::Serialize;
use serde_json::{Map, Value, json};
use std::{fs, path::Path};

use crate::{
    document::{
        authoring::splitter::split_document,
        catalog::{EmbeddedCatalogProvider, Jurisdiction},
        parser::OscalDocument,
    },
    error::{AppError, Result, io_error},
};

#[derive(Clone, Debug, Serialize)]
pub struct TemplateReport {
    pub standard: String,
    pub kind: String,
    pub title: String,
    pub output_path: Option<String>,
    pub split_to_dir: bool,
}

pub fn scaffold_template(
    standard: &str,
    doc_kind_str: &str,
    title: Option<&str>,
    output_path: Option<&Path>,
    split_dir: Option<&Path>,
) -> Result<(OscalDocument, TemplateReport)> {
    let standard_norm = standard.trim().to_lowercase();
    let kind_norm = doc_kind_str.trim().to_lowercase();
    let doc_title = title
        .map(String::from)
        .unwrap_or_else(|| default_title_for_standard(&standard_norm, &kind_norm));

    let template_val = build_template_json(&standard_norm, &kind_norm, &doc_title)?;
    let doc_str = serde_json::to_string_pretty(&template_val)
        .map_err(|e| AppError::Configuration(format!("Failed to format template: {e}")))?;

    if let Some(out_p) = output_path {
        fs::write(out_p, &doc_str).map_err(|e| io_error(out_p, e))?;
    }

    let doc = OscalDocument::from_str(&doc_str, output_path.map(Path::to_path_buf))?;

    if let Some(s_dir) = split_dir {
        split_document(&doc, s_dir)?;
    }

    let report = TemplateReport {
        standard: standard_norm,
        kind: doc.kind.name().to_string(),
        title: doc_title,
        output_path: output_path.map(|p| p.display().to_string()),
        split_to_dir: split_dir.is_some(),
    };

    Ok((doc, report))
}

fn default_title_for_standard(standard: &str, kind: &str) -> String {
    match standard {
        "fedramp-moderate" => format!("FedRAMP Moderate Baseline {kind}"),
        "fedramp-high" => format!("FedRAMP High Baseline {kind}"),
        "nist-800-53-r5" | "nist-800-53" => format!("NIST SP 800-53 Rev 5 {kind}"),
        "iso-27001" | "iso-27001-2022" => format!("ISO/IEC 27001:2022 ISMS {kind}"),
        "soc-2" => format!("SOC 2 Trust Services Criteria {kind}"),
        "cis-kubernetes" => format!("CIS Kubernetes Benchmark {kind}"),
        _ => format!("Custom {kind} ({standard})"),
    }
}

fn standard_to_jurisdiction(standard: &str) -> Result<Jurisdiction> {
    Jurisdiction::from_str_name(standard)
        .or_else(|| Jurisdiction::from_str_name(standard.split('-').next().unwrap_or(standard)))
        .ok_or_else(|| AppError::Configuration(format!("Unsupported standard: {standard}")))
}

fn build_template_json(standard: &str, kind: &str, title: &str) -> Result<Value> {
    let now = chrono::Utc::now().to_rfc3339();
    let doc_uuid = uuid::Uuid::new_v4().to_string();

    let jurisdiction = standard_to_jurisdiction(standard)?;
    let catalog_doc = EmbeddedCatalogProvider::get_catalog(jurisdiction)?;
    let catalog_root = catalog_doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Embedded catalog has no root object".to_owned()))?;
    let catalog_meta = catalog_root
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::Configuration("Embedded catalog metadata missing".to_owned()))?;
    let catalog_version = catalog_meta
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let oscal_version = catalog_meta
        .get("oscal-version")
        .and_then(Value::as_str)
        .unwrap_or("1.2.3");
    let controls = catalog_root
        .get("controls")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| AppError::Negative {
            class: "template",
            status: "embedded catalog has no controls".to_owned(),
        })?;

    let mut metadata = Map::new();
    metadata.insert("title".to_string(), json!(title));
    metadata.insert("published".to_string(), json!(now));
    metadata.insert("last-modified".to_string(), json!(now));
    metadata.insert("version".to_string(), json!(catalog_version));
    metadata.insert("oscal-version".to_string(), json!(oscal_version));

    match kind {
        "catalog" => Ok(json!({
            "catalog": {
                "uuid": doc_uuid,
                "metadata": metadata,
                "controls": controls
            }
        })),
        "profile" => Ok(json!({
            "profile": {
                "uuid": doc_uuid,
                "metadata": metadata,
                "imports": [
                    {
                        "href": "catalog.json",
                        "include-all": {}
                    }
                ],
                "modify": {
                    "set-parameters": [
                        {
                            "param-id": "ac-01_prm_1",
                            "values": ["at least annually"]
                        }
                    ]
                }
            }
        })),
        "ssp" | "system-security-plan" => {
            let comp_uuid = uuid::Uuid::new_v4().to_string();
            let info_uuid = uuid::Uuid::new_v4().to_string();
            let user_uuid = uuid::Uuid::new_v4().to_string();
            let req_uuid = uuid::Uuid::new_v4().to_string();
            Ok(json!({
                "system-security-plan": {
                    "uuid": doc_uuid,
                    "metadata": metadata,
                    "import-profile": {
                        "href": "profile.json"
                    },
                    "system-characteristics": {
                        "system-ids": [
                            {
                                "id": "SYS-001",
                                "identifier-type": "https://fedramp.gov"
                            }
                        ],
                        "system-name": title,
                        "description": format!("Enterprise system security plan for {title}."),
                        "system-information": {
                            "information-types": [
                                {
                                    "uuid": info_uuid,
                                    "title": "Operational System Data",
                                    "description": "System telemetry, configurations, and application logs.",
                                    "categorizations": [
                                        {
                                            "system": "http://doi.org/10.6028/NIST.SP.800-60v2r1",
                                            "information-type-ids": ["C.3.5.1"]
                                        }
                                    ],
                                    "confidentiality-impact": { "base": "fips-199-moderate" },
                                    "integrity-impact": { "base": "fips-199-moderate" },
                                    "availability-impact": { "base": "fips-199-moderate" }
                                }
                            ]
                        },
                        "security-sensitivity-level": if standard.contains("high") { "high" } else { "moderate" },
                        "status": { "state": "operational" },
                        "authorization-boundary": {
                            "description": "The authorization boundary includes all containerized workloads, ingress proxies, and attached storage."
                        }
                    },
                    "system-implementation": {
                        "users": [
                            {
                                "uuid": user_uuid,
                                "role-ids": ["system-admin"]
                            }
                        ],
                        "components": [
                            {
                                "uuid": comp_uuid,
                                "type": "software",
                                "title": title,
                                "description": "Primary containerized microservice payload",
                                "status": { "state": "operational" },
                                "props": [
                                    { "name": "version", "value": "1.0.0" }
                                ]
                            }
                        ]
                    },
                    "control-implementation": {
                        "description": "Standard baseline control implementation",
                        "implemented-requirements": [
                            {
                                "uuid": req_uuid,
                                "control-id": "ac-1",
                                "remarks": "Access control policy and procedures are maintained"
                            }
                        ]
                    }
                }
            }))
        }
        "component-definition" | "component" => {
            let comp_uuid = uuid::Uuid::new_v4().to_string();
            Ok(json!({
                "component-definition": {
                    "uuid": doc_uuid,
                    "metadata": metadata,
                    "components": [
                        {
                            "uuid": comp_uuid,
                            "type": "software",
                            "title": title,
                            "description": "Reusable validated OSCAL software component",
                            "props": [
                                { "name": "version", "value": "1.0.0" }
                            ]
                        }
                    ]
                }
            }))
        }
        _ => Err(AppError::Configuration(format!(
            "Unsupported template kind: '{kind}'. Use catalog, profile, ssp, or component-definition."
        ))),
    }
}
