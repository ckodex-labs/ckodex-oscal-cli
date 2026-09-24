use serde::Serialize;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    document::{
        parser::OscalDocument,
        schema::DocumentKind,
        validator::{ValidationOptions, validate_document},
    },
    error::{AppError, Result, io_error},
};

#[derive(Clone, Debug, Serialize)]
pub struct AssembleReport {
    pub input_dir: String,
    pub kind: String,
    pub controls_assembled: usize,
    pub components_assembled: usize,
    pub is_valid: bool,
    pub output_file: Option<String>,
}

pub fn assemble_directory(
    input_dir: &Path,
    output_path: Option<&Path>,
) -> Result<(OscalDocument, AssembleReport)> {
    if !input_dir.is_dir() {
        return Err(AppError::Configuration(format!(
            "Input path is not a directory: {}",
            input_dir.display()
        )));
    }

    let meta_file = input_dir.join("_metadata.yaml");
    if !meta_file.exists() {
        return Err(AppError::Configuration(format!(
            "Missing _metadata.yaml manifest in {}",
            input_dir.display()
        )));
    }

    let meta_content = fs::read_to_string(&meta_file).map_err(|e| io_error(&meta_file, e))?;
    let meta_val: Value = serde_yaml::from_str(&meta_content)
        .map_err(|e| AppError::Configuration(format!("Failed to parse _metadata.yaml: {e}")))?;

    let kind_str = meta_val
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("catalog");
    let doc_kind = DocumentKind::from_root_key(kind_str)
        .or(match kind_str {
            "catalog" => Some(DocumentKind::Catalog),
            "profile" => Some(DocumentKind::Profile),
            "ssp" | "system-security-plan" => Some(DocumentKind::Ssp),
            "component-definition" | "component_definition" => {
                Some(DocumentKind::ComponentDefinition)
            }
            "assessment-plan" | "assessment_plan" => Some(DocumentKind::AssessmentPlan),
            "assessment-results" | "assessment_results" => Some(DocumentKind::AssessmentResults),
            "poam" | "plan-of-action-and-milestones" => Some(DocumentKind::Poam),
            "mapping" | "mapping-collection" => Some(DocumentKind::Mapping),
            _ => None,
        })
        .ok_or_else(|| {
            AppError::Configuration(format!("Unsupported OSCAL kind in metadata: {kind_str}"))
        })?;

    let root_uuid = meta_val
        .get("uuid")
        .and_then(Value::as_str)
        .filter(|u| !u.is_empty())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let mut root_obj = Map::new();
    root_obj.insert("uuid".to_string(), json!(root_uuid));

    // Extract metadata sub-object
    if let Some(metadata) = meta_val.get("metadata") {
        root_obj.insert("metadata".to_string(), metadata.clone());
    } else {
        // Fallback metadata
        let mut m = Map::new();
        m.insert("title".to_string(), json!(format!("Assembled {kind_str}")));
        m.insert("version".to_string(), json!("1.0.0"));
        m.insert(
            "last-modified".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );
        m.insert("oscal-version".to_string(), json!("1.2.3"));
        root_obj.insert("metadata".to_string(), Value::Object(m));
    }

    let controls_assembled;
    let mut components_assembled = 0;

    match doc_kind {
        DocumentKind::Catalog => {
            let controls = assemble_controls_from_dir(input_dir)?;
            controls_assembled = controls.len();
            root_obj.insert("controls".to_string(), Value::Array(controls));
        }
        DocumentKind::Ssp => {
            // System characteristics
            let sc_file = input_dir.join("system-characteristics.yaml");
            if sc_file.exists() {
                let sc_content = fs::read_to_string(&sc_file).map_err(|e| io_error(&sc_file, e))?;
                let sc_val: Value = serde_yaml::from_str(&sc_content).map_err(|e| {
                    AppError::Configuration(format!(
                        "Failed to parse system-characteristics.yaml: {e}"
                    ))
                })?;
                root_obj.insert("system-characteristics".to_string(), sc_val);
            }

            // Components
            let comps_dir = input_dir.join("components");
            let mut components = Vec::new();
            if comps_dir.exists() && comps_dir.is_dir() {
                for entry in fs::read_dir(&comps_dir).map_err(|e| io_error(&comps_dir, e))? {
                    let entry = entry.map_err(|e| io_error(&comps_dir, e))?;
                    let p = entry.path();
                    if p.extension()
                        .is_some_and(|ext| ext == "yaml" || ext == "yml" || ext == "json")
                    {
                        let c_str = fs::read_to_string(&p).map_err(|e| io_error(&p, e))?;
                        let c_val: Value = serde_yaml::from_str(&c_str).map_err(|e| {
                            AppError::Configuration(format!(
                                "Failed to parse component {}: {e}",
                                p.display()
                            ))
                        })?;
                        components.push(c_val);
                    }
                }
            }
            components_assembled = components.len();

            let mut sys_imp = Map::new();
            sys_imp.insert("users".to_string(), json!([]));
            sys_imp.insert("components".to_string(), Value::Array(components));
            root_obj.insert("system-implementation".to_string(), Value::Object(sys_imp));

            // Implemented requirements
            let reqs_dir = input_dir.join("control-implementation");
            let mut reqs = Vec::new();
            if reqs_dir.exists() && reqs_dir.is_dir() {
                for entry in fs::read_dir(&reqs_dir).map_err(|e| io_error(&reqs_dir, e))? {
                    let entry = entry.map_err(|e| io_error(&reqs_dir, e))?;
                    let p = entry.path();
                    if p.extension().is_some_and(|ext| ext == "md") {
                        let req_val = assemble_implemented_req(&p)?;
                        reqs.push(req_val);
                    }
                }
            }
            controls_assembled = reqs.len();

            let mut ctrl_imp = Map::new();
            ctrl_imp.insert(
                "description".to_string(),
                json!("Assembled control implementations"),
            );
            ctrl_imp.insert("implemented-requirements".to_string(), Value::Array(reqs));
            root_obj.insert(
                "control-implementation".to_string(),
                Value::Object(ctrl_imp),
            );
        }
        _ => {
            // Generic controls assemble
            let controls = assemble_controls_from_dir(input_dir)?;
            controls_assembled = controls.len();
            root_obj.insert("controls".to_string(), Value::Array(controls));
        }
    }

    // Check for back-matter
    let bm_file = input_dir.join("back-matter.yaml");
    if bm_file.exists() {
        let bm_content = fs::read_to_string(&bm_file).map_err(|e| io_error(&bm_file, e))?;
        let bm_val: Value = serde_yaml::from_str(&bm_content).map_err(|e| {
            AppError::Configuration(format!("Failed to parse back-matter.yaml: {e}"))
        })?;
        root_obj.insert("back-matter".to_string(), bm_val);
    }

    let mut outer = Map::new();
    outer.insert(doc_kind.root_key().to_string(), Value::Object(root_obj));
    let doc_json = Value::Object(outer);

    let doc_str = serde_json::to_string_pretty(&doc_json).map_err(|e| {
        AppError::Configuration(format!("Failed to serialize assembled document: {e}"))
    })?;

    if let Some(out_p) = output_path {
        fs::write(out_p, &doc_str).map_err(|e| io_error(out_p, e))?;
    }

    let assembled_doc = OscalDocument::from_str(&doc_str, output_path.map(Path::to_path_buf))?;
    let val_report = validate_document(&assembled_doc, &ValidationOptions::default())?;

    let report = AssembleReport {
        input_dir: input_dir.display().to_string(),
        kind: doc_kind.name().to_string(),
        controls_assembled,
        components_assembled,
        is_valid: val_report.is_valid,
        output_file: output_path.map(|p| p.display().to_string()),
    };

    Ok((assembled_doc, report))
}

