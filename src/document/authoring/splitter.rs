use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path};

use crate::{
    document::parser::OscalDocument,
    error::{AppError, Result, io_error},
};

#[derive(Clone, Debug, Serialize)]
pub struct SplitReport {
    pub output_dir: String,
    pub kind: String,
    pub files_created: usize,
    pub controls_split: usize,
    pub components_split: usize,
}

pub fn split_document(doc: &OscalDocument, out_dir: &Path) -> Result<SplitReport> {
    fs::create_dir_all(out_dir).map_err(|e| io_error(out_dir, e))?;

    let _root_key = doc.kind.root_key();
    let root_obj = doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Document has no valid root object".to_owned()))?;

    let mut files_created = 0;
    let mut controls_split = 0;
    let mut components_split = 0;

    // 1. Write header manifest / metadata
    let mut meta_yaml = String::new();
    meta_yaml.push_str(&format!("kind: {}\n", doc.kind.name()));
    meta_yaml.push_str(&format!("uuid: {}\n", doc.uuid().unwrap_or("")));
    if let Some(meta) = root_obj.get("metadata") {
        let meta_str = serde_yaml::to_string(meta).map_err(|e| {
            AppError::Configuration(format!("Failed to serialize metadata YAML: {e}"))
        })?;
        meta_yaml.push_str(&meta_str);
    }

    let meta_file = out_dir.join("_metadata.yaml");
    fs::write(&meta_file, &meta_yaml).map_err(|e| io_error(&meta_file, e))?;
    files_created += 1;

    // 2. Handle Catalog / Group controls
    if let Some(controls) = root_obj.get("controls").and_then(Value::as_array) {
        let ctrls_dir = out_dir.join("controls");
        fs::create_dir_all(&ctrls_dir).map_err(|e| io_error(&ctrls_dir, e))?;
        for ctrl in controls {
            let ctrl_id = ctrl.get("id").and_then(Value::as_str).unwrap_or("unknown");
            let file_path = ctrls_dir.join(format!("{ctrl_id}.md"));
            write_control_markdown(&file_path, ctrl)?;
            files_created += 1;
            controls_split += 1;
        }
    }

    if let Some(groups) = root_obj.get("groups").and_then(Value::as_array) {
        for grp in groups {
            let grp_id = grp.get("id").and_then(Value::as_str).unwrap_or("group");
            let grp_dir = out_dir.join(grp_id);
            fs::create_dir_all(&grp_dir).map_err(|e| io_error(&grp_dir, e))?;

            if let Some(ctrls) = grp.get("controls").and_then(Value::as_array) {
                for ctrl in ctrls {
                    let ctrl_id = ctrl.get("id").and_then(Value::as_str).unwrap_or("unknown");
                    let file_path = grp_dir.join(format!("{ctrl_id}.md"));
                    write_control_markdown(&file_path, ctrl)?;
                    files_created += 1;
                    controls_split += 1;
                }
            }
        }
    }

    // 3. Handle SSP System Characteristics & Components & Control Implementations
    if let Some(sys_char) = root_obj.get("system-characteristics") {
        let sys_char_file = out_dir.join("system-characteristics.yaml");
        let sys_char_str = serde_yaml::to_string(sys_char).map_err(|e| {
            AppError::Configuration(format!("Failed to serialize system-characteristics: {e}"))
        })?;
        fs::write(&sys_char_file, &sys_char_str).map_err(|e| io_error(&sys_char_file, e))?;
        files_created += 1;
    }

    if let Some(sys_imp) = root_obj.get("system-implementation")
        && let Some(comps) = sys_imp.get("components").and_then(Value::as_array)
    {
        let comps_dir = out_dir.join("components");
        fs::create_dir_all(&comps_dir).map_err(|e| io_error(&comps_dir, e))?;
        for comp in comps {
            let comp_title = comp.get("title").and_then(Value::as_str).unwrap_or("comp");
            let slug = comp_title
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric(), "_");
            let file_path = comps_dir.join(format!("{slug}.yaml"));
            let comp_str = serde_yaml::to_string(comp).map_err(|e| {
                AppError::Configuration(format!("Failed to serialize component: {e}"))
            })?;
            fs::write(&file_path, &comp_str).map_err(|e| io_error(&file_path, e))?;
            files_created += 1;
            components_split += 1;
        }
    }

    if let Some(ctrl_imp) = root_obj.get("control-implementation")
        && let Some(reqs) = ctrl_imp
            .get("implemented-requirements")
            .and_then(Value::as_array)
    {
        let reqs_dir = out_dir.join("control-implementation");
        fs::create_dir_all(&reqs_dir).map_err(|e| io_error(&reqs_dir, e))?;
        for req in reqs {
            let cid = req
                .get("control-id")
                .and_then(Value::as_str)
                .unwrap_or("req");
            let file_path = reqs_dir.join(format!("{cid}.md"));
            write_implemented_req_markdown(&file_path, req)?;
            files_created += 1;
            controls_split += 1;
        }
    }

    // 4. Back-matter preservation
    if let Some(back_matter) = root_obj.get("back-matter") {
        let bm_file = out_dir.join("back-matter.yaml");
        let bm_str = serde_yaml::to_string(back_matter).map_err(|e| {
            AppError::Configuration(format!("Failed to serialize back-matter: {e}"))
        })?;
        fs::write(&bm_file, &bm_str).map_err(|e| io_error(&bm_file, e))?;
        files_created += 1;
    }

    Ok(SplitReport {
        output_dir: out_dir.display().to_string(),
        kind: doc.kind.name().to_string(),
        files_created,
        controls_split,
        components_split,
    })
}

