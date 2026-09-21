use serde_json::{json, Map, Value};
use std::{fs, path::Path};

use crate::{
    document::{
        parser::OscalDocument,
        schema::DocumentKind,
        validator::{validate_document, ValidationOptions},
    },
    error::{io_error, AppError, Result},
};

pub fn export_to_csv(doc: &OscalDocument, output_path: Option<&Path>) -> Result<String> {
    let root_object = doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Document has no valid root object".to_owned()))?;

    let csv_content = match doc.kind {
        DocumentKind::Catalog | DocumentKind::Profile => export_catalog_to_csv(root_object)?,
        DocumentKind::Ssp => export_ssp_to_csv(root_object)?,
        DocumentKind::AssessmentResults => export_assessment_results_to_csv(root_object)?,
        DocumentKind::Poam => export_poam_to_csv(root_object)?,
        _ => export_generic_controls_to_csv(root_object)?,
    };

    if let Some(destination_path) = output_path {
        fs::write(destination_path, &csv_content).map_err(|err| io_error(destination_path, err))?;
    }

    Ok(csv_content)
}

pub fn import_from_csv(
    csv_content: &str,
    target_kind: Option<DocumentKind>,
    title: Option<&str>,
    output_path: Option<&Path>,
) -> Result<OscalDocument> {
    let mut lines = csv_content.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| AppError::Configuration("CSV file is empty".to_owned()))?;

    let headers: Vec<String> = parse_csv_line(header_line)
        .into_iter()
        .map(|header| header.trim().to_lowercase())
        .collect();

    let kind = target_kind.unwrap_or_else(|| {
        if headers
            .iter()
            .any(|h| h.contains("implementation_status") || h.contains("component"))
        {
            DocumentKind::Ssp
        } else if headers
            .iter()
            .any(|h| h.contains("finding_id") || h.contains("severity"))
        {
            DocumentKind::AssessmentResults
        } else if headers
            .iter()
            .any(|h| h.contains("poam_id") || h.contains("scheduled_completion"))
        {
            DocumentKind::Poam
        } else {
            DocumentKind::Catalog
        }
    });

    let doc_title = title.unwrap_or(match kind {
        DocumentKind::Catalog => "Imported OSCAL Catalog",
        DocumentKind::Ssp => "Imported System Security Plan",
        DocumentKind::AssessmentResults => "Imported Assessment Results",
        DocumentKind::Poam => "Imported Plan of Action and Milestones",
        _ => "Imported OSCAL Document",
    });

    let doc_uuid = uuid::Uuid::new_v4().to_string();
    let current_timestamp = chrono::Utc::now().to_rfc3339();

    let mut metadata = Map::new();
    metadata.insert("title".to_string(), json!(doc_title));
    metadata.insert("published".to_string(), json!(current_timestamp));
    metadata.insert("last-modified".to_string(), json!(current_timestamp));
    metadata.insert("version".to_string(), json!("1.0.0"));
    metadata.insert("oscal-version".to_string(), json!("1.2.3"));

    let mut root_object = Map::new();
    root_object.insert("uuid".to_string(), json!(doc_uuid));
    root_object.insert("metadata".to_string(), Value::Object(metadata));

    let rows: Vec<Vec<String>> = lines.map(parse_csv_line).collect();

    match kind {
        DocumentKind::Catalog | DocumentKind::Profile => {
            let mut controls = Vec::new();
            for row in rows {
                if let Some(control) = build_control_from_csv_row(&headers, &row) {
                    controls.push(control);
                }
            }
            root_object.insert("controls".to_string(), Value::Array(controls));
        }
        DocumentKind::Ssp => {
            let mut implemented_requirements = Vec::new();
            for row in rows {
                if let Some(requirement) = build_ssp_requirement_from_csv_row(&headers, &row) {
                    implemented_requirements.push(requirement);
                }
            }
            let mut control_implementation = Map::new();
            control_implementation
                .insert("description".to_string(), json!("Imported implementations"));
            control_implementation.insert(
                "implemented-requirements".to_string(),
                Value::Array(implemented_requirements),
            );
            root_object.insert(
                "control-implementation".to_string(),
                Value::Object(control_implementation),
            );
        }
        DocumentKind::AssessmentResults => {
            let mut findings = Vec::new();
            for row in rows {
                if let Some(finding) = build_finding_from_csv_row(&headers, &row) {
                    findings.push(finding);
                }
            }
            let mut result_entry = Map::new();
            result_entry.insert("uuid".to_string(), json!(uuid::Uuid::new_v4().to_string()));
            result_entry.insert("title".to_string(), json!("Imported Assessment Findings"));
            result_entry.insert("start".to_string(), json!(current_timestamp));
            result_entry.insert("findings".to_string(), Value::Array(findings));
            root_object.insert("results".to_string(), json!([result_entry]));
        }
        DocumentKind::Poam => {
            let mut poam_items = Vec::new();
            for row in rows {
                if let Some(poam_item) = build_poam_item_from_csv_row(&headers, &row) {
                    poam_items.push(poam_item);
                }
            }
            root_object.insert("poam-items".to_string(), Value::Array(poam_items));
        }
        _ => {
            let mut controls = Vec::new();
            for row in rows {
                if let Some(control) = build_control_from_csv_row(&headers, &row) {
                    controls.push(control);
                }
            }
            root_object.insert("controls".to_string(), Value::Array(controls));
        }
    }

    let mut doc_json = Map::new();
    doc_json.insert(kind.root_key().to_string(), Value::Object(root_object));
    let doc_val = Value::Object(doc_json);

    let doc = OscalDocument::from_value(doc_val, output_path.map(|p| p.to_path_buf()))?;
    let _ = validate_document(&doc, &ValidationOptions::default());

    if let Some(destination_path) = output_path {
        let json_str = serde_json::to_string_pretty(&doc.value)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        fs::write(destination_path, &json_str).map_err(|err| io_error(destination_path, err))?;
    }

    Ok(doc)
}

