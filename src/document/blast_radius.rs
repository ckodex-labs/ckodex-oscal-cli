use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

use crate::{document::parser::OscalDocument, error::Result};

#[derive(Clone, Debug, Serialize)]
pub struct BlastRadiusReport {
    pub target_id: String,
    pub target_kind: String,
    pub documents_analyzed: Vec<String>,
    pub direct_dependents: Vec<ImpactedNode>,
    pub transitive_dependents: Vec<ImpactedNode>,
    pub affected_components: Vec<String>,
    pub affected_findings: Vec<String>,
    pub affected_poam_items: Vec<String>,
    pub risk_exposure_score: f64,
    pub is_critical_path: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
pub struct ImpactedNode {
    pub id: String,
    pub kind: String,
    pub relation: String,
    pub title: String,
    pub document: String,
}

pub fn analyze_blast_radius(
    primary_doc: &OscalDocument,
    context_docs: &[OscalDocument],
    target_id: &str,
    max_depth: usize,
) -> Result<BlastRadiusReport> {
    let mut all_docs = vec![primary_doc];
    for d in context_docs {
        all_docs.push(d);
    }

    let mut documents_analyzed = Vec::new();
    for d in &all_docs {
        let name = d
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| d.kind.name().to_string());
        documents_analyzed.push(name);
    }

    let target_kind = infer_target_kind(target_id, &all_docs);

    let mut direct_dependents = Vec::new();
    let mut transitive_dependents = Vec::new();
    let mut visited_ids = HashSet::new();
    visited_ids.insert(target_id.to_string());

    let mut queue = vec![(target_id.to_string(), 0)];
    let mut affected_components = BTreeSet::new();
    let mut affected_findings = BTreeSet::new();
    let mut affected_poam_items = BTreeSet::new();

    while let Some((current_id, depth)) = queue.pop() {
        if depth >= max_depth {
            continue;
        }

        let dependents = find_dependents_for_id(&current_id, &all_docs);
        for dep in dependents {
            if dep.kind == "Component" {
                affected_components.insert(dep.id.clone());
            } else if dep.kind == "Finding" || dep.kind == "Observation" {
                affected_findings.insert(dep.id.clone());
            } else if dep.kind == "PoamItem" {
                affected_poam_items.insert(dep.id.clone());
            }

            if depth == 0 {
                if !direct_dependents.contains(&dep) {
                    direct_dependents.push(dep.clone());
                }
            } else if !transitive_dependents.contains(&dep) {
                transitive_dependents.push(dep.clone());
            }

            if !visited_ids.contains(&dep.id) {
                visited_ids.insert(dep.id.clone());
                queue.push((dep.id, depth + 1));
            }
        }
    }

    // Compute Risk Exposure Score (0.0 to 10.0)
    let component_weight = (affected_components.len() as f64 * 1.5).min(4.0);
    let finding_weight = (affected_findings.len() as f64 * 2.0).min(4.0);
    let poam_weight = (affected_poam_items.len() as f64 * 1.0).min(2.0);
    let raw_score = (component_weight + finding_weight + poam_weight).min(10.0);
    let risk_exposure_score = (raw_score * 10.0).round() / 10.0;
    let is_critical_path = !affected_findings.is_empty() || risk_exposure_score >= 6.0;

    Ok(BlastRadiusReport {
        target_id: target_id.to_string(),
        target_kind,
        documents_analyzed,
        direct_dependents,
        transitive_dependents,
        affected_components: affected_components.into_iter().collect(),
        affected_findings: affected_findings.into_iter().collect(),
        affected_poam_items: affected_poam_items.into_iter().collect(),
        risk_exposure_score,
        is_critical_path,
    })
}

