use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path};

use crate::{
    document::parser::OscalDocument,
    error::{AppError, Result, io_error},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PolicyTarget {
    Rego,
    Kyverno,
    All,
}

impl PolicyTarget {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "rego" | "opa" => Some(Self::Rego),
            "kyverno" | "k8s" => Some(Self::Kyverno),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PolicyCompileReport {
    pub source_file: String,
    pub source_kind: String,
    pub target: String,
    pub policies_generated: usize,
    pub generated_files: Vec<String>,
}

pub fn compile_policies(
    doc: &OscalDocument,
    target: PolicyTarget,
    out_dir: &Path,
) -> Result<PolicyCompileReport> {
    fs::create_dir_all(out_dir).map_err(|e| io_error(out_dir, e))?;

    let root_obj = doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Missing root object".to_owned()))?;

    let mut generated_files = Vec::new();
    let mut count = 0;

    // Extract control IDs and component info from SSP or Component Definition
    let mut control_rules = Vec::new();

    if let Some(ctrl_imp) = root_obj.get("control-implementation") {
        if let Some(reqs) = ctrl_imp
            .get("implemented-requirements")
            .and_then(Value::as_array)
        {
            for req in reqs {
                let cid = req
                    .get("control-id")
                    .and_then(Value::as_str)
                    .unwrap_or("control");
                let desc = req.get("description").and_then(Value::as_str).unwrap_or("");
                control_rules.push((cid.to_string(), desc.to_string()));
            }
        }
    } else if let Some(components) = root_obj.get("components").and_then(Value::as_array) {
        for comp in components {
            if let Some(cimps) = comp
                .get("control-implementations")
                .and_then(Value::as_array)
            {
                for cimp in cimps {
                    if let Some(reqs) = cimp
                        .get("implemented-requirements")
                        .and_then(Value::as_array)
                    {
                        for req in reqs {
                            let cid = req
                                .get("control-id")
                                .and_then(Value::as_str)
                                .unwrap_or("control");
                            let desc = req.get("description").and_then(Value::as_str).unwrap_or("");
                            control_rules.push((cid.to_string(), desc.to_string()));
                        }
                    }
                }
            }
        }
    }

    if control_rules.is_empty() {
        // Fallback: extract from catalog controls
        if let Some(controls) = root_obj.get("controls").and_then(Value::as_array) {
            for ctrl in controls {
                let cid = ctrl.get("id").and_then(Value::as_str).unwrap_or("control");
                let title = ctrl.get("title").and_then(Value::as_str).unwrap_or("");
                control_rules.push((cid.to_string(), title.to_string()));
            }
        }
    }

    // Generate Rego Policies
    if target == PolicyTarget::Rego || target == PolicyTarget::All {
        for (cid, desc) in &control_rules {
            let safe_cid: String = cid
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            let rego_content = generate_rego_policy(cid, desc);
            let file_name = format!(
                "{}.rego",
                if safe_cid.trim_matches('_').is_empty() {
                    "control"
                } else {
                    safe_cid.trim_matches('_')
                }
            );
            let file_path = out_dir.join(&file_name);
            crate::document::io_safe::atomic_write_bytes(&file_path, rego_content.as_bytes())?;
            generated_files.push(file_name);
            count += 1;
        }

        // Generate central data manifest
        let manifest_path = out_dir.join("policy_manifest.json");
        let manifest_val = serde_json::json!({
            "source_kind": doc.kind.name(),
            "source_title": doc.title().unwrap_or("OSCAL Document"),
            "rules_count": control_rules.len(),
            "generated_at": chrono::Utc::now().to_rfc3339()
        });
        let manifest_json = serde_json::to_string_pretty(&manifest_val)
            .map_err(|e| crate::error::AppError::Configuration(e.to_string()))?;
        crate::document::io_safe::atomic_write_bytes(&manifest_path, manifest_json.as_bytes())?;
        generated_files.push("policy_manifest.json".to_string());
    }

    // Generate Kyverno ClusterPolicies
    if target == PolicyTarget::Kyverno || target == PolicyTarget::All {
        for (cid, desc) in &control_rules {
            let safe_cid: String = cid
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' {
                        c
                    } else {
                        '-'
                    }
                })
                .collect();
            let kyverno_yaml = generate_kyverno_policy(cid, desc);
            let file_name = format!(
                "kyverno_{}.yaml",
                if safe_cid.trim_matches('-').is_empty() {
                    "control"
                } else {
                    safe_cid.trim_matches('-')
                }
            );
            let file_path = out_dir.join(&file_name);
            crate::document::io_safe::atomic_write_bytes(&file_path, kyverno_yaml.as_bytes())?;
            generated_files.push(file_name);
            count += 1;
        }
    }

    Ok(PolicyCompileReport {
        source_file: doc
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        source_kind: doc.kind.name().to_string(),
        target: format!("{:?}", target),
        policies_generated: count,
        generated_files,
    })
}

