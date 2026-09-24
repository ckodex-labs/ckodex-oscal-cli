use serde::Serialize;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    document::{
        authoring::split_document,
        parser::OscalDocument,
        schema::DocumentKind,
        validator::{ValidationOptions, validate_document},
    },
    error::{AppError, Result, io_error},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum MergeStrategy {
    Manual,
    Ours,
    Theirs,
}

impl MergeStrategy {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "manual" => Some(Self::Manual),
            "ours" | "local" => Some(Self::Ours),
            "theirs" | "upstream" => Some(Self::Theirs),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MergeReport {
    pub strategy: String,
    pub controls_merged: usize,
    pub added_from_upstream: Vec<String>,
    pub preserved_local_additions: Vec<String>,
    pub updated_from_upstream: Vec<String>,
    pub retained_local_modifications: Vec<String>,
    pub conflicts: Vec<MergeConflict>,
    pub is_clean: bool,
    pub output_file: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MergeConflict {
    pub control_id: String,
    pub field: String,
    pub local_summary: String,
    pub upstream_summary: String,
    pub resolution: String,
}

pub fn sync_and_merge(
    base_doc: &OscalDocument,
    upstream_doc: &OscalDocument,
    local_doc: &OscalDocument,
    strategy: MergeStrategy,
    output_path: Option<&Path>,
    split_output_dir: Option<&Path>,
) -> Result<(OscalDocument, MergeReport)> {
    if base_doc.kind != upstream_doc.kind || base_doc.kind != local_doc.kind {
        return Err(AppError::Configuration(format!(
            "Cannot 3-way merge documents of different kinds: base={}, upstream={}, local={}",
            base_doc.kind.root_key(),
            upstream_doc.kind.root_key(),
            local_doc.kind.root_key()
        )));
    }

    let base_root = base_doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Base document has no root object".to_owned()))?;
    let upstream_root = upstream_doc.root_object().ok_or_else(|| {
        AppError::Configuration("Upstream document has no root object".to_owned())
    })?;
    let local_root = local_doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Local document has no root object".to_owned()))?;

    let mut report = MergeReport {
        strategy: format!("{strategy:?}"),
        controls_merged: 0,
        added_from_upstream: Vec::new(),
        preserved_local_additions: Vec::new(),
        updated_from_upstream: Vec::new(),
        retained_local_modifications: Vec::new(),
        conflicts: Vec::new(),
        is_clean: true,
        output_file: output_path.map(|p| p.display().to_string()),
    };

    let base_controls = extract_controls_map(base_root);
    let upstream_controls = extract_controls_map(upstream_root);
    let local_controls = extract_controls_map(local_root);

    // Merge control IDs from all 3 sets
    let mut all_ids = BTreeMap::new();
    for k in base_controls.keys() {
        all_ids.insert(k.clone(), ());
    }
    for k in upstream_controls.keys() {
        all_ids.insert(k.clone(), ());
    }
    for k in local_controls.keys() {
        all_ids.insert(k.clone(), ());
    }

    let mut merged_controls = Vec::new();

    for id in all_ids.keys() {
        let in_base = base_controls.get(id);
        let in_upstream = upstream_controls.get(id);
        let in_local = local_controls.get(id);

        match (in_base, in_upstream, in_local) {
            // Case 1: Added only in Upstream
            (None, Some(up), None) => {
                merged_controls.push((*up).clone());
                report.added_from_upstream.push(id.clone());
            }
            // Case 2: Added only in Local
            (None, None, Some(loc)) => {
                merged_controls.push((*loc).clone());
                report.preserved_local_additions.push(id.clone());
            }
            // Case 3: Present in both Upstream & Local, but not in Base
            (None, Some(up), Some(loc)) => {
                if *up == *loc {
                    merged_controls.push((*loc).clone());
                } else {
                    let resolved = resolve_control_conflict(id, up, loc, strategy, &mut report);
                    merged_controls.push(resolved);
                }
            }
            // Case 4: In Base, unchanged in Upstream, modified or deleted in Local
            (Some(b), Some(up), Some(loc)) if *b == *up => {
                if *b != *loc {
                    report.retained_local_modifications.push(id.clone());
                }
                merged_controls.push((*loc).clone());
            }
            // Case 5: In Base, unchanged in Local, modified in Upstream
            (Some(b), Some(up), Some(loc)) if *b == *loc => {
                if *b != *up {
                    report.updated_from_upstream.push(id.clone());
                }
                merged_controls.push((*up).clone());
            }
            // Case 6: In Base, modified in BOTH Upstream and Local
            (Some(_b), Some(up), Some(loc)) => {
                if *up == *loc {
                    merged_controls.push((*loc).clone());
                } else {
                    let resolved = resolve_control_conflict(id, up, loc, strategy, &mut report);
                    merged_controls.push(resolved);
                }
            }
            // Case 7: Deleted in Local, kept in Upstream
            (Some(b), Some(up), None) => {
                if *b == *up {
                    // Local deleted it, upstream didn't change it -> respect local deletion
                    report
                        .retained_local_modifications
                        .push(format!("{id} (deleted)"));
                } else {
                    // Upstream modified it while Local deleted it -> conflict
                    match strategy {
                        MergeStrategy::Theirs => {
                            merged_controls.push((*up).clone());
                            report.updated_from_upstream.push(id.clone());
                        }
                        MergeStrategy::Ours => {
                            report
                                .retained_local_modifications
                                .push(format!("{id} (deleted)"));
                        }
                        MergeStrategy::Manual => {
                            report.is_clean = false;
                            report.conflicts.push(MergeConflict {
                                control_id: id.clone(),
                                field: "deletion".to_string(),
                                local_summary: "Deleted locally".to_string(),
                                upstream_summary: "Updated upstream".to_string(),
                                resolution: "Restored with upstream update and conflict notice"
                                    .to_string(),
                            });
                            merged_controls.push((*up).clone());
                        }
                    }
                }
            }
            // Case 8: Deleted in Upstream, kept in Local
            (Some(b), None, Some(loc)) => {
                if *b == *loc {
                    // Upstream deleted it, local didn't modify it -> respect upstream deletion
                    report.updated_from_upstream.push(format!("{id} (deleted)"));
                } else {
                    // Local modified it while Upstream deleted it -> conflict
                    match strategy {
                        MergeStrategy::Ours | MergeStrategy::Manual => {
                            merged_controls.push((*loc).clone());
                            report.retained_local_modifications.push(id.clone());
                        }
                        MergeStrategy::Theirs => {
                            report.updated_from_upstream.push(format!("{id} (deleted)"));
                        }
                    }
                }
            }
            // Case 9: Deleted in both
            (Some(_), None, None) => {}
            (None, None, None) => {}
        }
    }

    report.controls_merged = merged_controls.len();
    if !report.conflicts.is_empty() {
        report.is_clean = false;
    }

    // Build merged root document
    let mut merged_root = local_root.clone();
    let root_key = local_doc.kind.root_key();

    match local_doc.kind {
        DocumentKind::Catalog | DocumentKind::Profile => {
            merged_root.insert("controls".to_string(), Value::Array(merged_controls));
        }
        DocumentKind::Ssp => {
            if let Some(ctrl_imp) = merged_root
                .get_mut("control-implementation")
                .and_then(Value::as_object_mut)
            {
                ctrl_imp.insert(
                    "implemented-requirements".to_string(),
                    Value::Array(merged_controls),
                );
            }
        }
        _ => {
            merged_root.insert("controls".to_string(), Value::Array(merged_controls));
        }
    }

    // Update metadata timestamp
    if let Some(meta) = merged_root
        .get_mut("metadata")
        .and_then(Value::as_object_mut)
    {
        meta.insert(
            "last-modified".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );
    }

    let mut doc_val = Map::new();
    doc_val.insert(root_key.to_string(), Value::Object(merged_root));
    let final_val = Value::Object(doc_val);

    let merged_doc = OscalDocument::from_value(final_val, output_path.map(|p| p.to_path_buf()))?;
    let _ = validate_document(&merged_doc, &ValidationOptions::default());

    if let Some(out_p) = output_path {
        let formatted = serde_json::to_string_pretty(&merged_doc.value)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        fs::write(out_p, &formatted).map_err(|e| io_error(out_p, e))?;
    }

    if let Some(split_dir) = split_output_dir {
        split_document(&merged_doc, split_dir)?;
    }

    Ok((merged_doc, report))
}

fn resolve_control_conflict(
    id: &str,
    upstream: &Value,
    local: &Value,
    strategy: MergeStrategy,
    report: &mut MergeReport,
) -> Value {
    match strategy {
        MergeStrategy::Ours => {
            report.retained_local_modifications.push(id.to_string());
            local.clone()
        }
        MergeStrategy::Theirs => {
            report.updated_from_upstream.push(id.to_string());
            upstream.clone()
        }
        MergeStrategy::Manual => {
            report.is_clean = false;
            let mut conflicted_ctrl = local.clone();

            let local_str = serde_json::to_string(local).unwrap_or_default();
            let upstream_str = serde_json::to_string(upstream).unwrap_or_default();

            report.conflicts.push(MergeConflict {
                control_id: id.to_string(),
                field: "body/parts".to_string(),
                local_summary: if local_str.len() > 60 {
                    format!("{}...", &local_str[..60])
                } else {
                    local_str.clone()
                },
                upstream_summary: if upstream_str.len() > 60 {
                    format!("{}...", &upstream_str[..60])
                } else {
                    upstream_str.clone()
                },
                resolution: "Embedded conflict markers in statement/description".to_string(),
            });

            // Insert conflict markers into parts or prose
            if let Some(obj) = conflicted_ctrl.as_object_mut() {
                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    let up_desc = upstream
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let marked =
                        format!("<<<<<<< LOCAL\n{desc}\n=======\n{up_desc}\n>>>>>>> UPSTREAM");
                    obj.insert("description".to_string(), json!(marked));
                } else if let Some(parts) = obj.get_mut("parts").and_then(Value::as_array_mut) {
                    for p in parts {
                        if let Some(p_obj) = p.as_object_mut()
                            && let Some(prose) = p_obj.get("prose").and_then(Value::as_str)
                        {
                            let marked = format!(
                                "<<<<<<< LOCAL\n{prose}\n=======\n(Upstream modified part)\n>>>>>>> UPSTREAM"
                            );
                            p_obj.insert("prose".to_string(), json!(marked));
                        }
                    }
                }
            }

            conflicted_ctrl
        }
    }
}

fn extract_controls_map(root: &Map<String, Value>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    let root_val = Value::Object(root.clone());
    collect_controls_into_map(&root_val, &mut map);
    if let Some(ctrl_imp) = root.get("control-implementation")
        && let Some(reqs) = ctrl_imp
            .get("implemented-requirements")
            .and_then(Value::as_array)
    {
        for r in reqs {
            if let Some(id) = r.get("control-id").and_then(Value::as_str) {
                map.insert(id.to_string(), r.clone());
            }
        }
    }
    map
}

fn collect_controls_into_map(val: &Value, map: &mut BTreeMap<String, Value>) {
    if let Some(obj) = val.as_object() {
        if let Some(controls) = obj.get("controls").and_then(Value::as_array) {
            for c in controls {
                if let Some(id) = c.get("id").and_then(Value::as_str) {
                    map.insert(id.to_string(), c.clone());
                }
                collect_controls_into_map(c, map);
            }
        }
        if let Some(groups) = obj.get("groups").and_then(Value::as_array) {
            for g in groups {
                collect_controls_into_map(g, map);
            }
        }
    }
}
