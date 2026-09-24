use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{AppError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixReport {
    pub rule_id: String,
    pub target_file: PathBuf,
    pub applied: bool,
    pub dry_run: bool,
    pub description: String,
    pub diff: String,
}

pub struct FixEngine;

impl FixEngine {
    pub fn fix_rule(rule_id: &str, file_path: &Path, dry_run: bool) -> Result<FixReport> {
        if !file_path.exists() {
            return Err(AppError::Configuration(format!(
                "Target file does not exist: {}",
                file_path.display()
            )));
        }

        let content = fs::read_to_string(file_path).map_err(|e| {
            AppError::Configuration(format!("Failed to read file {}: {e}", file_path.display()))
        })?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.starts_with("dockerfile") || file_name.ends_with(".dockerfile") {
            Self::fix_dockerfile(rule_id, file_path, &content, dry_run)
        } else if file_name.ends_with(".json") {
            Self::fix_k8s_json(rule_id, file_path, &content, dry_run)
        } else {
            // Default to YAML (e.g. .yaml, .yml)
            Self::fix_k8s_yaml(rule_id, file_path, &content, dry_run)
        }
    }

    fn fix_dockerfile(
        rule_id: &str,
        file_path: &Path,
        content: &str,
        dry_run: bool,
    ) -> Result<FixReport> {
        let mut new_lines = Vec::new();
        let mut fixed = false;
        let mut desc = String::new();

        match rule_id {
            "cis-k8s-5.2.1" | "docker-non-root" | "root-user" => {
                let has_user = content.lines().any(|l| l.trim_start().starts_with("USER "));
                if has_user {
                    for line in content.lines() {
                        if line.trim_start().starts_with("USER root")
                            || line.trim_start() == "USER 0"
                        {
                            new_lines.push("USER nonroot".to_string());
                            fixed = true;
                            desc = "Replaced 'USER root' with 'USER nonroot'".to_string();
                        } else {
                            new_lines.push(line.to_string());
                        }
                    }
                } else {
                    new_lines.extend(content.lines().map(|s| s.to_string()));
                    new_lines.push("".to_string());
                    new_lines.push(
                        "# Security remediation: execute as non-root user (CIS 5.2.1)".to_string(),
                    );
                    new_lines.push("USER 65532:65532".to_string());
                    fixed = true;
                    desc = "Appended 'USER 65532:65532' to Dockerfile".to_string();
                }
            }
            _ => {
                return Ok(FixReport {
                    rule_id: rule_id.to_string(),
                    target_file: file_path.to_path_buf(),
                    applied: false,
                    dry_run,
                    description: format!("No automated Dockerfile fix known for rule '{rule_id}'"),
                    diff: String::new(),
                });
            }
        }

        let new_content = new_lines.join("\n");
        let diff = format_diff(content, &new_content, file_path);

        if fixed && !dry_run {
            fs::write(file_path, &new_content).map_err(|e| {
                AppError::Configuration(format!("Failed to write {}: {e}", file_path.display()))
            })?;
        }

        Ok(FixReport {
            rule_id: rule_id.to_string(),
            target_file: file_path.to_path_buf(),
            applied: fixed,
            dry_run,
            description: desc,
            diff,
        })
    }

    fn fix_k8s_yaml(
        rule_id: &str,
        file_path: &Path,
        content: &str,
        dry_run: bool,
    ) -> Result<FixReport> {
        let mut yaml_val: YamlValue = serde_yaml::from_str(content).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to parse YAML manifest {}: {e}",
                file_path.display()
            ))
        })?;

        let (applied, desc) = apply_k8s_yaml_fix(rule_id, &mut yaml_val);

        if !applied {
            return Ok(FixReport {
                rule_id: rule_id.to_string(),
                target_file: file_path.to_path_buf(),
                applied: false,
                dry_run,
                description: format!("Rule '{rule_id}' is already satisfied or not applicable"),
                diff: String::new(),
            });
        }

        let new_content = serde_yaml::to_string(&yaml_val)
            .map_err(|e| AppError::Configuration(format!("Failed to serialize YAML: {e}")))?;

        let diff = format_diff(content, &new_content, file_path);

        if !dry_run {
            fs::write(file_path, &new_content).map_err(|e| {
                AppError::Configuration(format!("Failed to write {}: {e}", file_path.display()))
            })?;
        }

        Ok(FixReport {
            rule_id: rule_id.to_string(),
            target_file: file_path.to_path_buf(),
            applied: true,
            dry_run,
            description: desc,
            diff,
        })
    }

    fn fix_k8s_json(
        rule_id: &str,
        file_path: &Path,
        content: &str,
        dry_run: bool,
    ) -> Result<FixReport> {
        let mut json_val: JsonValue = serde_json::from_str(content).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to parse JSON manifest {}: {e}",
                file_path.display()
            ))
        })?;

        let (applied, desc) = apply_k8s_json_fix(rule_id, &mut json_val);

        if !applied {
            return Ok(FixReport {
                rule_id: rule_id.to_string(),
                target_file: file_path.to_path_buf(),
                applied: false,
                dry_run,
                description: format!("Rule '{rule_id}' is already satisfied or not applicable"),
                diff: String::new(),
            });
        }

        let new_content = serde_json::to_string_pretty(&json_val)
            .map_err(|e| AppError::Configuration(format!("Failed to serialize JSON: {e}")))?;

        let diff = format_diff(content, &new_content, file_path);

        if !dry_run {
            fs::write(file_path, &new_content).map_err(|e| {
                AppError::Configuration(format!("Failed to write {}: {e}", file_path.display()))
            })?;
        }

        Ok(FixReport {
            rule_id: rule_id.to_string(),
            target_file: file_path.to_path_buf(),
            applied: true,
            dry_run,
            description: desc,
            diff,
        })
    }
}

