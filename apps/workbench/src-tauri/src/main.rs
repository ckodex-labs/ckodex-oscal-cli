// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mizan_oscal::document::{
    analyze_blast_radius, inspect_document, sync_and_merge, validate_fedramp, FedrampBaseline,
    KubeAuditor, KubeClusterClient, MergeStrategy, OscalDocument, RegorusEvaluator,
};
use serde_json::{json, Value};
use std::path::Path;

#[tauri::command]
fn compute_blast_radius(file: String, target: String) -> Result<Value, String> {
    let doc = OscalDocument::from_file(Path::new(&file)).map_err(|e| e.to_string())?;
    let report = analyze_blast_radius(&doc, &[], &target, 4).map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_fedramp_pmo(file: String, baseline: String) -> Result<Value, String> {
    let doc = OscalDocument::from_file(Path::new(&file)).map_err(|e| e.to_string())?;
    let base_enum = match baseline.to_lowercase().as_str() {
        "low" => FedrampBaseline::Low,
        "high" => FedrampBaseline::High,
        _ => FedrampBaseline::Moderate,
    };
    let report = validate_fedramp(&doc, base_enum).map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn sync_3way(
    base: String,
    upstream: String,
    local: String,
    strategy: String,
) -> Result<Value, String> {
    let base_doc = OscalDocument::from_file(Path::new(&base)).map_err(|e| e.to_string())?;
    let upstream_doc = OscalDocument::from_file(Path::new(&upstream)).map_err(|e| e.to_string())?;
    let local_doc = OscalDocument::from_file(Path::new(&local)).map_err(|e| e.to_string())?;

    let strat = MergeStrategy::from_str_name(&strategy).unwrap_or(MergeStrategy::Manual);
    let (_merged_doc, report) =
        sync_and_merge(&base_doc, &upstream_doc, &local_doc, strat, None, None)
            .map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn inspect_oscal_doc(file: String) -> Result<Value, String> {
    let doc = OscalDocument::from_file(Path::new(&file)).map_err(|e| e.to_string())?;
    let report = inspect_document(&doc).map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn query_control_metadata(file: String, control_id: String) -> Result<Value, String> {
    let doc = OscalDocument::from_file(Path::new(&file)).map_err(|e| e.to_string())?;
    let root = doc.root_object().ok_or("Invalid document root")?;
    let cid_lower = control_id.to_lowercase();

    if let Some(controls) = root.get("controls").and_then(Value::as_array) {
        for c in controls {
            if c.get("id").and_then(Value::as_str) == Some(&cid_lower) {
                return Ok(c.clone());
            }
        }
    }
    Ok(json!({ "id": control_id, "found": false }))
}

#[tauri::command]
async fn audit_k8s_cluster(namespace: Option<String>) -> Result<Value, String> {
    let mut auditor = KubeAuditor::new(KubeClusterClient::try_connect().await);
    let (_doc, report) = auditor
        .audit_cluster(namespace.as_deref(), None, None)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn evaluate_rego_policy(
    policy_path: String,
    input_json: Value,
    query_package: String,
) -> Result<Value, String> {
    let mut evaluator = RegorusEvaluator::new();
    evaluator
        .add_policy_dir(Path::new(&policy_path))
        .map_err(|e| e.to_string())?;
    let result = evaluator
        .evaluate_compliance_rule(&query_package, input_json)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            compute_blast_radius,
            validate_fedramp_pmo,
            sync_3way,
            inspect_oscal_doc,
            query_control_metadata,
            audit_k8s_cluster,
            evaluate_rego_policy
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mizan desktop application");
}