fn assemble_controls_from_dir(dir: &Path) -> Result<Vec<Value>> {
    let mut controls_map: BTreeMap<String, Value> = BTreeMap::new();
    scan_dir_for_control_markdown(dir, &mut controls_map)?;
    Ok(controls_map.into_values().collect())
}

fn scan_dir_for_control_markdown(dir: &Path, map: &mut BTreeMap<String, Value>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| io_error(dir, e))? {
        let entry = entry.map_err(|e| io_error(dir, e))?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_control_markdown(&path, map)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if file_name.starts_with('_') || file_name == "README.md" {
                continue;
            }
            if let Ok(ctrl) = parse_control_markdown(&path) {
                let id = ctrl
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if !id.is_empty() {
                    map.insert(id, ctrl);
                }
            }
        }
    }
    Ok(())
}

fn parse_control_markdown(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path).map_err(|e| io_error(path, e))?;
    let (frontmatter, body) = extract_frontmatter_and_body(&content)?;

    let mut ctrl_map: Map<String, Value> = serde_yaml::from_str(&frontmatter).map_err(|e| {
        AppError::Configuration(format!(
            "Failed to parse YAML frontmatter in {}: {e}",
            path.display()
        ))
    })?;

    // Parse Markdown body sections into parts
    let parts = parse_markdown_parts(&body);
    if !parts.is_empty() {
        ctrl_map.insert("parts".to_string(), Value::Array(parts));
    }

    Ok(Value::Object(ctrl_map))
}

fn assemble_implemented_req(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path).map_err(|e| io_error(path, e))?;
    let (frontmatter, body) = extract_frontmatter_and_body(&content)?;

    let mut req_map: Map<String, Value> = serde_yaml::from_str(&frontmatter).map_err(|e| {
        AppError::Configuration(format!(
            "Failed to parse requirement frontmatter in {}: {e}",
            path.display()
        ))
    })?;

    if !req_map.contains_key("uuid") {
        req_map.insert("uuid".to_string(), json!(uuid::Uuid::new_v4().to_string()));
    }

    if !req_map.contains_key("description") {
        req_map.insert("description".to_string(), json!(body.trim()));
    }

    Ok(Value::Object(req_map))
}

fn extract_frontmatter_and_body(content: &str) -> Result<(String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Ok((String::new(), content.to_string()));
    }

    let rest = &trimmed[3..];
    if let Some(end_idx) = rest.find("\n---") {
        let frontmatter = rest[..end_idx].trim().to_string();
        let body_start = end_idx + 4;
        let body = rest[body_start..].trim().to_string();
        Ok((frontmatter, body))
    } else {
        Ok((String::new(), content.to_string()))
    }
}

fn parse_markdown_parts(body: &str) -> Vec<Value> {
    let mut parts = Vec::new();
    let mut current_section = String::new();
    let mut current_lines = Vec::new();

    for line in body.lines() {
        if let Some(stripped) = line.strip_prefix("## ") {
            if !current_section.is_empty() {
                let p = build_part_obj(&current_section, &current_lines.join("\n"));
                parts.push(p);
                current_lines.clear();
            }
            current_section = stripped.trim().to_lowercase();
        } else if !current_section.is_empty() {
            current_lines.push(line);
        }
    }

    if !current_section.is_empty() {
        let p = build_part_obj(&current_section, &current_lines.join("\n"));
        parts.push(p);
    }

    parts
}

fn build_part_obj(name: &str, prose: &str) -> Value {
    let mut p = Map::new();
    p.insert(
        "id".to_string(),
        json!(format!("part_{}", name.replace(' ', "_"))),
    );
    p.insert("name".to_string(), json!(name));
    p.insert("prose".to_string(), json!(prose.trim()));
    Value::Object(p)
}
