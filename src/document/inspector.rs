use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{document::parser::OscalDocument, error::Result};

#[derive(Clone, Debug, Serialize)]
pub struct DocumentSummary {
    pub file: Option<String>,
    pub kind: String,
    pub title: String,
    pub version: String,
    pub oscal_version: String,
    pub uuid: String,
    pub last_modified: String,
    pub published: Option<String>,
    pub stats: ModelStats,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ModelStats {
    pub total_controls: usize,
    pub controls_by_family: BTreeMap<String, usize>,
    pub total_groups: usize,
    pub total_params: usize,
    pub total_roles: usize,
    pub total_parties: usize,
    pub total_components: usize,
    pub total_findings: usize,
    pub total_observations: usize,
    pub total_poam_items: usize,
    pub total_mappings: usize,
}

pub fn inspect_document(doc: &OscalDocument) -> Result<DocumentSummary> {
    let title = doc.title().unwrap_or("").to_string();
    let version = doc.version().unwrap_or("").to_string();
    let oscal_version = doc.oscal_version().unwrap_or("").to_string();
    let uuid = doc.uuid().unwrap_or("").to_string();
    let last_modified = doc.last_modified().unwrap_or("").to_string();
    let published = doc
        .metadata()
        .and_then(|m| m.get("published"))
        .and_then(Value::as_str)
        .map(String::from);

    let mut stats = ModelStats::default();

    if let Some(root_obj) = doc.root_object() {
        // Count roles and parties in metadata
        if let Some(meta) = root_obj.get("metadata").and_then(Value::as_object) {
            if let Some(roles) = meta.get("roles").and_then(Value::as_array) {
                stats.total_roles = roles.len();
            }
            if let Some(parties) = meta.get("parties").and_then(Value::as_array) {
                stats.total_parties = parties.len();
            }
        }

        // Count components
        if let Some(comps) = root_obj.get("components").and_then(Value::as_array) {
            stats.total_components = comps.len();
        } else if let Some(sys_imp) = root_obj
            .get("system-implementation")
            .and_then(Value::as_object)
            && let Some(comps) = sys_imp.get("components").and_then(Value::as_array)
        {
            stats.total_components = comps.len();
        }

        // Count findings and observations
        if let Some(results) = root_obj.get("results").and_then(Value::as_array) {
            for res in results {
                if let Some(findings) = res.get("findings").and_then(Value::as_array) {
                    stats.total_findings += findings.len();
                }
                if let Some(obs) = res.get("observations").and_then(Value::as_array) {
                    stats.total_observations += obs.len();
                }
            }
        }

        // Count poam items
        if let Some(items) = root_obj.get("poam-items").and_then(Value::as_array) {
            stats.total_poam_items = items.len();
        }

        // Count mappings
        if let Some(mappings) = root_obj.get("mappings").and_then(Value::as_array) {
            stats.total_mappings = mappings.len();
        }

        // Recursive control and group counting
        count_elements(&doc.value, &mut stats);
    }

    Ok(DocumentSummary {
        file: doc.path.as_ref().map(|p| p.display().to_string()),
        kind: doc.kind.name().to_string(),
        title,
        version,
        oscal_version,
        uuid,
        last_modified,
        published,
        stats,
    })
}

fn count_elements(val: &Value, stats: &mut ModelStats) {
    match val {
        Value::Object(obj) => {
            if obj.contains_key("title")
                && obj.contains_key("id")
                && (obj.contains_key("parts")
                    || obj.contains_key("params")
                    || obj.contains_key("controls"))
            {
                let id = obj.get("id").and_then(Value::as_str).unwrap_or("");
                stats.total_controls += 1;
                let family = id
                    .split(['-', '_', '.'])
                    .next()
                    .unwrap_or(id)
                    .to_ascii_uppercase();
                if !family.is_empty() {
                    *stats.controls_by_family.entry(family).or_default() += 1;
                }
            }
            if obj.contains_key("groups")
                && let Some(arr) = obj.get("groups").and_then(Value::as_array)
            {
                stats.total_groups += arr.len();
            }
            if obj.contains_key("params")
                && let Some(arr) = obj.get("params").and_then(Value::as_array)
            {
                stats.total_params += arr.len();
            }
            for (_, v) in obj {
                count_elements(v, stats);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                count_elements(v, stats);
            }
        }
        _ => {}
    }
}