fn export_catalog_to_csv(root: &Map<String, Value>) -> Result<String> {
    let mut csv_buffer = String::from("control_id,family,title,parameters,statement,guidance\n");
    let mut controls = Vec::new();
    let root_val = Value::Object(root.clone());
    collect_controls_from_value(&root_val, &mut controls);

    for control in controls {
        let id = control.get("id").and_then(Value::as_str).unwrap_or("");
        let family = control.get("class").and_then(Value::as_str).unwrap_or("");
        let title = control.get("title").and_then(Value::as_str).unwrap_or("");

        let mut params_str = String::new();
        if let Some(params) = control.get("params").and_then(Value::as_array) {
            let parameter_entries: Vec<String> = params
                .iter()
                .filter_map(|param| {
                    let pid = param.get("id").and_then(Value::as_str).unwrap_or("");
                    let label = param.get("label").and_then(Value::as_str).unwrap_or("");
                    if !pid.is_empty() {
                        Some(format!("{pid}={label}"))
                    } else {
                        None
                    }
                })
                .collect();
            params_str = parameter_entries.join("; ");
        }

        let mut statement = String::new();
        let mut guidance = String::new();
        if let Some(parts) = control.get("parts").and_then(Value::as_array) {
            for part in parts {
                let name = part.get("name").and_then(Value::as_str).unwrap_or("");
                let prose = part.get("prose").and_then(Value::as_str).unwrap_or("");
                if name == "statement" {
                    statement = prose.to_string();
                } else if name == "guidance" {
                    guidance = prose.to_string();
                }
            }
        }

        csv_buffer.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv(id),
            escape_csv(family),
            escape_csv(title),
            escape_csv(&params_str),
            escape_csv(&statement),
            escape_csv(&guidance)
        ));
    }
    Ok(csv_buffer)
}

fn collect_controls_from_value(document_value: &Value, destination_list: &mut Vec<Value>) {
    if let Some(obj) = document_value.as_object() {
        if let Some(controls) = obj.get("controls").and_then(Value::as_array) {
            for control in controls {
                destination_list.push(control.clone());
                collect_controls_from_value(control, destination_list);
            }
        }
        if let Some(groups) = obj.get("groups").and_then(Value::as_array) {
            for group in groups {
                collect_controls_from_value(group, destination_list);
            }
        }
    }
}

