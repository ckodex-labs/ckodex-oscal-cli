use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

use crate::{document::parser::OscalDocument, error::Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ReconciliationVerdict {
    Aligned,
    DriftDetected,
    CriticalUnmitigated,
}

impl ReconciliationVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aligned => "ALIGNED",
            Self::DriftDetected => "DRIFT_DETECTED",
            Self::CriticalUnmitigated => "CRITICAL_UNMITIGATED",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ReconciliationReport {
    pub verdict: ReconciliationVerdict,
    pub component_reconciliation: Option<ComponentReconciliation>,
    pub finding_poam_reconciliation: Option<FindingPoamReconciliation>,
    pub action_items: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ComponentReconciliation {
    pub total_declared: usize,
    pub total_observed: usize,
    pub matching_components: Vec<String>,
    pub shadow_components: Vec<ObservedComponent>,
    pub phantom_components: Vec<String>,
    pub version_drifts: Vec<VersionDrift>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ObservedComponent {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionDrift {
    pub component_title: String,
    pub declared_version: String,
    pub observed_version: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct FindingPoamReconciliation {
    pub total_findings: usize,
    pub total_poam_items: usize,
    pub mitigated_findings: Vec<String>,
    pub unmitigated_findings: Vec<UnmitigatedFinding>,
    pub stale_poam_items: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnmitigatedFinding {
    pub finding_id: String,
    pub title: String,
    pub target_control: Option<String>,
    pub severity: String,
}

pub fn reconcile_compliance(
    ssp_doc: Option<&OscalDocument>,
    inventory_val: Option<&Value>,
    results_doc: Option<&OscalDocument>,
    poam_doc: Option<&OscalDocument>,
) -> Result<ReconciliationReport> {
    let mut action_items = Vec::new();
    let mut has_critical = false;
    let mut has_drift = false;

    // 1. Component & Inventory Reconciliation
    let comp_recon = if let (Some(ssp), Some(inv)) = (ssp_doc, inventory_val) {
        let recon = reconcile_components(ssp, inv, &mut action_items, &mut has_drift);
        Some(recon)
    } else {
        None
    };

    // 2. Finding & POA&M Reconciliation
    let finding_recon = if let (Some(results), Some(poam)) = (results_doc, poam_doc) {
        let recon = reconcile_findings_and_poam(
            results,
            poam,
            &mut action_items,
            &mut has_critical,
            &mut has_drift,
        );
        Some(recon)
    } else {
        None
    };

    let verdict = if has_critical {
        ReconciliationVerdict::CriticalUnmitigated
    } else if has_drift {
        ReconciliationVerdict::DriftDetected
    } else {
        ReconciliationVerdict::Aligned
    };

    Ok(ReconciliationReport {
        verdict,
        component_reconciliation: comp_recon,
        finding_poam_reconciliation: finding_recon,
        action_items,
    })
}

fn reconcile_components(
    ssp_doc: &OscalDocument,
    inv_val: &Value,
    action_items: &mut Vec<String>,
    has_drift: &mut bool,
) -> ComponentReconciliation {
    let mut declared_map: BTreeMap<String, Option<String>> = BTreeMap::new(); // name -> version

    if let Some(root) = ssp_doc.root_object() {
        let comps_opt = root
            .get("components")
            .or_else(|| {
                root.get("system-implementation")
                    .and_then(|si| si.get("components"))
            })
            .and_then(Value::as_array);

        if let Some(comps) = comps_opt {
            for c in comps {
                let title = c
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_lowercase();
                let ver = extract_version_from_component(c);
                if !title.is_empty() {
                    declared_map.insert(title, ver);
                }
            }
        }
    }

    let observed_list = extract_observed_components(inv_val);
    let mut observed_map: BTreeMap<String, Option<String>> = BTreeMap::new();
    for obs in &observed_list {
        observed_map.insert(obs.name.trim().to_lowercase(), obs.version.clone());
    }

    let mut matching_components = Vec::new();
    let mut shadow_components = Vec::new();
    let mut phantom_components = Vec::new();
    let mut version_drifts = Vec::new();

    for (decl_title, decl_ver) in &declared_map {
        if let Some(obs_ver) = observed_map.get(decl_title) {
            matching_components.push(decl_title.clone());
            if let (Some(dv), Some(ov)) = (decl_ver, obs_ver) {
                if dv != ov {
                    version_drifts.push(VersionDrift {
                        component_title: decl_title.clone(),
                        declared_version: dv.clone(),
                        observed_version: ov.clone(),
                    });
                    action_items.push(format!(
                        "Update SSP component '{decl_title}' version from {dv} to observed {ov}"
                    ));
                    *has_drift = true;
                }
            }
        } else {
            phantom_components.push(decl_title.clone());
            action_items.push(format!("Verify phantom component '{decl_title}' (declared in SSP but not observed in inventory)"));
            *has_drift = true;
        }
    }

    for obs in observed_list {
        let key = obs.name.trim().to_lowercase();
        if !declared_map.contains_key(&key) {
            action_items.push(format!(
                "Add shadow component '{}' to SSP system-implementation",
                obs.name
            ));
            shadow_components.push(obs);
            *has_drift = true;
        }
    }

    ComponentReconciliation {
        total_declared: declared_map.len(),
        total_observed: observed_map.len(),
        matching_components,
        shadow_components,
        phantom_components,
        version_drifts,
    }
}

fn extract_version_from_component(c: &Value) -> Option<String> {
    if let Some(props) = c.get("props").and_then(Value::as_array) {
        for p in props {
            if p.get("name").and_then(Value::as_str) == Some("version")
                || p.get("name").and_then(Value::as_str) == Some("software-version")
            {
                return p.get("value").and_then(Value::as_str).map(String::from);
            }
        }
    }
    None
}

fn extract_observed_components(inv_val: &Value) -> Vec<ObservedComponent> {
    let mut list = Vec::new();

    let items_opt = inv_val
        .as_array()
        .or_else(|| inv_val.get("components").and_then(Value::as_array))
        .or_else(|| inv_val.get("packages").and_then(Value::as_array));

    if let Some(items) = items_opt {
        for it in items {
            let name = it
                .get("name")
                .or_else(|| it.get("title"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();

            let version = it
                .get("version")
                .or_else(|| it.get("versionInfo"))
                .and_then(Value::as_str)
                .map(String::from);

            if !name.is_empty() {
                list.push(ObservedComponent {
                    name,
                    version,
                    source: "inventory".to_string(),
                });
            }
        }
    }

    list
}

fn reconcile_findings_and_poam(
    results_doc: &OscalDocument,
    poam_doc: &OscalDocument,
    action_items: &mut Vec<String>,
    has_critical: &mut bool,
    has_drift: &mut bool,
) -> FindingPoamReconciliation {
    let mut findings_map: BTreeMap<String, (String, Option<String>, String)> = BTreeMap::new(); // fid -> (title, target_control, severity)
    let mut poam_tracked_findings = HashSet::new();
    let mut poam_items_list = Vec::new();

    if let Some(root) = results_doc.root_object() {
        if let Some(results) = root.get("results").and_then(Value::as_array) {
            for res in results {
                if let Some(findings) = res.get("findings").and_then(Value::as_array) {
                    for f in findings {
                        let fid = f
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let title = f
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or("Finding")
                            .to_string();
                        let target_ctrl = f
                            .get("target")
                            .and_then(|t| t.get("target-id"))
                            .and_then(Value::as_str)
                            .map(String::from);
                        let severity = extract_finding_severity(f);
                        if !fid.is_empty() {
                            findings_map.insert(fid, (title, target_ctrl, severity));
                        }
                    }
                }
            }
        }
    }

    if let Some(root) = poam_doc.root_object() {
        if let Some(items) = root.get("poam-items").and_then(Value::as_array) {
            for item in items {
                let item_id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if !item_id.is_empty() {
                    poam_items_list.push(item_id.clone());
                }
                if let Some(rel_findings) = item.get("related-findings").and_then(Value::as_array) {
                    for rf in rel_findings {
                        if let Some(fid) = rf.get("finding-id").and_then(Value::as_str) {
                            poam_tracked_findings.insert(fid.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut mitigated_findings = Vec::new();
    let mut unmitigated_findings = Vec::new();

    for (fid, (title, target_ctrl, severity)) in &findings_map {
        if poam_tracked_findings.contains(fid) {
            mitigated_findings.push(fid.clone());
        } else {
            if severity == "critical" || severity == "high" {
                *has_critical = true;
            } else {
                *has_drift = true;
            }
            action_items.push(format!("CRITICAL: Open finding '{title}' ({fid}, severity: {severity}) has no tracking POA&M milestone"));
            unmitigated_findings.push(UnmitigatedFinding {
                finding_id: fid.clone(),
                title: title.clone(),
                target_control: target_ctrl.clone(),
                severity: severity.clone(),
            });
        }
    }

    let mut stale_poam_items = Vec::new();
    for p_fid in &poam_tracked_findings {
        if !findings_map.contains_key(p_fid) {
            stale_poam_items.push(p_fid.clone());
            action_items.push(format!("POA&M item references finding '{p_fid}' which is no longer active in assessment results"));
            *has_drift = true;
        }
    }

    FindingPoamReconciliation {
        total_findings: findings_map.len(),
        total_poam_items: poam_items_list.len(),
        mitigated_findings,
        unmitigated_findings,
        stale_poam_items,
    }
}

fn extract_finding_severity(f: &Value) -> String {
    if let Some(props) = f.get("props").and_then(Value::as_array) {
        for p in props {
            if p.get("name").and_then(Value::as_str) == Some("severity") {
                return p
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or("medium")
                    .to_lowercase();
            }
        }
    }
    "medium".to_string()
}
