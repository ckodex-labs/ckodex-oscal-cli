use regorus::{Engine, Value as RegoValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

use crate::error::{AppError, Result, io_error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub policy_file: String,
    pub query: String,
    pub passed: bool,
    pub findings: Vec<String>,
    pub raw_output: Value,
}

pub struct RegorusEvaluator {
    engine: Engine,
}

impl Default for RegorusEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl RegorusEvaluator {
    pub fn new() -> Self {
        let engine = Engine::new();
        Self { engine }
    }

    pub fn add_policy_str(&mut self, filename: &str, policy: &str) -> Result<()> {
        self.engine
            .add_policy(filename.to_string(), policy.to_string())
            .map_err(|err| {
                AppError::Configuration(format!("Failed to parse Rego policy: {err}"))
            })?;
        Ok(())
    }

    pub fn add_policy_file(&mut self, path: &Path) -> Result<()> {
        let content = crate::document::io_safe::read_bounded_utf8_file(
            path,
            crate::document::io_safe::MAX_OSCAL_FILE_BYTES,
        )?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("policy.rego");
        self.add_policy_str(filename, &content)
    }

    pub fn add_policy_dir(&mut self, dir_path: &Path) -> Result<usize> {
        let mut count = 0;
        if dir_path.is_file() {
            self.add_policy_file(dir_path)?;
            return Ok(1);
        }
        let entries = fs::read_dir(dir_path).map_err(|err| io_error(dir_path, err))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = fs::symlink_metadata(&path)
                && !meta.file_type().is_symlink()
                && meta.file_type().is_file()
                && path.extension().and_then(|ext| ext.to_str()) == Some("rego")
            {
                self.add_policy_file(&path)?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn set_input_json(&mut self, input: Value) -> Result<()> {
        let json_str = serde_json::to_string(&input).map_err(|err| {
            AppError::Configuration(format!("Failed to serialize input JSON: {err}"))
        })?;
        let rego_val = RegoValue::from_json_str(&json_str).map_err(|err| {
            AppError::Configuration(format!("Failed to convert input to Rego value: {err}"))
        })?;
        self.engine.set_input(rego_val);
        Ok(())
    }

    pub fn set_data_json(&mut self, data: Value) -> Result<()> {
        let json_str = serde_json::to_string(&data).map_err(|err| {
            AppError::Configuration(format!("Failed to serialize data JSON: {err}"))
        })?;
        let rego_val = RegoValue::from_json_str(&json_str).map_err(|err| {
            AppError::Configuration(format!("Failed to convert data to Rego value: {err}"))
        })?;
        self.engine.add_data(rego_val).map_err(|err| {
            AppError::Configuration(format!("Failed to add data to Rego engine: {err}"))
        })?;
        Ok(())
    }

    pub fn eval_query(&mut self, query: &str) -> Result<Value> {
        let results = self
            .engine
            .eval_query(query.to_string(), false)
            .map_err(|err| {
                AppError::Configuration(format!("Rego query '{query}' evaluation failed: {err}"))
            })?;
        let json_str = serde_json::to_string(&results).map_err(|err| {
            AppError::Configuration(format!("Failed to serialize evaluation result: {err}"))
        })?;
        serde_json::from_str(&json_str).map_err(|err| {
            AppError::Configuration(format!("Failed to deserialize evaluation JSON: {err}"))
        })
    }

    pub fn evaluate_compliance_rule(
        &mut self,
        package: &str,
        input: Value,
    ) -> Result<PolicyEvaluationResult> {
        let pkg_trimmed = package.trim();
        if pkg_trimmed.is_empty() {
            return Err(AppError::InvalidArgument(
                "Rego query package must not be empty".to_string(),
            ));
        }
        let pkg_clean = pkg_trimmed.strip_prefix("data.").unwrap_or(pkg_trimmed);

        self.set_input_json(input)?;
        let deny_query = format!("data.{pkg_clean}.deny");
        // Propagate syntax or engine evaluation errors directly instead of silently passing
        let deny_val = self.eval_query(&deny_query)?;

        let mut findings = Vec::new();
        let mut passed = true;
        let mut effective_query = deny_query;
        let mut raw_output = deny_val.clone();

        let mut deny_defined = false;
        if let Value::Object(map) = &deny_val
            && let Some(result_arr) = map.get("result").and_then(Value::as_array)
        {
            deny_defined = !result_arr.is_empty();
            for binding in result_arr {
                if let Some(expressions) = binding.get("expressions").and_then(Value::as_array) {
                    for expr in expressions {
                        if let Some(value) = expr.get("value") {
                            if let Some(items) = value.as_array() {
                                for item in items {
                                    if let Some(msg) = item.as_str() {
                                        findings.push(msg.to_string());
                                        passed = false;
                                    }
                                }
                            } else if let Some(msg) = value.as_str() {
                                findings.push(msg.to_string());
                                passed = false;
                            } else if value == &Value::Bool(true) {
                                findings.push(format!("Policy deny rule matched in {pkg_clean}"));
                                passed = false;
                            }
                        }
                    }
                }
            }
        }

        // Support allow-only Rego packages (where `deny` is undefined)
        if !deny_defined {
            let allow_query = format!("data.{pkg_clean}.allow");
            let allow_val = self.eval_query(&allow_query)?;
            if let Value::Object(map) = &allow_val
                && let Some(result_arr) = map.get("result").and_then(Value::as_array)
                && !result_arr.is_empty()
            {
                effective_query = allow_query;
                raw_output = allow_val.clone();
                for binding in result_arr {
                    if let Some(expressions) = binding.get("expressions").and_then(Value::as_array)
                    {
                        for expr in expressions {
                            if expr.get("value") == Some(&Value::Bool(false)) {
                                passed = false;
                                // Collect optional `violations` set messages if present
                                if let Ok(v_val) =
                                    self.eval_query(&format!("data.{pkg_clean}.violations"))
                                    && let Some(v_arr) =
                                        v_val.get("result").and_then(Value::as_array)
                                {
                                    for vb in v_arr {
                                        if let Some(vex) =
                                            vb.get("expressions").and_then(Value::as_array)
                                        {
                                            for ve in vex {
                                                if let Some(items) =
                                                    ve.get("value").and_then(Value::as_array)
                                                {
                                                    for item in items {
                                                        if let Some(msg) = item.as_str() {
                                                            findings.push(msg.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if findings.is_empty() {
                                    findings.push(format!(
                                        "Policy allow rule evaluated to false in {pkg_clean}"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(PolicyEvaluationResult {
            policy_file: package.to_string(),
            query: effective_query,
            passed,
            findings,
            raw_output,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_regorus_evaluator_deny_rule() {
        let mut evaluator = RegorusEvaluator::new();
        let rego_code = r#"
            package oscal.ac_2

            default allow = false

            allow if {
                input.user.role == "admin"
            }

            deny contains msg if {
                input.user.privileged == true
                not input.user.mfa_enabled
                msg := "Privileged user lacks MFA required by AC-2"
            }
        "#;

        evaluator
            .add_policy_str("ac_2.rego", rego_code)
            .expect("should add policy");

        // Failing test input (privileged without MFA)
        let bad_input = json!({
            "user": {
                "role": "engineer",
                "privileged": true,
                "mfa_enabled": false
            }
        });

        let result = evaluator
            .evaluate_compliance_rule("oscal.ac_2", bad_input)
            .expect("evaluation should succeed");

        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].contains("Privileged user lacks MFA"));

        // Passing test input (privileged with MFA)
        let good_input = json!({
            "user": {
                "role": "admin",
                "privileged": true,
                "mfa_enabled": true
            }
        });

        let result_good = evaluator
            .evaluate_compliance_rule("oscal.ac_2", good_input)
            .expect("evaluation should succeed");

        assert!(result_good.passed);
        assert!(result_good.findings.is_empty());

        // Broken query syntax must return Err rather than silently passing
        assert!(
            evaluator
                .evaluate_compliance_rule("oscal..broken!!syntax", json!({}))
                .is_err(),
            "malformed Rego query must fail closed with Err"
        );
    }

    #[test]
    fn test_regorus_evaluator_allow_only_policy() {
        let mut evaluator = RegorusEvaluator::new();
        let rego_code = r#"
            package oscal.allow_only

            default allow = false
            allow if {
                input.approved == true
            }
        "#;
        evaluator
            .add_policy_str("allow_only.rego", rego_code)
            .unwrap();

        let denied = evaluator
            .evaluate_compliance_rule("oscal.allow_only", json!({ "approved": false }))
            .unwrap();
        assert!(!denied.passed);
        assert!(!denied.findings.is_empty());

        let allowed = evaluator
            .evaluate_compliance_rule("oscal.allow_only", json!({ "approved": true }))
            .unwrap();
        assert!(allowed.passed);
    }
}