fn export_ssp_to_csv(root: &Map<String, Value>) -> Result<String> {
    let mut out =
        String::from("control_id,title,parameters,implementation_status,components,description\n");
    if let Some(ctrl_imp) = root.get("control-implementation") {
        if let Some(reqs) = ctrl_imp
            .get("implemented-requirements")
            .and_then(Value::as_array)
        {
            for req in reqs {
                let id = req.get("control-id").and_then(Value::as_str).unwrap_or("");
                let desc = req.get("description").and_then(Value::as_str).unwrap_or("");

                let mut status = "implemented".to_string();
                if let Some(props) = req.get("props").and_then(Value::as_array) {
                    for p in props {
                        if p.get("name").and_then(Value::as_str) == Some("status") {
                            if let Some(v) = p.get("value").and_then(Value::as_str) {
                                status = v.to_string();
                            }
                        }
                    }
                }

                let mut comp_ids = Vec::new();
                if let Some(by_comps) = req.get("by-components").and_then(Value::as_array) {
                    for bc in by_comps {
                        if let Some(cuuid) = bc.get("component-uuid").and_then(Value::as_str) {
                            comp_ids.push(cuuid.to_string());
                        }
                    }
                }

                out.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    escape_csv(id),
                    escape_csv(id),
                    escape_csv(""),
                    escape_csv(&status),
                    escape_csv(&comp_ids.join("; ")),
                    escape_csv(desc)
                ));
            }
        }
    }
    Ok(out)
}

fn export_assessment_results_to_csv(root: &Map<String, Value>) -> Result<String> {
    let mut out = String::from("finding_id,target_control,severity,title,status,description\n");
    if let Some(results) = root.get("results").and_then(Value::as_array) {
        for res in results {
            if let Some(findings) = res.get("findings").and_then(Value::as_array) {
                for f in findings {
                    let id = f.get("id").and_then(Value::as_str).unwrap_or("");
                    let title = f.get("title").and_then(Value::as_str).unwrap_or("");
                    let desc = f.get("description").and_then(Value::as_str).unwrap_or("");
                    let target = f.get("target-id").and_then(Value::as_str).unwrap_or("");

                    let mut severity = "medium".to_string();
                    let mut status = "open".to_string();
                    if let Some(props) = f.get("props").and_then(Value::as_array) {
                        for p in props {
                            let name = p.get("name").and_then(Value::as_str).unwrap_or("");
                            let val = p.get("value").and_then(Value::as_str).unwrap_or("");
                            if name == "severity" {
                                severity = val.to_string();
                            } else if name == "status" {
                                status = val.to_string();
                            }
                        }
                    }

                    out.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        escape_csv(id),
                        escape_csv(target),
                        escape_csv(&severity),
                        escape_csv(title),
                        escape_csv(&status),
                        escape_csv(desc)
                    ));
                }
            }
        }
    }
    Ok(out)
}

fn export_poam_to_csv(root: &Map<String, Value>) -> Result<String> {
    let mut out =
        String::from("poam_id,control_id,title,scheduled_completion,status,description\n");
    if let Some(items) = root.get("poam-items").and_then(Value::as_array) {
        for item in items {
            let id = item.get("id").and_then(Value::as_str).unwrap_or("");
            let title = item.get("title").and_then(Value::as_str).unwrap_or("");
            let desc = item
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let scheduled = item
                .get("scheduled-completion-date")
                .and_then(Value::as_str)
                .unwrap_or("");

            let mut status = "open".to_string();
            let mut ctrl_id = String::new();
            if let Some(props) = item.get("props").and_then(Value::as_array) {
                for p in props {
                    let name = p.get("name").and_then(Value::as_str).unwrap_or("");
                    let val = p.get("value").and_then(Value::as_str).unwrap_or("");
                    if name == "status" {
                        status = val.to_string();
                    } else if name == "control-id" {
                        ctrl_id = val.to_string();
                    }
                }
            }

            out.push_str(&format!(
                "{},{},{},{},{},{}\n",
                escape_csv(id),
                escape_csv(&ctrl_id),
                escape_csv(title),
                escape_csv(scheduled),
                escape_csv(&status),
                escape_csv(desc)
            ));
        }
    }
    Ok(out)
}

fn export_generic_controls_to_csv(root: &Map<String, Value>) -> Result<String> {
    export_catalog_to_csv(root)
}

