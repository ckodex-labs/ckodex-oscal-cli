use serde::Serialize;
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

use crate::{
    document::parser::OscalDocument,
    error::{io_error, AppError, Result},
};

#[derive(Clone, Debug, Serialize)]
pub struct DedupReport {
    pub file: Option<String>,
    pub kind: String,
    pub controls_deduplicated: usize,
    pub components_deduplicated: usize,
    pub parties_deduplicated: usize,
    pub resources_deduplicated: usize,
    pub total_duplicates_removed: usize,
    pub modifications: Vec<String>,
}

pub fn deduplicate_document(
    doc: &OscalDocument,
    output_path: Option<&Path>,
) -> Result<(OscalDocument, DedupReport)> {
    let mut cloned_val = doc.value.clone();
    let root_key = doc.kind.root_key();

    let mut modifications = Vec::new();
    let mut controls_dedup = 0;
    let mut components_dedup = 0;
    let mut parties_dedup = 0;
    let mut resources_dedup = 0;
    let mut resource_remap = HashMap::new();

    if let Some(root_obj) = cloned_val.get_mut(root_key).and_then(Value::as_object_mut) {
        // 1. Deduplicate Metadata Parties
        if let Some(meta) = root_obj.get_mut("metadata").and_then(Value::as_object_mut) {
            if let Some(parties) = meta.get_mut("parties").and_then(Value::as_array_mut) {
                let (cleaned_parties, party_remap, count) =
                    dedup_parties(parties, &mut modifications);
                *parties = cleaned_parties;
                parties_dedup = count;

                // Re-map party-uuids in responsible-parties
                if let Some(resp_parties) = meta
                    .get_mut("responsible-parties")
                    .and_then(Value::as_array_mut)
                {
                    remap_responsible_parties(resp_parties, &party_remap);
                }
            }
        }

        // 2. Deduplicate Back-Matter Resources
        if let Some(back_matter) = root_obj
            .get_mut("back-matter")
            .and_then(Value::as_object_mut)
        {
            if let Some(resources) = back_matter
                .get_mut("resources")
                .and_then(Value::as_array_mut)
            {
                let (cleaned_resources, remap, count) =
                    dedup_resources(resources, &mut modifications);
                *resources = cleaned_resources;
                resources_dedup = count;
                resource_remap = remap;
            }
        }

        // 3. Deduplicate Controls in Catalog / Group
        if let Some(controls) = root_obj.get_mut("controls").and_then(Value::as_array_mut) {
            let (cleaned_ctrls, count) = dedup_control_array(controls, &mut modifications);
            *controls = cleaned_ctrls;
            controls_dedup += count;
        }

        if let Some(groups) = root_obj.get_mut("groups").and_then(Value::as_array_mut) {
            for grp in groups.iter_mut() {
                dedup_group_controls(grp, &mut controls_dedup, &mut modifications);
            }
        }

        // 4. Deduplicate Components in System-Implementation / Components
        let mut comp_remap = HashMap::new();
        if let Some(comps) = root_obj.get_mut("components").and_then(Value::as_array_mut) {
            let (cleaned_comps, remap, count) = dedup_components(comps, &mut modifications);
            *comps = cleaned_comps;
            components_dedup = count;
            comp_remap = remap;
        } else if let Some(sys_imp) = root_obj
            .get_mut("system-implementation")
            .and_then(Value::as_object_mut)
        {
            if let Some(comps) = sys_imp.get_mut("components").and_then(Value::as_array_mut) {
                let (cleaned_comps, remap, count) = dedup_components(comps, &mut modifications);
                *comps = cleaned_comps;
                components_dedup = count;
                comp_remap = remap;
            }
        }

        if !comp_remap.is_empty() {
            remap_component_uuids(root_obj, &comp_remap);
        }
    }

    if !resource_remap.is_empty() {
        remap_resource_links(&mut cloned_val, &resource_remap);
    }

    let total_duplicates_removed =
        controls_dedup + components_dedup + parties_dedup + resources_dedup;

    let serialized = serde_json::to_string_pretty(&cloned_val).map_err(|e| {
        AppError::Configuration(format!("Failed to serialize deduplicated document: {e}"))
    })?;

    if let Some(out_p) = output_path {
        fs::write(out_p, &serialized).map_err(|e| io_error(out_p, e))?;
    }

    let new_doc = OscalDocument::from_str(&serialized, output_path.map(Path::to_path_buf))?;

    let report = DedupReport {
        file: doc.path.as_ref().map(|p| p.display().to_string()),
        kind: doc.kind.name().to_string(),
        controls_deduplicated: controls_dedup,
        components_deduplicated: components_dedup,
        parties_deduplicated: parties_dedup,
        resources_deduplicated: resources_dedup,
        total_duplicates_removed,
        modifications,
    };

    Ok((new_doc, report))
}

fn dedup_parties(
    parties: &[Value],
    modifications: &mut Vec<String>,
) -> (Vec<Value>, HashMap<String, String>, usize) {
    let mut seen_parties: HashMap<String, (String, Value)> = HashMap::new(); // key -> (canonical_uuid, party_val)
    let mut party_remap = HashMap::new();
    let mut count = 0;

    for p in parties {
        let uuid = p
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let name = p.get("name").and_then(Value::as_str).unwrap_or("");
        let email = p
            .get("email-addresses")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(Value::as_str)
            .unwrap_or("");

        let key = if !email.is_empty() {
            format!("email:{email}")
        } else {
            format!("name:{}", name.trim().to_lowercase())
        };

        if let Some((canon_uuid, _)) = seen_parties.get(&key) {
            party_remap.insert(uuid.clone(), canon_uuid.clone());
            modifications.push(format!(
                "Merged duplicate party '{name}' (UUID {uuid} -> {canon_uuid})"
            ));
            count += 1;
        } else {
            seen_parties.insert(key, (uuid.clone(), p.clone()));
        }
    }

    let cleaned: Vec<Value> = seen_parties.into_values().map(|(_, val)| val).collect();
    (cleaned, party_remap, count)
}

