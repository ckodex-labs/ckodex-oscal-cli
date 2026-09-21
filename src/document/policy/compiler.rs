use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path};

use crate::{
    document::parser::OscalDocument,
    error::{io_error, AppError, Result},
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
            let rego_content = generate_rego_policy(cid, desc);
            let file_name = format!("{}.rego", cid.replace('-', "_"));
            let file_path = out_dir.join(&file_name);
            fs::write(&file_path, &rego_content).map_err(|e| io_error(&file_path, e))?;
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
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest_val).unwrap(),
        )
        .map_err(|e| io_error(&manifest_path, e))?;
        generated_files.push("policy_manifest.json".to_string());
    }

    // Generate Kyverno ClusterPolicies
    if target == PolicyTarget::Kyverno || target == PolicyTarget::All {
        for (cid, desc) in &control_rules {
            let kyverno_yaml = generate_kyverno_policy(cid, desc);
            let file_name = format!("kyverno_{}.yaml", cid.replace('_', "-"));
            let file_path = out_dir.join(&file_name);
            fs::write(&file_path, &kyverno_yaml).map_err(|e| io_error(&file_path, e))?;
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

fn generate_rego_policy(cid: &str, desc: &str) -> String {
    let pkg_name = cid.replace('-', "_");
    format!(
        r#"# METADATA
# title: OSCAL Compliance Rule for {cid}
# description: {desc}
# custom:
#   control_id: "{cid}"
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
    msg := "Control {cid} enforcement is disabled in target configuration"
}}

violations contains msg if {{
    input.compliance.{pkg_name}.status == "unsatisfied"
    msg := "Control {cid} status is marked unsatisfied"
}}
"#
    )
}

fn generate_kyverno_policy(cid: &str, desc: &str) -> String {
    let policy_name = format!("oscal-compliance-{}", cid.to_lowercase().replace('_', "-"));
    format!(
        r#"apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: {policy_name}
  annotations:
    policies.kyverno.io/title: "OSCAL Control {cid}"
    policies.kyverno.io/category: "Compliance & Security"
    policies.kyverno.io/description: "{desc}"
    oscal.compliance/control-id: "{cid}"
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
        message: "Resource must adhere to OSCAL control {cid} baseline security constraints."
        pattern:
          spec:
            template:
              spec:
                =(securityContext):
                  =(runAsNonRoot): true
"#
    )
}
