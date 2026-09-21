use regorus::{Engine, Value as RegoValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

use crate::error::{io_error, AppError, Result};

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
        let content = fs::read_to_string(path).map_err(|err| io_error(path, err))?;
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
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rego") {
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
        self.set_input_json(input)?;
        let deny_query = format!("data.{package}.deny");
        let deny_result = self.eval_query(&deny_query);

        let mut findings = Vec::new();
        let mut passed = true;

        if let Ok(Value::Object(map)) = &deny_result {
            if let Some(result_arr) = map.get("result").and_then(Value::as_array) {
                for binding in result_arr {
                    if let Some(expressions) = binding.get("expressions").and_then(Value::as_array)
                    {
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
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(PolicyEvaluationResult {
            policy_file: package.to_string(),
            query: deny_query,
            passed,
            findings,
            raw_output: deny_result.unwrap_or(Value::Null),
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
    }
}