fn remap_responsible_parties(resp_parties: &mut [Value], party_remap: &HashMap<String, String>) {
    for rp in resp_parties {
        if let Some(party_uuids) = rp.get_mut("party-uuids").and_then(Value::as_array_mut) {
            for pu in party_uuids.iter_mut() {
                if let Some(u_str) = pu.as_str() {
                    if let Some(target) = party_remap.get(u_str) {
                        *pu = json!(target);
                    }
                }
            }
        }
    }
}

fn dedup_resources(
    resources: &[Value],
    modifications: &mut Vec<String>,
) -> (Vec<Value>, HashMap<String, String>, usize) {
    let mut seen: HashMap<String, (String, Value)> = HashMap::new();
    let mut remap = HashMap::new();
    let mut count = 0;

    for res in resources {
        let uuid = res
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let title = res.get("title").and_then(Value::as_str).unwrap_or("");
        let href = res
            .get("rlinks")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|r| r.get("href"))
            .and_then(Value::as_str)
            .unwrap_or(title);

        let key = href.trim().to_lowercase();
        if let Some((canon_uuid, _)) = seen.get(&key) {
            remap.insert(uuid.clone(), canon_uuid.clone());
            modifications.push(format!(
                "Merged duplicate resource '{title}' (UUID {uuid} -> {canon_uuid})"
            ));
            count += 1;
        } else {
            seen.insert(key, (uuid.clone(), res.clone()));
        }
    }

    let cleaned: Vec<Value> = seen.into_values().map(|(_, val)| val).collect();
    (cleaned, remap, count)
}

fn remap_resource_links(val: &mut Value, resource_remap: &HashMap<String, String>) {
    match val {
        Value::Object(obj) => {
            if let Some(href) = obj.get_mut("href") {
                if let Some(href_str) = href.as_str() {
                    if let Some(uuid_part) = href_str.strip_prefix('#') {
                        if let Some(canonical) = resource_remap.get(uuid_part) {
                            *href = json!(format!("#{canonical}"));
                        }
                    }
                }
            }
            for (_, v) in obj {
                remap_resource_links(v, resource_remap);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                remap_resource_links(v, resource_remap);
            }
        }
        _ => {}
    }
}

fn dedup_components(
    comps: &[Value],
    modifications: &mut Vec<String>,
) -> (Vec<Value>, HashMap<String, String>, usize) {
    let mut seen: HashMap<String, (String, Value)> = HashMap::new();
    let mut remap = HashMap::new();
    let mut count = 0;

    for comp in comps {
        let uuid = comp
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let title = comp.get("title").and_then(Value::as_str).unwrap_or("");
        let ctype = comp.get("type").and_then(Value::as_str).unwrap_or("");

        let key = format!("{}:{}", ctype, title.trim().to_lowercase());
        if let Some((canon_uuid, _)) = seen.get(&key) {
            remap.insert(uuid.clone(), canon_uuid.clone());
            modifications.push(format!(
                "Merged duplicate component '{title}' ({ctype}) (UUID {uuid} -> {canon_uuid})"
            ));
            count += 1;
        } else {
            seen.insert(key, (uuid.clone(), comp.clone()));
        }
    }

    let cleaned: Vec<Value> = seen.into_values().map(|(_, val)| val).collect();
    (cleaned, remap, count)
}

fn remap_component_uuids(root: &mut Map<String, Value>, comp_remap: &HashMap<String, String>) {
    if let Some(ctrl_imp) = root
        .get_mut("control-implementation")
        .and_then(Value::as_object_mut)
    {
        if let Some(reqs) = ctrl_imp
            .get_mut("implemented-requirements")
            .and_then(Value::as_array_mut)
        {
            for req in reqs {
                if let Some(by_comps) = req.get_mut("by-components").and_then(Value::as_array_mut) {
                    for bc in by_comps {
                        if let Some(cuuid) = bc.get_mut("component-uuid") {
                            if let Some(u_str) = cuuid.as_str() {
                                if let Some(target) = comp_remap.get(u_str) {
                                    *cuuid = json!(target);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn dedup_control_array(controls: &[Value], modifications: &mut Vec<String>) -> (Vec<Value>, usize) {
    let mut map: BTreeMap<String, Value> = BTreeMap::new();
    let mut count = 0;

    for ctrl in controls {
        let cid = ctrl
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(existing) = map.get_mut(&cid) {
            // Merge parameters
            if let Some(new_params) = ctrl.get("params").and_then(Value::as_array) {
                let existing_params = existing
                    .as_object_mut()
                    .unwrap()
                    .entry("params".to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Some(arr) = existing_params.as_array_mut() {
                    for np in new_params {
                        if !arr.contains(np) {
                            arr.push(np.clone());
                        }
                    }
                }
            }
            modifications.push(format!("Merged duplicate control '{cid}'"));
            count += 1;
        } else {
            map.insert(cid, ctrl.clone());
        }
    }

    (map.into_values().collect(), count)
}

fn dedup_group_controls(grp: &mut Value, count: &mut usize, modifications: &mut Vec<String>) {
    if let Some(ctrls) = grp.get_mut("controls").and_then(Value::as_array_mut) {
        let (cleaned, c) = dedup_control_array(ctrls, modifications);
        *ctrls = cleaned;
        *count += c;
    }
    if let Some(sub_grps) = grp.get_mut("groups").and_then(Value::as_array_mut) {
        for sub in sub_grps {
            dedup_group_controls(sub, count, modifications);
        }
    }
}
