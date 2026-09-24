use serde::Serialize;
use serde_json::{Map, Value, json};
use std::{fs, path::Path};

use crate::{
    document::{
        k8s::client::KubeClusterClient, parser::OscalDocument, policy::evaluator::RegorusEvaluator,
        schema::DocumentKind,
    },
    error::{AppError, Result, io_error},
};

#[derive(Clone, Debug, Serialize)]
pub struct KubeAuditReport {
    pub cluster_connected: bool,
    pub target_namespace: String,
    pub resources_evaluated: usize,
    pub total_findings: usize,
    pub satisfied_controls: usize,
    pub unsatisfied_controls: usize,
    pub output_file: Option<String>,
}

pub struct KubeAuditor {
    client: KubeClusterClient,
    evaluator: RegorusEvaluator,
}

impl KubeAuditor {
    pub fn new(client: KubeClusterClient) -> Self {
        let evaluator = RegorusEvaluator::new();
        Self { client, evaluator }
    }

    pub fn load_policy_rules(&mut self, rego_code: &str) -> Result<()> {
        self.evaluator
            .add_policy_str("k8s_oscal_rules.rego", rego_code)
    }

    pub fn load_policy_dir(&mut self, dir_path: &Path) -> Result<usize> {
        self.evaluator.add_policy_dir(dir_path)
    }

    pub async fn audit_cluster(
        &mut self,
        namespace: Option<&str>,
        custom_rules_dir: Option<&Path>,
        output_path: Option<&Path>,
    ) -> Result<(OscalDocument, KubeAuditReport)> {
        let target_ns = namespace.ok_or_else(|| {
            AppError::Configuration("Kubernetes target namespace is required".to_owned())
        })?;

        // Load built-in NIST SP 800-53 r5 Kubernetes compliance Rego rules if no custom dir
        if let Some(dir) = custom_rules_dir {
            let _ = self.evaluator.add_policy_dir(dir)?;
        } else {
            let default_rules = r#"
                package oscal.k8s

                # AC-6: Least Privilege (Disallow Privilege Escalation)
                deny contains msg if {
                    container := input.spec.containers[_]
                    container.securityContext.allowPrivilegeEscalation == true
                    msg := sprintf("Container '%v' in pod '%v' allows privilege escalation, violating AC-6", [container.name, input.metadata.name])
                }

                # CM-7: Least Functionality (Enforce Read-Only Root Filesystem)
                deny contains msg if {
                    container := input.spec.containers[_]
                    not container.securityContext.readOnlyRootFilesystem
                    msg := sprintf("Container '%v' in pod '%v' does not enforce read-only root filesystem, violating CM-7", [container.name, input.metadata.name])
                }

                # AC-2 / IA-2: Non-Root Execution
                deny contains msg if {
                    container := input.spec.containers[_]
                    container.securityContext.runAsNonRoot != true
                    msg := sprintf("Container '%v' in pod '%v' runs as root or unspecified user, violating AC-2/IA-2", [container.name, input.metadata.name])
                }
            "#;
            self.evaluator
                .add_policy_str("default_k8s_rules.rego", default_rules)?;
        }

        let pods = self.client.list_pods(Some(target_ns)).await?;
        let resources_evaluated = pods.len();

        let mut findings = Vec::new();
        let mut satisfied_controls = 0;
        let mut unsatisfied_controls = 0;

        for pod in &pods {
            let pod_name = pod
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");

            // Evaluate AC-6
            let ac6_res = self
                .evaluator
                .evaluate_compliance_rule("oscal.k8s", pod.clone())?;
            if ac6_res.passed {
                satisfied_controls += 1;
            } else {
                unsatisfied_controls += 1;
                for f in ac6_res.findings {
                    findings.push(json!({
                        "id": format!("finding-ac6-{}", uuid::Uuid::new_v4()),
                        "title": "Privilege Escalation Violation",
                        "description": f,
                        "target": {
                            "type": "pod",
                            "name": pod_name,
                            "target-id": format!("k8s:pod:{target_ns}/{pod_name}")
                        },
                        "related-controls": ["ac-6"],
                        "status": "not-satisfied"
                    }));
                }
            }
        }

        let total_findings = findings.len();

        // Construct official OSCAL Assessment Results document
        let doc_uuid = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let mut metadata = Map::new();
        metadata.insert(
            "title".to_string(),
            json!(format!("Kubernetes Live Cluster Assessment ({target_ns})")),
        );
        metadata.insert("published".to_string(), json!(now));
        metadata.insert("last-modified".to_string(), json!(now));
        metadata.insert("version".to_string(), json!("1.0.0"));
        metadata.insert("oscal-version".to_string(), json!("1.2.3"));

        let mut result_entry = Map::new();
        result_entry.insert("uuid".to_string(), json!(uuid::Uuid::new_v4().to_string()));
        result_entry.insert(
            "title".to_string(),
            json!(format!("Mizan Regorus In-Process Audit: {target_ns}")),
        );
        result_entry.insert("start".to_string(), json!(now));
        result_entry.insert("findings".to_string(), Value::Array(findings));

        let mut root_obj = Map::new();
        root_obj.insert("uuid".to_string(), json!(doc_uuid));
        root_obj.insert("metadata".to_string(), Value::Object(metadata));
        root_obj.insert("results".to_string(), json!([result_entry]));

        let mut doc_json = Map::new();
        doc_json.insert(
            DocumentKind::AssessmentResults.root_key().to_string(),
            Value::Object(root_obj),
        );

        let doc = OscalDocument::from_value(
            Value::Object(doc_json),
            output_path.map(|p| p.to_path_buf()),
        )?;

        if let Some(out_p) = output_path {
            let json_str = serde_json::to_string_pretty(&doc.value)
                .map_err(|err| AppError::Configuration(err.to_string()))?;
            fs::write(out_p, &json_str).map_err(|err| io_error(out_p, err))?;
        }

        let report = KubeAuditReport {
            cluster_connected: self.client.is_connected(),
            target_namespace: target_ns.to_string(),
            resources_evaluated,
            total_findings,
            satisfied_controls,
            unsatisfied_controls,
            output_file: output_path.map(|p| p.to_string_lossy().to_string()),
        };

        Ok((doc, report))
    }
}