fn infer_target_kind(target_id: &str, docs: &[&OscalDocument]) -> String {
    for doc in docs {
        if let Some(root) = doc.root_object() {
            // Check controls
            if let Some(ctrls) = root.get("controls").and_then(Value::as_array) {
                for c in ctrls {
                    if c.get("id").and_then(Value::as_str) == Some(target_id) {
                        return "Control".to_string();
                    }
                    if let Some(params) = c.get("params").and_then(Value::as_array) {
                        for p in params {
                            if p.get("id").and_then(Value::as_str) == Some(target_id) {
                                return "Parameter".to_string();
                            }
                        }
                    }
                }
            }
            // Check components
            if let Some(comps) = root.get("components").and_then(Value::as_array) {
                for comp in comps {
                    if comp.get("uuid").and_then(Value::as_str) == Some(target_id) {
                        return "Component".to_string();
                    }
                }
            }
        }
    }
    if target_id.contains("_prm_") || target_id.contains("-param") {
        "Parameter".to_string()
    } else if target_id.contains('-') && target_id.len() <= 10 {
        "Control".to_string()
    } else if uuid::Uuid::parse_str(target_id).is_ok() {
        "EntityUUID".to_string()
    } else {
        "Identifier".to_string()
    }
}

fn find_dependents_for_id(target_id: &str, docs: &[&OscalDocument]) -> Vec<ImpactedNode> {
    let mut list = Vec::new();
    for doc in docs {
        let doc_name = doc
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .unwrap_or(doc.kind.name())
            .to_string();

        if let Some(root) = doc.root_object() {
            scan_value_for_dependents(&doc.value, target_id, &doc_name, &mut list);
            // Specific model structural checks
            scan_ssp_dependents(root, target_id, &doc_name, &mut list);
            scan_profile_dependents(root, target_id, &doc_name, &mut list);
            scan_results_dependents(root, target_id, &doc_name, &mut list);
            scan_poam_dependents(root, target_id, &doc_name, &mut list);
        }
    }
    list
}

