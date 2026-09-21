use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    document::policy::evaluator::{PolicyEvaluationResult, RegorusEvaluator},
    error::{AppError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyRuleMeta {
    pub id: String,
    pub name: String,
    pub benchmark: String,
    pub target_controls: Vec<String>,
    pub severity: String,
    pub description: String,
    pub rego_source: String,
}

pub struct BuiltinRulepack;

impl BuiltinRulepack {
    pub fn get_rules() -> Vec<PolicyRuleMeta> {
        vec![
            PolicyRuleMeta {
                id: "cis-k8s-5.2.1".to_string(),
                name: "Disallow Privileged Containers".to_string(),
                benchmark: "CIS Kubernetes Benchmark v1.8 / FedRAMP High AC-6".to_string(),
                target_controls: vec!["ac-6".to_string(), "cm-7".to_string()],
                severity: "High".to_string(),
                description: "Privileged containers share all capabilities of the host kernel and must be blocked in production.".to_string(),
                rego_source: r#"
package oscal.k8s.privileged

deny contains msg if {
    container := input.spec.containers[_]
    container.securityContext.privileged == true
    msg := sprintf("Container %v has privileged: true", [container.name])
}
"#.to_string(),
            },
            PolicyRuleMeta {
                id: "cis-k8s-5.2.6".to_string(),
                name: "Require Read-Only Root Filesystem".to_string(),
                benchmark: "CIS Kubernetes Benchmark v1.8 / FedRAMP High SI-4".to_string(),
                target_controls: vec!["si-4".to_string(), "cm-7".to_string()],
                severity: "Medium".to_string(),
                description: "Containers must run with read-only root filesystems to prevent runtime binary tampering.".to_string(),
                rego_source: r#"
package oscal.k8s.readonly_root

deny contains msg if {
    container := input.spec.containers[_]
    not container.securityContext.readOnlyRootFilesystem == true
    msg := sprintf("Container %v does not set readOnlyRootFilesystem: true", [container.name])
}
"#.to_string(),
            },
            PolicyRuleMeta {
                id: "fedramp-ac-2".to_string(),
                name: "Disallow Root User Execution".to_string(),
                benchmark: "FedRAMP Rev 5 High / NIST SP 800-53 AC-2".to_string(),
                target_controls: vec!["ac-2".to_string(), "ia-2".to_string()],
                severity: "High".to_string(),
                description: "Containers must specify runAsNonRoot: true to enforce non-privileged execution context.".to_string(),
                rego_source: r#"
package oscal.fedramp.non_root

deny contains msg if {
    container := input.spec.containers[_]
    not container.securityContext.runAsNonRoot == true
    msg := sprintf("Container %v does not enforce runAsNonRoot: true", [container.name])
}
"#.to_string(),
            },
            PolicyRuleMeta {
                id: "itsg33-boundary-isolation".to_string(),
                name: "CCCS ITSG-33 Sovereign Boundary Isolation".to_string(),
                benchmark: "CCCS ITSG-33 PBMM / SC-7 Boundary Protection".to_string(),
                target_controls: vec!["sc-7".to_string(), "ac-4".to_string()],
                severity: "High".to_string(),
                description: "Enforce network boundary isolation and egress restriction policies for Protected B workloads.".to_string(),
                rego_source: r#"
package oscal.itsg33.boundary

deny contains msg if {
    not input.spec.ingress
    msg := "NetworkPolicy must define explicit ingress isolation rules"
}
"#.to_string(),
            },
        ]
    }

    pub fn evaluate_rule(rule_id: &str, input_resource: &Value) -> Result<PolicyEvaluationResult> {
        let rules = Self::get_rules();
        let rule = rules.iter().find(|r| r.id == rule_id).ok_or_else(|| {
            AppError::Configuration(format!("Rule '{rule_id}' not found in rulepack"))
        })?;

        let mut evaluator = RegorusEvaluator::new();
        evaluator.add_policy_str(&format!("rule_{rule_id}.rego"), &rule.rego_source)?;

        let pkg = rule
            .rego_source
            .lines()
            .find(|l| l.starts_with("package "))
            .map(|l| l.trim_start_matches("package ").trim())
            .unwrap_or("oscal");

        evaluator.evaluate_compliance_rule(pkg, input_resource.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rulepack_evaluation_deny_and_allow() {
        let bad_pod = serde_json::json!({
            "spec": {
                "containers": [
                    {
                        "name": "workload",
                        "securityContext": {
                            "privileged": true,
                            "runAsNonRoot": false
                        }
                    }
                ]
            }
        });

        let res = BuiltinRulepack::evaluate_rule("cis-k8s-5.2.1", &bad_pod).unwrap();
        assert!(!res.passed);
        assert!(!res.findings.is_empty());

        let good_pod = serde_json::json!({
            "spec": {
                "containers": [
                    {
                        "name": "workload",
                        "securityContext": {
                            "privileged": false,
                            "runAsNonRoot": true,
                            "readOnlyRootFilesystem": true
                        }
                    }
                ]
            }
        });

        let res_good = BuiltinRulepack::evaluate_rule("cis-k8s-5.2.1", &good_pod).unwrap();
        assert!(res_good.passed);
        assert!(res_good.findings.is_empty());
    }
}
