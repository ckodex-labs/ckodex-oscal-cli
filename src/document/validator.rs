use chrono::DateTime;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    document::{parser::OscalDocument, schema::SchemaRegistry},
    error::Result,
};

#[derive(Clone, Debug, Default)]
pub struct ValidationOptions {
    pub strict_constraints: bool,
    pub quiet: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ValidationReport {
    pub file: Option<String>,
    pub kind: String,
    pub is_valid: bool,
    pub schema_valid: bool,
    pub constraints_valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Warning)
            .count()
    }
}

pub fn validate_document(
    doc: &OscalDocument,
    opts: &ValidationOptions,
) -> Result<ValidationReport> {
    let registry = SchemaRegistry::global()?;
    let validator = registry.validator_for(doc.kind);

    let mut diagnostics = Vec::new();
    let mut schema_valid = true;

    // Tier 1: JSON Schema Validation
    let errors = validator.iter_errors(&doc.value);
    for err in errors {
        schema_valid = false;
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            code: "OSCAL-SCHEMA-001".to_owned(),
            path: err.instance_path.to_string(),
            message: err.to_string(),
        });
    }

    // Tier 2: Metaschema / Integrity Constraints Validation
    let mut constraints_valid = true;
    validate_constraints(doc, opts, &mut diagnostics, &mut constraints_valid);

    let is_valid = schema_valid
        && (!opts.strict_constraints || constraints_valid)
        && (diagnostics
            .iter()
            .all(|d| d.level != DiagnosticLevel::Error));

    Ok(ValidationReport {
        file: doc.path.as_ref().map(|p| p.display().to_string()),
        kind: doc.kind.name().to_owned(),
        is_valid,
        schema_valid,
        constraints_valid,
        diagnostics,
    })
}

fn validate_constraints(
    doc: &OscalDocument,
    _opts: &ValidationOptions,
    diagnostics: &mut Vec<Diagnostic>,
    constraints_valid: &mut bool,
) {
    if let Some(root_obj) = doc.root_object() {
        // 1. Root UUID validation
        if let Some(uuid_val) = root_obj.get("uuid").and_then(Value::as_str) {
            match Uuid::parse_str(uuid_val) {
                Ok(parsed) => {
                    if parsed.get_version_num() != 4 {
                        diagnostics.push(Diagnostic {
                            level: DiagnosticLevel::Warning,
                            code: "oscal-uuid-v4".to_owned(),
                            path: format!("{}/uuid", doc.kind.root_key()),
                            message: format!("UUID '{uuid_val}' is valid UUID but not version 4"),
                        });
                    }
                }
                Err(err) => {
                    *constraints_valid = false;
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Error,
                        code: "oscal-uuid-syntax".to_owned(),
                        path: format!("{}/uuid", doc.kind.root_key()),
                        message: format!("Invalid UUID format '{uuid_val}': {err}"),
                    });
                }
            }
        }

        // 2. Metadata timestamp validation
        if let Some(meta) = root_obj.get("metadata").and_then(Value::as_object) {
            for ts_key in ["last-modified", "published"] {
                if let Some(ts_val) = meta.get(ts_key).and_then(Value::as_str) {
                    if let Err(err) = DateTime::parse_from_rfc3339(ts_val) {
                        *constraints_valid = false;
                        diagnostics.push(Diagnostic {
                            level: DiagnosticLevel::Error,
                            code: "oscal-datetime-rfc3339".to_owned(),
                            path: format!("{}/metadata/{}", doc.kind.root_key(), ts_key),
                            message: format!("Invalid RFC3339 date-time format for '{ts_key}': '{ts_val}' ({err})"),
                        });
                    }
                }
            }

            // 3. Metadata oscal-version check
            if let Some(ver) = meta.get("oscal-version").and_then(Value::as_str) {
                if !ver.starts_with("1.") {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        code: "oscal-version-target".to_owned(),
                        path: format!("{}/metadata/oscal-version", doc.kind.root_key()),
                        message: format!("Document targets OSCAL version '{ver}', validator is tuned for OSCAL 1.2.3"),
                    });
                }
            }
        }

        // 4. Back-matter resource reference integrity
        let mut declared_resource_uuids = std::collections::HashSet::new();
        if let Some(back_matter) = root_obj.get("back-matter").and_then(Value::as_object) {
            if let Some(resources) = back_matter.get("resources").and_then(Value::as_array) {
                for res in resources {
                    if let Some(uuid_str) = res.get("uuid").and_then(Value::as_str) {
                        declared_resource_uuids.insert(uuid_str.to_string());
                    }
                }
            }
        }

        check_back_matter_links(
            &doc.value,
            &declared_resource_uuids,
            doc.kind.root_key(),
            diagnostics,
        );
    }
}

fn check_back_matter_links(
    value: &Value,
    declared_resources: &std::collections::HashSet<String>,
    current_path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match value {
        Value::Object(map) => {
            if let Some(href) = map.get("href").and_then(Value::as_str) {
                if let Some(uuid_target) = href.strip_prefix('#') {
                    if !declared_resources.contains(uuid_target) && !declared_resources.is_empty() {
                        diagnostics.push(Diagnostic {
                            level: DiagnosticLevel::Warning,
                            code: "oscal-resource-link-unresolved".to_owned(),
                            path: format!("{current_path}/href"),
                            message: format!("Link href '{href}' does not match any declared resource UUID in back-matter"),
                        });
                    }
                }
            }
            for (k, v) in map {
                let next_path = format!("{current_path}/{k}");
                check_back_matter_links(v, declared_resources, &next_path, diagnostics);
            }
        }
        Value::Array(arr) => {
            for (idx, item) in arr.iter().enumerate() {
                let next_path = format!("{current_path}[{idx}]");
                check_back_matter_links(item, declared_resources, &next_path, diagnostics);
            }
        }
        _ => {}
    }
}