fn scan_ssp_dependents(
    root: &serde_json::Map<String, Value>,
    target_id: &str,
    doc_name: &str,
    list: &mut Vec<ImpactedNode>,
) {
    if let Some(ctrl_imp) = root
        .get("control-implementation")
        .and_then(Value::as_object)
    {
        if let Some(reqs) = ctrl_imp
            .get("implemented-requirements")
            .and_then(Value::as_array)
        {
            for req in reqs {
                let ctrl_id = req.get("control-id").and_then(Value::as_str).unwrap_or("");
                let is_match = ctrl_id == target_id;

                if is_match {
                    list.push(ImpactedNode {
                        id: ctrl_id.to_string(),
                        kind: "ImplementedRequirement".to_string(),
                        relation: "implements-control".to_string(),
                        title: format!("SSP Requirement for {ctrl_id}"),
                        document: doc_name.to_string(),
                    });
                }

                if let Some(by_comps) = req.get("by-components").and_then(Value::as_array) {
                    for bc in by_comps {
                        let comp_uuid = bc
                            .get("component-uuid")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        let desc = bc.get("description").and_then(Value::as_str).unwrap_or("");
                        if is_match || comp_uuid == target_id {
                            list.push(ImpactedNode {
                                id: comp_uuid.to_string(),
                                kind: "Component".to_string(),
                                relation: "implements-via-component".to_string(),
                                title: if desc.len() > 30 {
                                    format!("{}...", &desc[..27])
                                } else {
                                    desc.to_string()
                                },
                                document: doc_name.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

fn scan_profile_dependents(
    root: &serde_json::Map<String, Value>,
    target_id: &str,
    doc_name: &str,
    list: &mut Vec<ImpactedNode>,
) {
    if let Some(imports) = root.get("imports").and_then(Value::as_array) {
        for imp in imports {
            if let Some(inc_ctrls) = imp.get("include-controls").and_then(Value::as_array) {
                for inc in inc_ctrls {
                    if let Some(with_ids) = inc.get("with-ids").and_then(Value::as_array) {
                        for wid in with_ids {
                            if wid.as_str() == Some(target_id) {
                                list.push(ImpactedNode {
                                    id: target_id.to_string(),
                                    kind: "ProfileImport".to_string(),
                                    relation: "imports-control".to_string(),
                                    title: format!("Profile selects control {target_id}"),
                                    document: doc_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(modify) = root.get("modify").and_then(Value::as_object) {
        if let Some(set_params) = modify.get("set-parameters").and_then(Value::as_array) {
            for sp in set_params {
                let pid = sp.get("param-id").and_then(Value::as_str).unwrap_or("");
                let norm_target = target_id.replace('-', "_");
                let norm_pid = pid.replace('-', "_");
                if pid == target_id || norm_pid.starts_with(&norm_target) {
                    list.push(ImpactedNode {
                        id: pid.to_string(),
                        kind: "SetParameter".to_string(),
                        relation: "overrides-parameter".to_string(),
                        title: format!("Parameter Override {pid}"),
                        document: doc_name.to_string(),
                    });
                }
            }
        }
        if let Some(alters) = modify.get("alters").and_then(Value::as_array) {
            for alt in alters {
                let cid = alt.get("control-id").and_then(Value::as_str).unwrap_or("");
                if cid == target_id {
                    list.push(ImpactedNode {
                        id: cid.to_string(),
                        kind: "Alter".to_string(),
                        relation: "alters-control".to_string(),
                        title: format!("Profile Alter for {cid}"),
                        document: doc_name.to_string(),
                    });
                }
            }
        }
    }
}

fn scan_results_dependents(
    root: &serde_json::Map<String, Value>,
    target_id: &str,
    doc_name: &str,
    list: &mut Vec<ImpactedNode>,
) {
    if let Some(results) = root.get("results").and_then(Value::as_array) {
        for res in results {
            if let Some(findings) = res.get("findings").and_then(Value::as_array) {
                for f in findings {
                    let fid = f.get("id").and_then(Value::as_str).unwrap_or("");
                    let title = f.get("title").and_then(Value::as_str).unwrap_or("Finding");
                    let mut matched = fid == target_id;

                    if let Some(target) = f.get("target").and_then(Value::as_object) {
                        if target.get("target-id").and_then(Value::as_str) == Some(target_id) {
                            matched = true;
                        }
                    }

                    if matched {
                        list.push(ImpactedNode {
                            id: fid.to_string(),
                            kind: "Finding".to_string(),
                            relation: "evaluates-finding".to_string(),
                            title: title.to_string(),
                            document: doc_name.to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn scan_poam_dependents(
    root: &serde_json::Map<String, Value>,
    target_id: &str,
    doc_name: &str,
    list: &mut Vec<ImpactedNode>,
) {
    if let Some(items) = root.get("poam-items").and_then(Value::as_array) {
        for item in items {
            let item_id = item.get("id").and_then(Value::as_str).unwrap_or("");
            let title = item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("POA&M Item");
            let mut matched = item_id == target_id;

            if let Some(rel_findings) = item.get("related-findings").and_then(Value::as_array) {
                for rf in rel_findings {
                    if rf.get("finding-id").and_then(Value::as_str) == Some(target_id) {
                        matched = true;
                    }
                }
            }

            if matched {
                list.push(ImpactedNode {
                    id: item_id.to_string(),
                    kind: "PoamItem".to_string(),
                    relation: "remediates-finding".to_string(),
                    title: title.to_string(),
                    document: doc_name.to_string(),
                });
            }
        }
    }
}

fn scan_value_for_dependents(
    val: &Value,
    target_id: &str,
    doc_name: &str,
    list: &mut Vec<ImpactedNode>,
) {
    match val {
        Value::Object(obj) => {
            if let Some(id) = obj.get("id").and_then(Value::as_str) {
                if let Some(params) = obj.get("params").and_then(Value::as_array) {
                    for p in params {
                        if p.get("id").and_then(Value::as_str) == Some(target_id) {
                            let title = obj.get("title").and_then(Value::as_str).unwrap_or(id);
                            list.push(ImpactedNode {
                                id: id.to_string(),
                                kind: "Control".to_string(),
                                relation: "contains-parameter".to_string(),
                                title: title.to_string(),
                                document: doc_name.to_string(),
                            });
                        }
                    }
                }
            }
            for (_, v) in obj {
                scan_value_for_dependents(v, target_id, doc_name, list);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                scan_value_for_dependents(v, target_id, doc_name, list);
            }
        }
        _ => {}
    }
}
