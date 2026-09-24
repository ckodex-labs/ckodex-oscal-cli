use chrono::Utc;
use serde_json::{Map, Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    document::{parser::OscalDocument, schema::DocumentKind},
    error::{AppError, Result, io_error},
};

pub fn resolve_profile(
    profile_doc: &OscalDocument,
    output_path: Option<&Path>,
) -> Result<OscalDocument> {
    if profile_doc.kind != DocumentKind::Profile {
        return Err(AppError::Configuration(format!(
            "Expected profile document for resolution, got {}",
            profile_doc.kind.name()
        )));
    }

    let profile_obj = profile_doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Malformed profile document".to_owned()))?;

    let imports = profile_obj
        .get("imports")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Configuration("Profile missing 'imports' array".to_owned()))?;

    let base_dir = profile_doc
        .path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."));

    // Collect parameter overrides from modify.set-parameters
    let mut param_overrides: HashMap<String, Value> = HashMap::new();
    if let Some(modify) = profile_obj.get("modify").and_then(Value::as_object)
        && let Some(set_params) = modify.get("set-parameters").and_then(Value::as_array)
    {
        for sp in set_params {
            if let Some(pid) = sp.get("param-id").and_then(Value::as_str) {
                param_overrides.insert(pid.to_string(), sp.clone());
            }
        }
    }

    // Collect alters from modify.alters
    let mut alters_map: HashMap<String, Vec<Value>> = HashMap::new();
    if let Some(modify) = profile_obj.get("modify").and_then(Value::as_object)
        && let Some(alters) = modify.get("alters").and_then(Value::as_array)
    {
        for alter in alters {
            if let Some(cid) = alter.get("control-id").and_then(Value::as_str) {
                alters_map
                    .entry(cid.to_string())
                    .or_default()
                    .push(alter.clone());
            }
        }
    }

    let mut resolved_controls: Vec<Value> = Vec::new();
    let mut resolved_groups: Vec<Value> = Vec::new();

    for import in imports {
        let href = import
            .get("href")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Configuration("Import missing 'href'".to_owned()))?;

        let import_path = resolve_href(base_dir, href);
        let imported_doc = OscalDocument::from_file(&import_path)?;

        if imported_doc.kind != DocumentKind::Catalog {
            return Err(AppError::Configuration(format!(
                "Imported document '{}' is a {}, only catalog imports are currently supported for resolution",
                href,
                imported_doc.kind.name()
            )));
        }

        let imported_cat = imported_doc.root_object().ok_or_else(|| {
            AppError::Configuration(format!("Imported catalog '{}' has no root object", href))
        })?;

        // Extract selection rules
        let (include_all, included_ids, excluded_ids) = parse_import_selection(import);

        // Process top-level controls
        if let Some(controls) = imported_cat.get("controls").and_then(Value::as_array) {
            for ctrl in controls {
                if is_control_selected(ctrl, include_all, &included_ids, &excluded_ids) {
                    let mut resolved_ctrl = ctrl.clone();
                    apply_control_modifications(&mut resolved_ctrl, &param_overrides, &alters_map)?;
                    resolved_controls.push(resolved_ctrl);
                }
            }
        }

        // Process groups
        if let Some(groups) = imported_cat.get("groups").and_then(Value::as_array) {
            for grp in groups {
                let mut resolved_grp = grp.clone();
                filter_and_modify_group(
                    &mut resolved_grp,
                    include_all,
                    &included_ids,
                    &excluded_ids,
                    &param_overrides,
                    &alters_map,
                )?;
                if group_has_controls(&resolved_grp) {
                    resolved_groups.push(resolved_grp);
                }
            }
        }
    }

    let profile_meta = profile_doc.metadata().cloned().unwrap_or_default();
    let profile_title = profile_meta
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Configuration("Profile metadata missing 'title'".to_owned()))?;

    let now_str = Utc::now().to_rfc3339();
    let new_uuid = Uuid::new_v4().to_string();

    let mut cat_meta = Map::new();
    cat_meta.insert(
        "title".to_owned(),
        json!(format!("[Resolved] {profile_title}")),
    );
    cat_meta.insert("published".to_owned(), json!(now_str));
    cat_meta.insert("last-modified".to_owned(), json!(now_str));
    cat_meta.insert("version".to_owned(), json!("1.0.0-resolved"));
    cat_meta.insert("oscal-version".to_owned(), json!("1.2.3"));

    let mut resolved_catalog = Map::new();
    resolved_catalog.insert("uuid".to_owned(), json!(new_uuid));
    resolved_catalog.insert("metadata".to_owned(), Value::Object(cat_meta));

    if !resolved_groups.is_empty() {
        resolved_catalog.insert("groups".to_owned(), Value::Array(resolved_groups));
    }
    if !resolved_controls.is_empty() {
        resolved_catalog.insert("controls".to_owned(), Value::Array(resolved_controls));
    }

    let mut root_val = Map::new();
    root_val.insert("catalog".to_owned(), Value::Object(resolved_catalog));
    let resolved_json = Value::Object(root_val);

    if let Some(out_p) = output_path {
        let serialized = serde_json::to_string_pretty(&resolved_json).map_err(|e| {
            AppError::Configuration(format!("Failed to serialize resolved catalog: {e}"))
        })?;
        fs::write(out_p, serialized).map_err(|e| io_error(out_p, e))?;
    }

    let resolved_str = serde_json::to_string(&resolved_json).map_err(AppError::Serialization)?;
    OscalDocument::from_str(&resolved_str, output_path.map(Path::to_path_buf))
}

