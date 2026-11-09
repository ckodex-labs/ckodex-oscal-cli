// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mizan_oscal::document::{
    FedrampBaseline, KubeAuditor, KubeClusterClient, MergeStrategy, OscalDocument,
    RegorusEvaluator, analyze_blast_radius, inspect_document, sync_and_merge, validate_fedramp,
};
use serde_json::{Value, json};
use std::path::{Component, PathBuf};

/// Validates that a user-supplied path does not contain parent-directory traversal (`..`),
/// Windows/UNC path prefixes, NUL bytes, or symlink escapes (including intermediate ancestor
/// symlinks) when accessed via Tauri IPC.
fn validate_ipc_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err("IPC path must not be empty or contain NUL bytes".to_string());
    }
    let path = PathBuf::from(trimmed);
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                return Err(format!("IPC path traversal ('..') is forbidden: {trimmed}"));
            }
            Component::Prefix(_) => {
                return Err(format!("IPC path prefix is forbidden: {trimmed}"));
            }
            _ => {}
        }
    }

    let mut current = PathBuf::new();
    for comp in path.components() {
        current.push(comp.as_os_str());
        let is_system_root_alias =
            current != path && matches!(current.to_str(), Some("/var" | "/tmp" | "/etc"));
        if !is_system_root_alias
            && let Ok(meta) = std::fs::symlink_metadata(&current)
            && meta.file_type().is_symlink()
        {
            return Err(format!("Symlinked IPC paths are forbidden: {trimmed}"));
        }
    }

    if path.is_relative()
        && let Ok(cwd) = std::env::current_dir()
        && let Ok(canon_cwd) = std::fs::canonicalize(&cwd)
    {
        let joined = cwd.join(&path);
        let mut probe = joined.as_path();
        while !probe.exists() {
            if let Some(parent) = probe.parent() {
                probe = parent;
            } else {
                break;
            }
        }
        if probe.exists()
            && let Ok(canon_target) = std::fs::canonicalize(probe)
            && !canon_target.starts_with(&canon_cwd)
        {
            return Err(format!(
                "IPC relative path escapes workspace directory: {trimmed}"
            ));
        }
    }

    Ok(path)
}