fn write_control_markdown(file_path: &Path, ctrl: &Value) -> Result<()> {
    let mut frontmatter_map = serde_json::Map::new();
    if let Some(id) = ctrl.get("id") {
        frontmatter_map.insert("id".to_string(), id.clone());
    }
    if let Some(title) = ctrl.get("title") {
        frontmatter_map.insert("title".to_string(), title.clone());
    }
    if let Some(class) = ctrl.get("class") {
        frontmatter_map.insert("class".to_string(), class.clone());
    }
    if let Some(params) = ctrl.get("params") {
        frontmatter_map.insert("params".to_string(), params.clone());
    }
    if let Some(props) = ctrl.get("props") {
        frontmatter_map.insert("props".to_string(), props.clone());
    }
    if let Some(links) = ctrl.get("links") {
        frontmatter_map.insert("links".to_string(), links.clone());
    }

    let yaml_frontmatter = serde_yaml::to_string(&frontmatter_map).map_err(|e| {
        AppError::Configuration(format!("Failed to format control frontmatter YAML: {e}"))
    })?;

    let title_str = ctrl
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Control");
    let id_str = ctrl.get("id").and_then(Value::as_str).unwrap_or("");

    let mut body = String::new();
    body.push_str("---\n");
    body.push_str(&yaml_frontmatter);
    body.push_str("---\n\n");
    body.push_str(&format!("# {} - {}\n\n", id_str.to_uppercase(), title_str));

    if let Some(parts) = ctrl.get("parts").and_then(Value::as_array) {
        for part in parts {
            let pname = part.get("name").and_then(Value::as_str).unwrap_or("part");
            let prose = part.get("prose").and_then(Value::as_str).unwrap_or("");
            body.push_str(&format!("## {}\n\n{}\n\n", capitalize_first(pname), prose));

            if let Some(sub_parts) = part.get("parts").and_then(Value::as_array) {
                for sp in sub_parts {
                    let s_pname = sp.get("name").and_then(Value::as_str).unwrap_or("item");
                    let s_prose = sp.get("prose").and_then(Value::as_str).unwrap_or("");
                    body.push_str(&format!(
                        "### {}\n\n{}\n\n",
                        capitalize_first(s_pname),
                        s_prose
                    ));
                }
            }
        }
    }

    fs::write(file_path, &body).map_err(|e| io_error(file_path, e))?;
    Ok(())
}

fn write_implemented_req_markdown(file_path: &Path, req: &Value) -> Result<()> {
    let mut frontmatter_map = serde_json::Map::new();
    if let Some(u) = req.get("uuid") {
        frontmatter_map.insert("uuid".to_string(), u.clone());
    }
    if let Some(cid) = req.get("control-id") {
        frontmatter_map.insert("control-id".to_string(), cid.clone());
    }
    if let Some(set_params) = req.get("set-parameters") {
        frontmatter_map.insert("set-parameters".to_string(), set_params.clone());
    }
    if let Some(props) = req.get("props") {
        frontmatter_map.insert("props".to_string(), props.clone());
    }

    let yaml_frontmatter = serde_yaml::to_string(&frontmatter_map).map_err(|e| {
        AppError::Configuration(format!(
            "Failed to format requirement frontmatter YAML: {e}"
        ))
    })?;

    let cid_str = req.get("control-id").and_then(Value::as_str).unwrap_or("");
    let desc_str = req.get("description").and_then(Value::as_str).unwrap_or("");

    let mut body = String::new();
    body.push_str("---\n");
    body.push_str(&yaml_frontmatter);
    body.push_str("---\n\n");
    body.push_str(&format!(
        "# Implemented Requirement for {}\n\n",
        cid_str.to_uppercase()
    ));

    if !desc_str.is_empty() {
        body.push_str(&format!("## Overview\n\n{}\n\n", desc_str));
    }

    if let Some(by_comps) = req.get("by-components").and_then(Value::as_array) {
        body.push_str("## Component Implementations\n\n");
        for bc in by_comps {
            let cuuid = bc
                .get("component-uuid")
                .and_then(Value::as_str)
                .unwrap_or("");
            let desc = bc.get("description").and_then(Value::as_str).unwrap_or("");
            body.push_str(&format!("### Component `{}`\n\n{}\n\n", cuuid, desc));
        }
    }

    fs::write(file_path, &body).map_err(|e| io_error(file_path, e))?;
    Ok(())
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
