use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use std::{fs, path::Path};
use uuid::Uuid;

use crate::{
    document::parser::{FileFormat, OscalDocument},
    error::{Result, io_error},
};

/// Canonical NIST OSCAL DNS Namespace for deterministic UUIDv5 generation
pub const OSCAL_NAMESPACE_UUID: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

/// Computes a deterministic UUIDv5 for an OSCAL document based on its kind, title, and version.
pub fn compute_deterministic_uuidv5(doc: &OscalDocument) -> Uuid {
    let root_key = doc.kind.root_key();
    let title = doc
        .metadata()
        .and_then(|m| m.get("title").and_then(Value::as_str))
        .unwrap_or("");
    let version = doc
        .metadata()
        .and_then(|m| m.get("version").and_then(Value::as_str))
        .unwrap_or("1.0.0");
    let seed = format!("urn:oscal:{root_key}:{title}:{version}");
    Uuid::new_v5(&OSCAL_NAMESPACE_UUID, seed.as_bytes())
}

#[derive(Clone, Debug, Serialize)]
pub struct LintIssue {
    pub code: &'static str,
    pub severity: &'static str,
    pub path: String,
    pub message: String,
    pub auto_fixable: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LintReport {
    pub file: Option<String>,
    pub kind: String,
    pub issues: Vec<LintIssue>,
    pub fixed_count: usize,
}

pub fn lint_document(doc: &mut OscalDocument, auto_fix: bool) -> Result<LintReport> {
    let mut issues = Vec::new();
    let mut fixed_count = 0;
    let mut mutated = false;

    let root_key = doc.kind.root_key().to_string();

    // 1. Root UUID linting & fix
    let mut root_uuid_issue = None;
    if let Some(root_obj) = doc.root_object() {
        match root_obj.get("uuid").and_then(Value::as_str) {
            None => {
                root_uuid_issue = Some(LintIssue {
                    code: "OSCAL-LINT-002",
                    severity: "ERROR",
                    path: format!("{root_key}/uuid"),
                    message: "Root document is missing 'uuid'".to_string(),
                    auto_fixable: true,
                });
            }
            Some(u) => {
                if Uuid::parse_str(u).is_err() {
                    root_uuid_issue = Some(LintIssue {
                        code: "OSCAL-LINT-002",
                        severity: "ERROR",
                        path: format!("{root_key}/uuid"),
                        message: format!("Root UUID '{u}' is invalid syntax"),
                        auto_fixable: true,
                    });
                }
            }
        }
    }

    if let Some(issue) = root_uuid_issue {
        if auto_fix {
            let deterministic_uuid = compute_deterministic_uuidv5(doc);
            if let Some(root_obj_mut) = doc.root_object_mut() {
                root_obj_mut.insert("uuid".to_string(), json!(deterministic_uuid.to_string()));
                fixed_count += 1;
                mutated = true;
            }
        }
        issues.push(issue);
    }

    // 2. Metadata last-modified timestamp linting & fix
    let mut last_mod_issue = None;
    if let Some(meta) = doc.metadata() {
        match meta.get("last-modified").and_then(Value::as_str) {
            None => {
                last_mod_issue = Some(LintIssue {
                    code: "OSCAL-LINT-001",
                    severity: "WARN",
                    path: format!("{root_key}/metadata/last-modified"),
                    message: "Missing 'last-modified' timestamp in metadata".to_string(),
                    auto_fixable: true,
                });
            }
            Some(ts) => {
                if DateTime::parse_from_rfc3339(ts).is_err() {
                    last_mod_issue = Some(LintIssue {
                        code: "OSCAL-LINT-001",
                        severity: "ERROR",
                        path: format!("{root_key}/metadata/last-modified"),
                        message: format!("Invalid RFC3339 date-time '{ts}'"),
                        auto_fixable: true,
                    });
                }
            }
        }
    }

    if let Some(issue) = last_mod_issue {
        if auto_fix && let Some(meta_mut) = doc.metadata_mut() {
            meta_mut.insert("last-modified".to_string(), json!(Utc::now().to_rfc3339()));
            fixed_count += 1;
            mutated = true;
        }
        issues.push(issue);
    }

    // 3. Metadata oscal-version linting & fix
    let mut oscal_ver_issue = None;
    if let Some(meta) = doc.metadata() {
        match meta.get("oscal-version").and_then(Value::as_str) {
            None => {
                oscal_ver_issue = Some(LintIssue {
                    code: "OSCAL-LINT-003",
                    severity: "WARN",
                    path: format!("{root_key}/metadata/oscal-version"),
                    message: "Missing 'oscal-version' in metadata (recommended '1.2.3')"
                        .to_string(),
                    auto_fixable: true,
                });
            }
            Some(ver) => {
                if ver != "1.2.3" && !ver.starts_with("1.") {
                    oscal_ver_issue = Some(LintIssue {
                        code: "OSCAL-LINT-003",
                        severity: "WARN",
                        path: format!("{root_key}/metadata/oscal-version"),
                        message: format!(
                            "Document oscal-version is '{ver}', current standard is 1.2.3"
                        ),
                        auto_fixable: false,
                    });
                }
            }
        }
    }

    if let Some(issue) = oscal_ver_issue {
        if auto_fix
            && issue.auto_fixable
            && let Some(meta_mut) = doc.metadata_mut()
        {
            meta_mut.insert("oscal-version".to_string(), json!("1.2.3"));
            fixed_count += 1;
            mutated = true;
        }
        issues.push(issue);
    }

    // 4. Save file if mutated and path is available
    if mutated && let Some(path) = &doc.path {
        save_document(doc, path)?;
    }

    Ok(LintReport {
        file: doc.path.as_ref().map(|p| p.display().to_string()),
        kind: doc.kind.name().to_string(),
        issues,
        fixed_count,
    })
}

fn save_document(doc: &OscalDocument, path: &Path) -> Result<()> {
    let content = match doc.format {
        FileFormat::Json | FileFormat::Proto => {
            serde_json::to_string_pretty(&doc.value).map_err(|e| {
                crate::error::AppError::Configuration(format!("Failed to serialize JSON: {e}"))
            })?
        }
        FileFormat::Yaml => serde_yaml::to_string(&doc.value).map_err(|e| {
            crate::error::AppError::Configuration(format!("Failed to serialize YAML: {e}"))
        })?,
        FileFormat::Csv => crate::document::tabular::export_to_csv(doc, None)?,
    };
    fs::write(path, content).map_err(|e| io_error(path, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_uuidv5_stability() {
        let json_str = r#"{
            "system-security-plan": {
                "metadata": {
                    "title": "Deterministic Test SSP",
                    "version": "1.0.0"
                }
            }
        }"#;
        let mut doc1 = OscalDocument::from_str(json_str, None).expect("parse doc1");
        let mut doc2 = OscalDocument::from_str(json_str, None).expect("parse doc2");

        let rep1 = lint_document(&mut doc1, true).expect("lint doc1");
        let rep2 = lint_document(&mut doc2, true).expect("lint doc2");

        assert!(rep1.fixed_count >= 1);
        assert!(rep2.fixed_count >= 1);

        let uuid1 = doc1.uuid().expect("uuid1 present");
        let uuid2 = doc2.uuid().expect("uuid2 present");

        assert_eq!(uuid1, uuid2, "UUIDv5 must be strictly deterministic across runs");
        assert_eq!(Uuid::parse_str(uuid1).unwrap().get_version_num(), 5);
    }
}
