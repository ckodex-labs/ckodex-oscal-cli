use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

use crate::{
    document::parser::OscalDocument,
    error::{AppError, Result},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum FedrampBaseline {
    Low,
    Moderate,
    High,
}

impl FedrampBaseline {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "low" => Some(Self::Low),
            "moderate" | "mod" => Some(Self::Moderate),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Moderate => "Moderate",
            Self::High => "High",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FedrampValidationReport {
    pub file: String,
    pub kind: String,
    pub baseline: String,
    pub is_compliant: bool,
    pub total_rules_checked: usize,
    pub passed_rules: usize,
    pub failed_rules: usize,
    pub findings: Vec<FedrampRuleFinding>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FedrampRuleFinding {
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub detail: String,
}

pub fn validate_fedramp(
    doc: &OscalDocument,
    baseline: FedrampBaseline,
) -> Result<FedrampValidationReport> {
    let root_obj = doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Missing root object".to_owned()))?;

    let mut findings = Vec::new();
    let mut total_rules: usize = 0;

    // Rule 1: FIPS 199 Impact Level Check
    total_rules += 1;
    if let Some(sys_char) = root_obj.get("system-characteristics") {
        let sec_level = sys_char
            .get("security-sensitivity-level")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase();

        let expected_level = match baseline {
            FedrampBaseline::Low => "low",
            FedrampBaseline::Moderate => "moderate",
            FedrampBaseline::High => "high",
        };

        if sec_level != expected_level && !sec_level.contains(expected_level) {
            findings.push(FedrampRuleFinding {
                rule_id: "FEDRAMP-SEC-01".to_string(),
                title: "Security Sensitivity Level Mismatch".to_string(),
                severity: "high".to_string(),
                detail: format!(
                    "System characteristics security-sensitivity-level is '{sec_level}', expected '{expected_level}' for FedRAMP {} baseline",
                    baseline.as_str()
                ),
            });
        }
    }

    // Rule 2: Mandatory FedRAMP Roles
    total_rules += 1;
    let mut roles_set = HashSet::new();
    if let Some(meta) = root_obj.get("metadata")
        && let Some(roles) = meta.get("roles").and_then(Value::as_array)
    {
        for r in roles {
            if let Some(id) = r.get("id").and_then(Value::as_str) {
                roles_set.insert(id.to_lowercase());
            }
        }
    }
    let required_roles = ["system-owner", "isso"];
    for req_role in &required_roles {
        if !roles_set.contains(*req_role) {
            findings.push(FedrampRuleFinding {
                rule_id: "FEDRAMP-ROLE-01".to_string(),
                title: format!("Missing Mandatory Role: {req_role}"),
                severity: "medium".to_string(),
                detail: format!(
                    "FedRAMP requires the '{req_role}' role defined in document metadata"
                ),
            });
        }
    }

    // Rule 3: Implemented Requirements Coverage
    total_rules += 1;
    let mut covered_controls = HashSet::new();
    if let Some(ctrl_imp) = root_obj.get("control-implementation")
        && let Some(reqs) = ctrl_imp
            .get("implemented-requirements")
            .and_then(Value::as_array)
    {
        for req in reqs {
            if let Some(cid) = req.get("control-id").and_then(Value::as_str) {
                covered_controls.insert(cid.to_lowercase());
            }
        }
    }

    let core_fedramp_controls = ["ac-1", "ac-2", "au-1", "ia-1", "sc-1"];
    for ctrl_id in &core_fedramp_controls {
        if !covered_controls.contains(*ctrl_id) {
            findings.push(FedrampRuleFinding {
                rule_id: "FEDRAMP-CTRL-01".to_string(),
                title: format!("Unimplemented Core Control: {ctrl_id}"),
                severity: "high".to_string(),
                detail: format!(
                    "FedRAMP {} baseline requires explicit implementation for core control '{ctrl_id}'",
                    baseline.as_str()
                ),
            });
        }
    }

    // Rule 4: System Information Categorization
    total_rules += 1;
    if let Some(sys_char) = root_obj.get("system-characteristics") {
        let has_inf_types = sys_char
            .get("system-information")
            .and_then(|si| si.get("information-types"))
            .and_then(Value::as_array)
            .is_some_and(|arr| !arr.is_empty());

        if !has_inf_types {
            findings.push(FedrampRuleFinding {
                rule_id: "FEDRAMP-CAT-01".to_string(),
                title: "Missing Information Categorization".to_string(),
                severity: "high".to_string(),
                detail: "FedRAMP requires NIST SP 800-60 information types categorized with FIPS 199 base impacts".to_string(),
            });
        }
    }

    let failed_rules = findings.len();
    let passed_rules = total_rules.saturating_sub(failed_rules);
    let is_compliant = findings.is_empty();

    Ok(FedrampValidationReport {
        file: doc
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        kind: doc.kind.name().to_string(),
        baseline: baseline.as_str().to_string(),
        is_compliant,
        total_rules_checked: total_rules,
        passed_rules,
        failed_rules,
        findings,
    })
}