#[tauri::command]
fn compute_blast_radius(file: String, target: String) -> Result<Value, String> {
    let path = validate_ipc_path(&file)?;
    let doc = OscalDocument::from_file(&path).map_err(|e| e.to_string())?;
    let report = analyze_blast_radius(&doc, &[], &target, 4).map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_fedramp_pmo(file: String, baseline: String) -> Result<Value, String> {
    let path = validate_ipc_path(&file)?;
    let doc = OscalDocument::from_file(&path).map_err(|e| e.to_string())?;
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
    let base_p = validate_ipc_path(&base)?;
    let upstream_p = validate_ipc_path(&upstream)?;
    let local_p = validate_ipc_path(&local)?;
    let base_doc = OscalDocument::from_file(&base_p).map_err(|e| e.to_string())?;
    let upstream_doc = OscalDocument::from_file(&upstream_p).map_err(|e| e.to_string())?;
    let local_doc = OscalDocument::from_file(&local_p).map_err(|e| e.to_string())?;

    let strat = MergeStrategy::from_str_name(&strategy).unwrap_or(MergeStrategy::Manual);
    let (_merged_doc, report) =
        sync_and_merge(&base_doc, &upstream_doc, &local_doc, strat, None, None)
            .map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn inspect_oscal_doc(file: String) -> Result<Value, String> {
    let path = validate_ipc_path(&file)?;
    let doc = OscalDocument::from_file(&path).map_err(|e| e.to_string())?;
    let report = inspect_document(&doc).map_err(|e| e.to_string())?;
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn query_control_metadata(file: String, control_id: String) -> Result<Value, String> {
    let path = validate_ipc_path(&file)?;
    let doc = OscalDocument::from_file(&path).map_err(|e| e.to_string())?;
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
    let validated_policy = validate_ipc_path(&policy_path)?;
    let mut evaluator = RegorusEvaluator::new();
    evaluator
        .add_policy_dir(&validated_policy)
        .map_err(|e| e.to_string())?;
    let result = evaluator
        .evaluate_compliance_rule(&query_package, input_json)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

const ALLOWED_TAURI_UTCP_TOOLS: &[&str] = &[
    "query_control",
    "inspect_document",
    "compute_blast_radius",
    "validate_fedramp",
    "evaluate_rego_policy",
    "export_baseline_catalog",
    "verify_slsa_provenance",
    "query_cas_store",
    "evaluate_policy_rulepack",
    "fabric_status",
    "fabric_validate_spiffe",
    "search_utcp_tools",
];

#[tauri::command]
fn get_utcp_manual(transport: Option<String>, base_url: Option<String>) -> Result<Value, String> {
    let t_str = transport.as_deref().unwrap_or("cli");
    let t_type = mizan_oscal::utcp::UtcpTransportType::from_str_name(t_str)
        .ok_or_else(|| format!("Unsupported UTCP transport: {t_str}"))?;
    let url = base_url.as_deref().unwrap_or("http://localhost:3000");
    let manual = mizan_oscal::utcp::UtcpManual::build_mizan_manual(t_type, url);
    serde_json::to_value(manual).map_err(|e| e.to_string())
}

#[tauri::command]
fn call_utcp_tool(tool_name: String, inputs: Option<Value>) -> Result<Value, String> {
    let normalized = tool_name
        .rsplit('.')
        .next()
        .unwrap_or(tool_name.as_str())
        .trim();
    if !ALLOWED_TAURI_UTCP_TOOLS.contains(&normalized) {
        return Err(format!(
            "UTCP tool not permitted over Tauri IPC: {tool_name}"
        ));
    }
    let args = inputs.unwrap_or_else(|| json!({}));
    if let Some(obj) = args.as_object() {
        if obj.contains_key("output_file") || obj.contains_key("output_dir") {
            return Err("Output path arguments are forbidden over Tauri IPC".to_string());
        }
        for (k, v) in obj {
            if (k == "file"
                || k.ends_with("_file")
                || k.ends_with("_path")
                || k.ends_with("_dir"))
                && let Some(p_str) = v.as_str()
            {
                validate_ipc_path(p_str)?;
            }
        }
    }
    let receipt =
        mizan_oscal::utcp::execute_utcp_tool(normalized, &args).map_err(|e| e.to_string())?;
    serde_json::to_value(receipt).map_err(|e| e.to_string())
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
            evaluate_rego_policy,
            get_utcp_manual,
            call_utcp_tool
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mizan desktop application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ipc_path_rejects_traversal_and_symlinks() {
        assert!(validate_ipc_path("").is_err());
        assert!(validate_ipc_path("   ").is_err());
        assert!(validate_ipc_path("bad\0path.json").is_err());
        assert!(validate_ipc_path("../secret.json").is_err());
        assert!(validate_ipc_path("sub/../../secret.json").is_err());

        #[cfg(unix)]
        {
            let tmp =
                std::env::temp_dir().join(format!("mizan-tauri-ipc-test-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&tmp);
            let real_dir = tmp.join("real");
            std::fs::create_dir_all(&real_dir).unwrap();
            let real_file = real_dir.join("doc.json");
            std::fs::write(&real_file, "{}").unwrap();

            assert!(validate_ipc_path(real_file.to_str().unwrap()).is_ok());

            let link_dir = tmp.join("symdir");
            std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();
            let via_symdir = link_dir.join("doc.json");
            assert!(
                validate_ipc_path(via_symdir.to_str().unwrap()).is_err(),
                "intermediate directory symlink must be rejected"
            );

            let link_file = tmp.join("symfile.json");
            std::os::unix::fs::symlink(&real_file, &link_file).unwrap();
            assert!(
                validate_ipc_path(link_file.to_str().unwrap()).is_err(),
                "leaf file symlink must be rejected"
            );

            let _ = std::fs::remove_dir_all(&tmp);
        }
    }

    #[test]
    fn test_tauri_utcp_ipc_commands() {
        let manual =
            get_utcp_manual(Some("http".to_string()), None).expect("get_utcp_manual should succeed");
        assert_eq!(manual["utcp_version"], "1.0.1");
        assert!(manual["tools"].as_array().unwrap().len() >= 19);

        let receipt = call_utcp_tool("mizan_oscal.fabric_status".to_string(), Some(json!({})))
            .expect("call_utcp_tool fabric_status should succeed");
        assert_eq!(receipt["status"], "VERIFIED");
        assert_eq!(receipt["output"]["status"], "OPERATIONAL");

        assert!(
            call_utcp_tool("export_utcp_manual".to_string(), Some(json!({}))).is_err(),
            "write-capable tool must be rejected over Tauri IPC"
        );
        assert!(
            call_utcp_tool(
                "inspect_document".to_string(),
                Some(json!({ "file": "../escape.json" }))
            )
            .is_err(),
            "path traversal in call_utcp_tool must be rejected"
        );
    }
}
