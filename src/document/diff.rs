use serde::Serialize;
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    document::parser::OscalDocument,
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize)]
pub struct DocumentDiffReport {
    pub kind: String,
    pub title_a: String,
    pub title_b: String,
    pub version_a: String,
    pub version_b: String,
    pub controls_added: Vec<String>,
    pub controls_removed: Vec<String>,
    pub controls_modified: Vec<ControlDiff>,
    pub metadata_changes: Vec<FieldDiff>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FieldDiff {
    pub field: String,
    pub old_val: String,
    pub new_val: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlDiff {
    pub control_id: String,
    pub title_a: Option<String>,
    pub title_b: Option<String>,
    pub changes: Vec<FieldDiff>,
    pub text_diff: String,
}

pub fn diff_documents(doc_a: &OscalDocument, doc_b: &OscalDocument) -> Result<DocumentDiffReport> {
    if doc_a.kind != doc_b.kind {
        return Err(AppError::Configuration(format!(
            "Cannot diff documents of different kinds: '{}' vs '{}'",
            doc_a.kind.name(),
            doc_b.kind.name()
        )));
    }

    let mut metadata_changes = Vec::new();
    let title_a = doc_a.title().unwrap_or("").to_string();
    let title_b = doc_b.title().unwrap_or("").to_string();
    if title_a != title_b {
        metadata_changes.push(FieldDiff {
            field: "title".to_string(),
            old_val: title_a.clone(),
            new_val: title_b.clone(),
        });
    }

    let version_a = doc_a.version().unwrap_or("").to_string();
    let version_b = doc_b.version().unwrap_or("").to_string();
    if version_a != version_b {
        metadata_changes.push(FieldDiff {
            field: "version".to_string(),
            old_val: version_a.clone(),
            new_val: version_b.clone(),
        });
    }

    let last_mod_a = doc_a.last_modified().unwrap_or("").to_string();
    let last_mod_b = doc_b.last_modified().unwrap_or("").to_string();
    if last_mod_a != last_mod_b {
        metadata_changes.push(FieldDiff {
            field: "last-modified".to_string(),
            old_val: last_mod_a,
            new_val: last_mod_b,
        });
    }

    let ctrls_a = collect_all_controls(&doc_a.value);
    let ctrls_b = collect_all_controls(&doc_b.value);

    let ids_a: BTreeSet<&String> = ctrls_a.keys().collect();
    let ids_b: BTreeSet<&String> = ctrls_b.keys().collect();

    let controls_added: Vec<String> = ids_b.difference(&ids_a).map(|s| (*s).clone()).collect();
    let controls_removed: Vec<String> = ids_a.difference(&ids_b).map(|s| (*s).clone()).collect();

    let mut controls_modified = Vec::new();
    for id in ids_a.intersection(&ids_b) {
        let ca = &ctrls_a[*id];
        let cb = &ctrls_b[*id];

        let str_a = serde_json::to_string_pretty(ca).unwrap();
        let str_b = serde_json::to_string_pretty(cb).unwrap();

        if str_a != str_b {
            let t_a = ca.get("title").and_then(Value::as_str).map(String::from);
            let t_b = cb.get("title").and_then(Value::as_str).map(String::from);

            let diff = TextDiff::from_lines(&str_a, &str_b);
            let mut diff_buf = String::new();
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                diff_buf.push_str(&format!("{sign} {}", change.value()));
            }

            let mut changes = Vec::new();
            if t_a != t_b {
                changes.push(FieldDiff {
                    field: "title".to_string(),
                    old_val: t_a.clone().unwrap_or_default(),
                    new_val: t_b.clone().unwrap_or_default(),
                });
            }

            controls_modified.push(ControlDiff {
                control_id: (*id).clone(),
                title_a: t_a,
                title_b: t_b,
                changes,
                text_diff: diff_buf,
            });
        }
    }

    Ok(DocumentDiffReport {
        kind: doc_a.kind.name().to_string(),
        title_a,
        title_b,
        version_a,
        version_b,
        controls_added,
        controls_removed,
        controls_modified,
        metadata_changes,
    })
}

fn collect_all_controls(val: &Value) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    recurse_controls(val, &mut map);
    map
}

fn recurse_controls(val: &Value, map: &mut BTreeMap<String, Value>) {
    match val {
        Value::Object(obj) => {
            if let Some(id) = obj.get("id").and_then(Value::as_str) {
                // If it looks like a control (has title or params or parts)
                if obj.contains_key("title")
                    || obj.contains_key("parts")
                    || obj.contains_key("params")
                {
                    map.insert(id.to_string(), val.clone());
                }
            }
            for (_, v) in obj {
                recurse_controls(v, map);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                recurse_controls(v, map);
            }
        }
        _ => {}
    }
}