fn build_control_from_csv_row(headers: &[String], row: &[String]) -> Option<Value> {
    let id_idx = find_header_idx(headers, &["control_id", "id", "control", "number"])?;
    let id = row.get(id_idx)?.trim();
    if id.is_empty() {
        return None;
    }

    let title_idx = find_header_idx(headers, &["title", "name", "control_title"]);
    let title = title_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or(id);

    let family_idx = find_header_idx(headers, &["family", "class", "group"]);
    let family = family_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("");

    let mut parts = Vec::new();
    if let Some(stmt_idx) = find_header_idx(headers, &["statement", "description", "requirement"]) {
        if let Some(stmt) = row.get(stmt_idx) {
            if !stmt.trim().is_empty() {
                parts.push(json!({
                    "id": format!("{id}_smt"),
                    "name": "statement",
                    "prose": stmt.trim()
                }));
            }
        }
    }

    if let Some(guide_idx) =
        find_header_idx(headers, &["guidance", "supplemental_guidance", "remarks"])
    {
        if let Some(guide) = row.get(guide_idx) {
            if !guide.trim().is_empty() {
                parts.push(json!({
                    "id": format!("{id}_gdn"),
                    "name": "guidance",
                    "prose": guide.trim()
                }));
            }
        }
    }

    let mut ctrl = json!({
        "id": id,
        "class": family,
        "title": title
    });

    if !parts.is_empty() {
        ctrl["parts"] = Value::Array(parts);
    }

    Some(ctrl)
}

fn build_ssp_requirement_from_csv_row(headers: &[String], row: &[String]) -> Option<Value> {
    let id_idx = find_header_idx(headers, &["control_id", "id", "control", "number"])?;
    let id = row.get(id_idx)?.trim();
    if id.is_empty() {
        return None;
    }

    let desc_idx = find_header_idx(
        headers,
        &["description", "implementation", "statement", "prose"],
    );
    let desc = desc_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("Implementation detail provided in matrix.");

    let status_idx = find_header_idx(headers, &["implementation_status", "status", "state"]);
    let status = status_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("implemented");

    Some(json!({
        "uuid": uuid::Uuid::new_v4().to_string(),
        "control-id": id,
        "description": desc,
        "props": [
            {
                "name": "status",
                "value": status
            }
        ]
    }))
}

fn build_finding_from_csv_row(headers: &[String], row: &[String]) -> Option<Value> {
    let id_idx = find_header_idx(headers, &["finding_id", "id", "finding"])?;
    let id = row.get(id_idx)?.trim();
    if id.is_empty() {
        return None;
    }

    let title_idx = find_header_idx(headers, &["title", "name", "finding_title"]);
    let title = title_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("Finding");

    let target_idx = find_header_idx(headers, &["target_control", "control_id", "target"]);
    let target = target_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("");

    let sev_idx = find_header_idx(headers, &["severity", "impact", "risk"]);
    let severity = sev_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("medium");

    let desc_idx = find_header_idx(headers, &["description", "details", "detail"]);
    let desc = desc_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("");

    Some(json!({
        "id": id,
        "title": title,
        "description": desc,
        "target-id": target,
        "props": [
            {
                "name": "severity",
                "value": severity
            }
        ]
    }))
}

fn build_poam_item_from_csv_row(headers: &[String], row: &[String]) -> Option<Value> {
    let id_idx = find_header_idx(headers, &["poam_id", "id", "item_id"])?;
    let id = row.get(id_idx)?.trim();
    if id.is_empty() {
        return None;
    }

    let title_idx = find_header_idx(headers, &["title", "name"]);
    let title = title_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("Remediation Item");

    let desc_idx = find_header_idx(headers, &["description", "details"]);
    let desc = desc_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("");

    let sched_idx = find_header_idx(
        headers,
        &["scheduled_completion", "due_date", "completion_date"],
    );
    let scheduled = sched_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("");

    let status_idx = find_header_idx(headers, &["status", "state"]);
    let status = status_idx
        .and_then(|idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or("open");

    Some(json!({
        "id": id,
        "title": title,
        "description": desc,
        "scheduled-completion-date": scheduled,
        "props": [
            {
                "name": "status",
                "value": status
            }
        ]
    }))
}

fn find_header_idx(headers: &[String], candidates: &[&str]) -> Option<usize> {
    for c in candidates {
        if let Some(pos) = headers
            .iter()
            .position(|h| h == c || h.replace('-', "_") == *c)
        {
            return Some(pos);
        }
    }
    None
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut inside_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if inside_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    inside_quotes = !inside_quotes;
                }
            }
            ',' if !inside_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}