fn apply_k8s_yaml_fix(rule_id: &str, root: &mut YamlValue) -> (bool, String) {
    let mut modified = false;
    let mut desc = String::new();

    // Check either root.spec.containers (Pod) or root.spec.template.spec.containers (Deployment, etc.)
    let containers_opt = if let Some(spec) = root.get_mut("spec") {
        if let Some(containers) = spec.get_mut("containers") {
            containers.as_sequence_mut()
        } else if let Some(template) = spec.get_mut("template") {
            if let Some(tpl_spec) = template.get_mut("spec") {
                tpl_spec
                    .get_mut("containers")
                    .and_then(|c| c.as_sequence_mut())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(containers) = containers_opt {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                let sec_ctx_key = YamlValue::String("securityContext".to_string());
                if !container_map.contains_key(&sec_ctx_key) {
                    container_map.insert(
                        sec_ctx_key.clone(),
                        YamlValue::Mapping(serde_yaml::Mapping::new()),
                    );
                }

                if let Some(sec_ctx) = container_map
                    .get_mut(&sec_ctx_key)
                    .and_then(|s| s.as_mapping_mut())
                {
                    match rule_id {
                        "cis-k8s-5.2.1" => {
                            sec_ctx.insert(
                                YamlValue::String("runAsNonRoot".to_string()),
                                YamlValue::Bool(true),
                            );
                            modified = true;
                            desc = "Set securityContext.runAsNonRoot = true".to_string();
                        }
                        "cis-k8s-5.2.2" => {
                            sec_ctx.insert(
                                YamlValue::String("readOnlyRootFilesystem".to_string()),
                                YamlValue::Bool(true),
                            );
                            modified = true;
                            desc = "Set securityContext.readOnlyRootFilesystem = true".to_string();
                        }
                        "cis-k8s-5.2.5" => {
                            sec_ctx.insert(
                                YamlValue::String("allowPrivilegeEscalation".to_string()),
                                YamlValue::Bool(false),
                            );
                            modified = true;
                            desc =
                                "Set securityContext.allowPrivilegeEscalation = false".to_string();
                        }
                        "cis-k8s-5.2.6" => {
                            sec_ctx.insert(
                                YamlValue::String("privileged".to_string()),
                                YamlValue::Bool(false),
                            );
                            modified = true;
                            desc = "Set securityContext.privileged = false".to_string();
                        }
                        "cis-k8s-5.2.7" => {
                            let mut cap_map = serde_yaml::Mapping::new();
                            cap_map.insert(
                                YamlValue::String("drop".to_string()),
                                YamlValue::Sequence(vec![YamlValue::String("ALL".to_string())]),
                            );
                            sec_ctx.insert(
                                YamlValue::String("capabilities".to_string()),
                                YamlValue::Mapping(cap_map),
                            );
                            modified = true;
                            desc = "Set securityContext.capabilities.drop = ['ALL']".to_string();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    (modified, desc)
}

fn apply_k8s_json_fix(rule_id: &str, root: &mut JsonValue) -> (bool, String) {
    let mut modified = false;
    let mut desc = String::new();

    let containers_opt = if let Some(spec) = root.get_mut("spec") {
        if let Some(containers) = spec.get_mut("containers") {
            containers.as_array_mut()
        } else if let Some(template) = spec.get_mut("template") {
            if let Some(tpl_spec) = template.get_mut("spec") {
                tpl_spec
                    .get_mut("containers")
                    .and_then(|c| c.as_array_mut())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(containers) = containers_opt {
        for container in containers {
            if let Some(container_obj) = container.as_object_mut() {
                if !container_obj.contains_key("securityContext") {
                    container_obj.insert("securityContext".to_string(), serde_json::json!({}));
                }

                if let Some(sec_ctx) = container_obj
                    .get_mut("securityContext")
                    .and_then(|s| s.as_object_mut())
                {
                    match rule_id {
                        "cis-k8s-5.2.1" => {
                            sec_ctx.insert("runAsNonRoot".to_string(), JsonValue::Bool(true));
                            modified = true;
                            desc = "Set securityContext.runAsNonRoot = true".to_string();
                        }
                        "cis-k8s-5.2.2" => {
                            sec_ctx.insert(
                                "readOnlyRootFilesystem".to_string(),
                                JsonValue::Bool(true),
                            );
                            modified = true;
                            desc = "Set securityContext.readOnlyRootFilesystem = true".to_string();
                        }
                        "cis-k8s-5.2.5" => {
                            sec_ctx.insert(
                                "allowPrivilegeEscalation".to_string(),
                                JsonValue::Bool(false),
                            );
                            modified = true;
                            desc =
                                "Set securityContext.allowPrivilegeEscalation = false".to_string();
                        }
                        "cis-k8s-5.2.6" => {
                            sec_ctx.insert("privileged".to_string(), JsonValue::Bool(false));
                            modified = true;
                            desc = "Set securityContext.privileged = false".to_string();
                        }
                        "cis-k8s-5.2.7" => {
                            sec_ctx.insert(
                                "capabilities".to_string(),
                                serde_json::json!({ "drop": ["ALL"] }),
                            );
                            modified = true;
                            desc = "Set securityContext.capabilities.drop = ['ALL']".to_string();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    (modified, desc)
}

fn format_diff(old: &str, new: &str, file: &Path) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let mut diff = format!("--- a/{}\n+++ b/{}\n", file.display(), file.display());

    let mut added = Vec::new();
    for line in &new_lines {
        if !old_lines.contains(line) {
            added.push(format!("+ {line}"));
        }
    }
    if added.is_empty() {
        diff.push_str("@@ No visible text difference @@\n");
    } else {
        diff.push_str(&added.join("\n"));
        diff.push('\n');
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_k8s_yaml_privileged() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-fix-{}", uuid::Uuid::new_v4()));
        let pod_yaml = temp_dir.join("pod.yaml");
        fs::create_dir_all(&temp_dir).unwrap();

        let initial_yaml = r#"apiVersion: v1
kind: Pod
spec:
  containers:
    - name: api
      image: nginx:alpine
"#;
        fs::write(&pod_yaml, initial_yaml).unwrap();

        let report = FixEngine::fix_rule("cis-k8s-5.2.6", &pod_yaml, false).unwrap();
        assert!(report.applied);
        assert!(!report.dry_run);

        let modified = fs::read_to_string(&pod_yaml).unwrap();
        assert!(modified.contains("privileged: false"));

        // Dry run on runAsNonRoot
        let dry_report = FixEngine::fix_rule("cis-k8s-5.2.1", &pod_yaml, true).unwrap();
        assert!(dry_report.applied);
        assert!(dry_report.dry_run);
        assert!(dry_report.diff.contains("+"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_fix_dockerfile_user() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-dockerfile-{}", uuid::Uuid::new_v4()));
        let df_path = temp_dir.join("Dockerfile");
        fs::create_dir_all(&temp_dir).unwrap();

        let initial_df = "FROM alpine:3.20\nRUN apk add --no-cache curl\n";
        fs::write(&df_path, initial_df).unwrap();

        let report = FixEngine::fix_rule("cis-k8s-5.2.1", &df_path, false).unwrap();
        assert!(report.applied);

        let modified = fs::read_to_string(&df_path).unwrap();
        assert!(modified.contains("USER 65532:65532"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
