use serde::Serialize;
use serde_json::{json, Value};
use std::{fs, path::Path};

use crate::{
    document::parser::OscalDocument,
    error::{io_error, AppError, Result},
};

#[derive(Clone, Debug, Serialize)]
pub struct PolicyIngestReport {
    pub total_evaluated: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub generated_findings: usize,
    pub assessment_results_file: Option<String>,
}

pub fn ingest_policy_results(
    eval_log_path: &Path,
    title: &str,
    output_path: Option<&Path>,
) -> Result<(OscalDocument, PolicyIngestReport)> {
    let content = fs::read_to_string(eval_log_path).map_err(|e| io_error(eval_log_path, e))?;
    let eval_json: Value = serde_json::from_str(&content).map_err(|e| {
        AppError::Configuration(format!("Failed to parse policy evaluation JSON: {e}"))
    })?;

    let now = chrono::Utc::now().to_rfc3339();
    let ar_uuid = uuid::Uuid::new_v4().to_string();
    let result_uuid = uuid::Uuid::new_v4().to_string();

    let mut observations = Vec::new();
    let mut findings = Vec::new();
    let mut passed_count = 0;
    let mut failed_count = 0;

    // Scan OPA or Kyverno evaluations
    let items = extract_eval_items(&eval_json)?;
    for item in items {
        let obs_uuid = uuid::Uuid::new_v4().to_string();
        let cid = item.control_id;
        let is_pass = item.passed;
        let msg = item.message;

        observations.push(json!({
            "uuid": obs_uuid,
            "description": format!("Automated policy evaluation for {cid}: {msg}"),
            "methods": ["automated-policy-eval"],
            "types": ["policy-rule-check"],
            "collected": now,
            "relevant-evidence": [
                {
                    "description": format!("Policy evaluation output for {cid}")
                }
            ]
        }));

        if is_pass {
            passed_count += 1;
        } else {
            failed_count += 1;
            let find_uuid = uuid::Uuid::new_v4().to_string();
            findings.push(json!({
                "uuid": find_uuid,
                "title": format!("Non-compliant policy check for {cid}"),
                "description": msg,
                "target": {
                    "type": "control",
                    "target-id": cid,
                    "status": { "state": "unsatisfied" }
                },
                "related-observations": [
                    { "observation-uuid": obs_uuid }
                ],
                "props": [
                    { "name": "severity", "value": "high" },
                    { "name": "source", "value": "c2p-policy-engine" }
                ]
            }));
        }
    }

    let total_evaluated = passed_count + failed_count;

    let doc_json = json!({
        "assessment-results": {
            "uuid": ar_uuid,
            "metadata": {
                "title": title,
                "published": now,
                "last-modified": now,
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "import-ap": {
                "href": "assessment-plan.json"
            },
            "results": [
                {
                    "uuid": result_uuid,
                    "title": format!("{title} Execution Results"),
                    "description": "Continuous automated Compliance-to-Policy evaluation run",
                    "start": now,
                    "end": now,
                    "observations": observations,
                    "findings": findings
                }
            ]
        }
    });

    let doc_str = serde_json::to_string_pretty(&doc_json).map_err(|e| {
        AppError::Configuration(format!("Failed to format assessment-results: {e}"))
    })?;

    if let Some(out_p) = output_path {
        fs::write(out_p, &doc_str).map_err(|e| io_error(out_p, e))?;
    }

    let doc = OscalDocument::from_str(&doc_str, output_path.map(Path::to_path_buf))?;

    let report = PolicyIngestReport {
        total_evaluated,
        passed_checks: passed_count,
        failed_checks: failed_count,
        generated_findings: findings.len(),
        assessment_results_file: output_path.map(|p| p.display().to_string()),
    };

    Ok((doc, report))
}

struct EvalItem {
    control_id: String,
    passed: bool,
    message: String,
}

fn extract_eval_items(val: &Value) -> Result<Vec<EvalItem>> {
    let mut list = Vec::new();

    // Check array format: [{"control_id": "ac-1", "allowed": true, ...}]
    if let Some(arr) = val.as_array() {
        for item in arr {
            let cid = item
                .get("control_id")
                .or_else(|| item.get("control-id"))
                .or_else(|| item.get("rule"))
                .and_then(Value::as_str)
                .unwrap_or("general-control")
                .to_string();

            let passed = item
                .get("allowed")
                .or_else(|| item.get("passed"))
                .and_then(Value::as_bool)
                .unwrap_or_else(|| item.get("status").and_then(Value::as_str) == Some("pass"));

            let msg = item
                .get("message")
                .or_else(|| item.get("error"))
                .and_then(Value::as_str)
                .unwrap_or(if passed {
                    "Policy rule passed successfully"
                } else {
                    "Policy rule evaluation failed"
                })
                .to_string();

            list.push(EvalItem {
                control_id: cid,
                passed,
                message: msg,
            });
        }
    } else if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            let passed =
                v.get("allow")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| AppError::Negative {
                        class: "policy-ingest",
                        status: format!("Evaluation for '{k}' missing explicit 'allow' boolean"),
                    })?;
            let msg = if passed {
                "Policy rule passed".to_string()
            } else {
                v.get("violations")
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                    .and_then(Value::as_str)
                    .unwrap_or("Violation detected")
                    .to_string()
            };
            list.push(EvalItem {
                control_id: k.replace('_', "-"),
                passed,
                message: msg,
            });
        }
    }

    Ok(list)
}