fn resolve_href(base: &Path, href: &str) -> PathBuf {
    if href.starts_with("http://") || href.starts_with("https://") {
        PathBuf::from(href)
    } else {
        base.join(href)
    }
}

fn parse_import_selection(import: &Value) -> (bool, HashSet<String>, HashSet<String>) {
    let mut include_all = false;
    let mut included_ids = HashSet::new();
    let mut excluded_ids = HashSet::new();

    if let Some(inc) = import.get("include-all")
        && (inc.is_object() || inc.as_bool().unwrap_or(false))
    {
        include_all = true;
    }

    if let Some(inc_ctrls) = import.get("include-controls").and_then(Value::as_array) {
        for item in inc_ctrls {
            if let Some(with_child) = item.get("with-child-controls").and_then(Value::as_str)
                && with_child == "yes"
            {
                include_all = true;
            }
            if let Some(call) = item.get("with-ids").and_then(Value::as_array) {
                for id in call {
                    if let Some(id_str) = id.as_str() {
                        included_ids.insert(id_str.to_string());
                    }
                }
            }
            if let Some(call) = item.get("matching").and_then(Value::as_array) {
                for m in call {
                    if let Some(pat) = m.get("pattern").and_then(Value::as_str) {
                        included_ids.insert(pat.to_string());
                    }
                }
            }
        }
    } else if !include_all && import.get("include-all").is_none() {
        include_all = true;
    }

    if let Some(exc_ctrls) = import.get("exclude-controls").and_then(Value::as_array) {
        for item in exc_ctrls {
            if let Some(call) = item.get("with-ids").and_then(Value::as_array) {
                for id in call {
                    if let Some(id_str) = id.as_str() {
                        excluded_ids.insert(id_str.to_string());
                    }
                }
            }
        }
    }

    (include_all, included_ids, excluded_ids)
}

fn is_control_selected(
    ctrl: &Value,
    include_all: bool,
    included: &HashSet<String>,
    excluded: &HashSet<String>,
) -> bool {
    let cid = ctrl.get("id").and_then(Value::as_str).unwrap_or("");
    if excluded.contains(cid) {
        return false;
    }
    if include_all {
        return true;
    }
    included.contains(cid)
}

fn apply_control_modifications(
    ctrl: &mut Value,
    param_overrides: &HashMap<String, Value>,
    alters: &HashMap<String, Vec<Value>>,
) -> Result<()> {
    let cid = ctrl
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    // Override params
    if let Some(params) = ctrl.get_mut("params").and_then(Value::as_array_mut) {
        for param in params {
            if let Some(pid) = param.get("id").and_then(Value::as_str)
                && let Some(override_val) = param_overrides.get(pid)
            {
                if let Some(vals) = override_val.get("values") {
                    let param_obj = param.as_object_mut().ok_or_else(|| {
                        AppError::Configuration("Control param is not an object".to_owned())
                    })?;
                    param_obj.insert("values".to_owned(), vals.clone());
                }
                if let Some(label) = override_val.get("label") {
                    let param_obj = param.as_object_mut().ok_or_else(|| {
                        AppError::Configuration("Control param is not an object".to_owned())
                    })?;
                    param_obj.insert("label".to_owned(), label.clone());
                }
            }
        }
    }

    // Apply alters
    if let Some(alter_list) = alters.get(&cid) {
        for alter in alter_list {
            if let Some(adds) = alter.get("adds").and_then(Value::as_array) {
                for add in adds {
                    if let Some(props) = add.get("props").and_then(Value::as_array) {
                        let ctrl_obj = ctrl.as_object_mut().ok_or_else(|| {
                            AppError::Configuration("Control is not an object".to_owned())
                        })?;
                        let ctrl_props = ctrl_obj
                            .entry("props".to_owned())
                            .or_insert_with(|| Value::Array(Vec::new()));
                        if let Some(arr) = ctrl_props.as_array_mut() {
                            for p in props {
                                arr.push(p.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Recurse into sub-controls
    if let Some(sub_ctrls) = ctrl.get_mut("controls").and_then(Value::as_array_mut) {
        for sub in sub_ctrls {
            apply_control_modifications(sub, param_overrides, alters)?;
        }
    }

    Ok(())
}

fn filter_and_modify_group(
    grp: &mut Value,
    include_all: bool,
    included: &HashSet<String>,
    excluded: &HashSet<String>,
    param_overrides: &HashMap<String, Value>,
    alters: &HashMap<String, Vec<Value>>,
) -> Result<()> {
    if let Some(ctrls) = grp.get_mut("controls").and_then(Value::as_array_mut) {
        ctrls.retain(|c| is_control_selected(c, include_all, included, excluded));
        for c in ctrls.iter_mut() {
            apply_control_modifications(c, param_overrides, alters)?;
        }
    }

    if let Some(sub_grps) = grp.get_mut("groups").and_then(Value::as_array_mut) {
        for sub in sub_grps.iter_mut() {
            filter_and_modify_group(
                sub,
                include_all,
                included,
                excluded,
                param_overrides,
                alters,
            )?;
        }
        sub_grps.retain(group_has_controls);
    }

    Ok(())
}

fn group_has_controls(grp: &Value) -> bool {
    let has_direct = grp
        .get("controls")
        .and_then(Value::as_array)
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let has_sub = grp
        .get("groups")
        .and_then(Value::as_array)
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    has_direct || has_sub
}