fn sanitize_rego_identifier(raw: &str) -> String {
    let mapped: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = mapped.trim_matches('_');
    if trimmed.is_empty() {
        return "control".to_string();
    }
    if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("c_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

fn sanitize_k8s_name(raw: &str) -> String {
    let mapped: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = mapped.trim_matches('-');
    if trimmed.is_empty() {
        "control".to_string()
    } else {
        trimmed.to_string()
    }
}

fn escape_policy_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\r' | '\n' => out.push(' '),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}

fn generate_rego_policy(cid: &str, desc: &str) -> String {
    let pkg_name = sanitize_rego_identifier(cid);
    let safe_cid = escape_policy_string(cid);
    let safe_desc = escape_policy_string(desc);
    format!(
        r#"# METADATA
# title: OSCAL Compliance Rule for {safe_cid}
# description: {safe_desc}
# custom:
#   control_id: "{safe_cid}"
#   framework: "NIST-SP-800-53"

package oscal.compliance.{pkg_name}

import future.keywords.if
import future.keywords.in

default allow = false
default violations = []

# AC/SC policy evaluation rule
allow if {{
    count(violations) == 0
}}

# Check rule violations
violations contains msg if {{
    not input.compliance.{pkg_name}.enabled
    msg := "Control {safe_cid} enforcement is disabled in target configuration"
}}

violations contains msg if {{
    input.compliance.{pkg_name}.status == "unsatisfied"
    msg := "Control {safe_cid} status is marked unsatisfied"
}}
"#
    )
}

fn generate_kyverno_policy(cid: &str, desc: &str) -> String {
    let policy_name = format!("oscal-compliance-{}", sanitize_k8s_name(cid));
    let safe_cid = escape_policy_string(cid);
    let safe_desc = escape_policy_string(desc);
    format!(
        r#"apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: {policy_name}
  annotations:
    policies.kyverno.io/title: "OSCAL Control {safe_cid}"
    policies.kyverno.io/category: "Compliance & Security"
    policies.kyverno.io/description: "{safe_desc}"
    oscal.compliance/control-id: "{safe_cid}"
spec:
  validationFailureAction: Audit
  background: true
  rules:
    - name: enforce-{policy_name}
      match:
        any:
          - resources:
              kinds:
                - Pod
                - Deployment
                - StatefulSet
      validate:
        message: "Resource must adhere to OSCAL control {safe_cid} baseline security constraints."
        pattern:
          spec:
            template:
              spec:
                =(securityContext):
                  =(runAsNonRoot): true
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c2p_policy_compiler_escapes_multiline_descriptions_and_identifiers() {
        let cid = "ac-2(1)";
        let desc = "line 1\n\"quoted\" line 2";

        let rego = generate_rego_policy(cid, desc);
        assert!(rego.contains("package oscal.compliance.ac_2_1"));
        assert!(rego.contains(r#"# description: line 1 \"quoted\" line 2"#));
        assert!(!rego.contains("\n\"quoted\""));

        let mut evaluator = crate::document::policy::RegorusEvaluator::new();
        evaluator
            .add_policy_str("ac_2_1.rego", &rego)
            .expect("generated Rego policy must compile cleanly in Regorus");

        let kyverno = generate_kyverno_policy(cid, desc);
        assert!(kyverno.contains("name: oscal-compliance-ac-2-1"));
        let parsed_yaml: Value = serde_yaml::from_str(&kyverno)
            .expect("generated Kyverno YAML must be valid YAML even with quotes/newlines");
        assert_eq!(
            parsed_yaml["metadata"]["annotations"]["policies.kyverno.io/description"],
            "line 1 \"quoted\" line 2"
        );
        assert_eq!(
            parsed_yaml["metadata"]["annotations"]["oscal.compliance/control-id"],
            "ac-2(1)"
        );

        let digit_rego = generate_rego_policy("1.1.1", "benchmark control");
        assert!(digit_rego.contains("package oscal.compliance.c_1_1_1"));
    }
}
