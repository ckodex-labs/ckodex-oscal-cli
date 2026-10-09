use chrono::Utc;
use serde_json::{Map, Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    document::{parser::OscalDocument, schema::DocumentKind},
    error::{AppError, Result, io_error},
};

pub fn resolve_profile(
    profile_doc: &OscalDocument,
    output_path: Option<&Path>,
) -> Result<OscalDocument> {
    if profile_doc.kind != DocumentKind::Profile {
        return Err(AppError::Configuration(format!(
            "Expected profile document for resolution, got {}",
            profile_doc.kind.name()
        )));
    }

    let profile_obj = profile_doc
        .root_object()
        .ok_or_else(|| AppError::Configuration("Malformed profile document".to_owned()))?;

    let imports = profile_obj
        .get("imports")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Configuration("Profile missing 'imports' array".to_owned()))?;

    let base_dir = profile_doc
        .path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."));

    // Collect parameter overrides from modify.set-parameters
    let mut param_overrides: HashMap<String, Value> = HashMap::new();
    if let Some(modify) = profile_obj.get("modify").and_then(Value::as_object)
        && let Some(set_params) = modify.get("set-parameters").and_then(Value::as_array)
    {
        for sp in set_params {
            if let Some(pid) = sp.get("param-id").and_then(Value::as_str) {
                param_overrides.insert(pid.to_string(), sp.clone());
            }
        }
    }

    // Collect alters from modify.alters
    let mut alters_map: HashMap<String, Vec<Value>> = HashMap::new();
    if let Some(modify) = profile_obj.get("modify").and_then(Value::as_object)
        && let Some(alters) = modify.get("alters").and_then(Value::as_array)
    {
        for alter in alters {
            if let Some(cid) = alter.get("control-id").and_then(Value::as_str) {
                alters_map
                    .entry(cid.to_string())
                    .or_default()
                    .push(alter.clone());
            }
        }
    }

    let mut resolved_controls: Vec<Value> = Vec::new();
    let mut resolved_groups: Vec<Value> = Vec::new();

    for import in imports {
        let href = import
            .get("href")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Configuration("Import missing 'href'".to_owned()))?;

        let import_path = resolve_import_href(profile_obj, base_dir, href)?;
        let imported_doc = OscalDocument::from_file(&import_path)?;

        if imported_doc.kind != DocumentKind::Catalog {
            return Err(AppError::Configuration(format!(
                "Imported document '{}' is a {}, only catalog imports are currently supported for resolution",
                href,
                imported_doc.kind.name()
            )));
        }

        let imported_cat = imported_doc.root_object().ok_or_else(|| {
            AppError::Configuration(format!("Imported catalog '{}' has no root object", href))
        })?;

        // Extract selection rules
        let selection = Selection::from_import(import);

        // Process top-level controls
        if let Some(controls) = imported_cat.get("controls").and_then(Value::as_array) {
            resolved_controls.extend(select_controls(
                controls,
                &selection,
                Inherit::default(),
                &param_overrides,
                &alters_map,
            )?);
        }

        // Process groups
        if let Some(groups) = imported_cat.get("groups").and_then(Value::as_array) {
            for grp in groups {
                if let Some(resolved_grp) =
                    select_group(grp, &selection, &param_overrides, &alters_map)?
                {
                    resolved_groups.push(resolved_grp);
                }
            }
        }
    }

    let profile_meta = profile_doc.metadata().cloned().unwrap_or_default();
    let profile_title = profile_meta
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Configuration("Profile metadata missing 'title'".to_owned()))?;

    let now_str = Utc::now().to_rfc3339();
    let new_uuid = Uuid::new_v4().to_string();

    let mut cat_meta = Map::new();
    cat_meta.insert(
        "title".to_owned(),
        json!(format!("[Resolved] {profile_title}")),
    );
    cat_meta.insert("published".to_owned(), json!(now_str));
    cat_meta.insert("last-modified".to_owned(), json!(now_str));
    cat_meta.insert("version".to_owned(), json!("1.0.0-resolved"));
    cat_meta.insert("oscal-version".to_owned(), json!("1.2.3"));

    let mut resolved_catalog = Map::new();
    resolved_catalog.insert("uuid".to_owned(), json!(new_uuid));
    resolved_catalog.insert("metadata".to_owned(), Value::Object(cat_meta));

    if !resolved_groups.is_empty() {
        resolved_catalog.insert("groups".to_owned(), Value::Array(resolved_groups));
    }
    if !resolved_controls.is_empty() {
        resolved_catalog.insert("controls".to_owned(), Value::Array(resolved_controls));
    }

    let mut root_val = Map::new();
    root_val.insert("catalog".to_owned(), Value::Object(resolved_catalog));
    let resolved_json = Value::Object(root_val);

    if let Some(out_p) = output_path {
        let serialized = serde_json::to_string_pretty(&resolved_json).map_err(|e| {
            AppError::Configuration(format!("Failed to serialize resolved catalog: {e}"))
        })?;
        fs::write(out_p, serialized).map_err(|e| io_error(out_p, e))?;
    }

    let resolved_str = serde_json::to_string(&resolved_json).map_err(AppError::Serialization)?;
    OscalDocument::from_str(&resolved_str, output_path.map(Path::to_path_buf))
}

/// Resolve a profile import href to a filesystem path.
///
/// OSCAL permits two forms:
/// - a direct URI reference (relative path), resolved against the profile's directory;
/// - a fragment `#<uuid>` that points at a back-matter resource, whose `rlinks`
///   carry the actual location. A JSON rlink is preferred; otherwise the first
///   rlink is used.
///
/// Remote (http/https) locations are rejected explicitly: the resolver is offline
/// by design and does not fetch.
fn resolve_import_href(
    profile_obj: &Map<String, Value>,
    base: &Path,
    href: &str,
) -> Result<PathBuf> {
    let location = if let Some(uuid) = href.strip_prefix('#') {
        let resource = profile_obj
            .get("back-matter")
            .and_then(|bm| bm.get("resources"))
            .and_then(Value::as_array)
            .and_then(|rs| {
                rs.iter()
                    .find(|r| r.get("uuid").and_then(Value::as_str) == Some(uuid))
            })
            .ok_or_else(|| {
                AppError::Configuration(format!(
                    "Import href '{href}' does not match any back-matter resource in the profile"
                ))
            })?;
        let rlinks = resource
            .get("rlinks")
            .and_then(Value::as_array)
            .filter(|r| !r.is_empty())
            .ok_or_else(|| {
                AppError::Configuration(format!(
                    "Back-matter resource '{uuid}' referenced by import has no rlinks"
                ))
            })?;
        let is_json = |r: &&Value| {
            r.get("media-type")
                .and_then(Value::as_str)
                .is_some_and(|m| m.ends_with("json"))
                || r.get("href")
                    .and_then(Value::as_str)
                    .is_some_and(|h| h.ends_with(".json"))
        };
        rlinks
            .iter()
            .find(is_json)
            .or_else(|| rlinks.first())
            .and_then(|r| r.get("href"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::Configuration(format!(
                    "Back-matter resource '{uuid}' has no usable rlink href"
                ))
            })?
            .to_string()
    } else {
        href.to_string()
    };

    if location.starts_with("http://") || location.starts_with("https://") {
        return Err(AppError::Configuration(format!(
            "Import location '{location}' is remote; the resolver is offline and does not fetch. Download the catalog and reference it by relative path"
        )));
    }

    let direct = base.join(&location);
    if direct.exists() {
        return Ok(direct);
    }
    // Official NIST profiles reference the catalog by its repository-relative path
    // (e.g. ../../../../nist.gov/.../catalog.json). When that tree is not present,
    // fall back to a sibling file with the same name in the profile's directory.
    if let Some(name) = Path::new(&location).file_name() {
        let sibling = base.join(name);
        if sibling.exists() {
            return Ok(sibling);
        }
    }
    Ok(direct)
}

/// Import selection per the OSCAL profile model.
struct Selection {
    include_all: bool,
    ids: HashSet<String>,
    ids_with_children: HashSet<String>,
    patterns: Vec<(String, bool)>,
    excluded: HashSet<String>,
    excluded_with_children: HashSet<String>,
    excluded_patterns: Vec<(String, bool)>,
}

#[derive(Clone, Copy, Default)]
struct Inherit {
    included: bool,
    excluded: bool,
}

impl Selection {
    fn from_import(import: &Value) -> Self {
        let mut s = Selection {
            include_all: import.get("include-all").is_some(),
            ids: HashSet::new(),
            ids_with_children: HashSet::new(),
            patterns: Vec::new(),
            excluded: HashSet::new(),
            excluded_with_children: HashSet::new(),
            excluded_patterns: Vec::new(),
        };

        let inc = import.get("include-controls").and_then(Value::as_array);
        if let Some(items) = inc {
            for item in items {
                Self::read_call(item, &mut s.ids, &mut s.ids_with_children, &mut s.patterns);
            }
        } else if !s.include_all {
            // Neither include-all nor include-controls: treat as include-all,
            // matching the previous behaviour of this resolver.
            s.include_all = true;
        }

        if let Some(items) = import.get("exclude-controls").and_then(Value::as_array) {
            for item in items {
                Self::read_call(
                    item,
                    &mut s.excluded,
                    &mut s.excluded_with_children,
                    &mut s.excluded_patterns,
                );
            }
        }
        s
    }

    fn read_call(
        item: &Value,
        ids: &mut HashSet<String>,
        with_children: &mut HashSet<String>,
        patterns: &mut Vec<(String, bool)>,
    ) {
        let children = item
            .get("with-child-controls")
            .and_then(Value::as_str)
            .is_some_and(|v| v == "yes");
        if let Some(list) = item.get("with-ids").and_then(Value::as_array) {
            for id in list.iter().filter_map(Value::as_str) {
                ids.insert(id.to_string());
                if children {
                    with_children.insert(id.to_string());
                }
            }
        }
        if let Some(list) = item.get("matching").and_then(Value::as_array) {
            for pat in list
                .iter()
                .filter_map(|m| m.get("pattern").and_then(Value::as_str))
            {
                patterns.push((pat.to_string(), children));
            }
        }
    }

    /// Returns (selected, inheritance passed to children).
    fn evaluate(&self, id: &str, parent: Inherit) -> (bool, Inherit) {
        let pat_hit = |pats: &[(String, bool)]| {
            pats.iter()
                .filter(|(p, _)| glob_match(p, id))
                .fold((false, false), |(_, ch), (_, c)| (true, ch || *c))
        };

        let (inc_pat, inc_pat_children) = pat_hit(&self.patterns);
        let (exc_pat, exc_pat_children) = pat_hit(&self.excluded_patterns);

        let excluded = parent.excluded || self.excluded.contains(id) || exc_pat;
        let included = self.include_all || parent.included || self.ids.contains(id) || inc_pat;

        let child = Inherit {
            included: self.include_all
                || parent.included
                || self.ids_with_children.contains(id)
                || inc_pat_children,
            excluded: parent.excluded
                || self.excluded_with_children.contains(id)
                || exc_pat_children,
        };
        (included && !excluded, child)
    }
}

/// Minimal glob matcher supporting `*` and `?`, as used by OSCAL `matching/@pattern`.
fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (None::<usize>, 0usize);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Select controls (recursively) from a list. A selected control keeps only its
/// selected descendants. Selected descendants of an unselected control are
/// promoted to the current level so they are not silently lost.
fn select_controls(
    controls: &[Value],
    sel: &Selection,
    parent: Inherit,
    param_overrides: &HashMap<String, Value>,
    alters: &HashMap<String, Vec<Value>>,
) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    for ctrl in controls {
        let id = ctrl.get("id").and_then(Value::as_str).unwrap_or("");
        let (selected, child_inherit) = sel.evaluate(id, parent);

        let children = match ctrl.get("controls").and_then(Value::as_array) {
            Some(sub) => select_controls(sub, sel, child_inherit, param_overrides, alters)?,
            None => Vec::new(),
        };

        if selected {
            let mut resolved = ctrl.clone();
            if let Some(obj) = resolved.as_object_mut() {
                if children.is_empty() {
                    obj.remove("controls");
                } else {
                    obj.insert("controls".to_owned(), Value::Array(children));
                }
            }
            apply_control_modifications(&mut resolved, param_overrides, alters)?;
            out.push(resolved);
        } else {
            out.extend(children);
        }
    }
    Ok(out)
}

fn select_group(
    grp: &Value,
    sel: &Selection,
    param_overrides: &HashMap<String, Value>,
    alters: &HashMap<String, Vec<Value>>,
) -> Result<Option<Value>> {
    let mut resolved = grp.clone();
    let Some(obj) = resolved.as_object_mut() else {
        return Ok(None);
    };

    let controls = match grp.get("controls").and_then(Value::as_array) {
        Some(c) => select_controls(c, sel, Inherit::default(), param_overrides, alters)?,
        None => Vec::new(),
    };
    let mut sub_groups = Vec::new();
    if let Some(gs) = grp.get("groups").and_then(Value::as_array) {
        for g in gs {
            if let Some(r) = select_group(g, sel, param_overrides, alters)? {
                sub_groups.push(r);
            }
        }
    }

    if controls.is_empty() && sub_groups.is_empty() {
        return Ok(None);
    }
    if controls.is_empty() {
        obj.remove("controls");
    } else {
        obj.insert("controls".to_owned(), Value::Array(controls));
    }
    if sub_groups.is_empty() {
        obj.remove("groups");
    } else {
        obj.insert("groups".to_owned(), Value::Array(sub_groups));
    }
    Ok(Some(resolved))
}

/// Apply parameter overrides and alterations to a single control.
/// Children are handled by `select_controls`, so this does not recurse
/// (recursing here applied alters twice to nested controls).
fn apply_control_modifications(
    ctrl: &mut Value,
    param_overrides: &HashMap<String, Value>,
    alters: &HashMap<String, Vec<Value>>,
) -> Result<()> {
    let cid = ctrl
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if let Some(params) = ctrl.get_mut("params").and_then(Value::as_array_mut) {
        for param in params {
            if let Some(pid) = param.get("id").and_then(Value::as_str)
                && let Some(override_val) = param_overrides.get(pid)
            {
                let param_obj = param.as_object_mut().ok_or_else(|| {
                    AppError::Configuration("Control param is not an object".to_owned())
                })?;
                if let Some(vals) = override_val.get("values") {
                    param_obj.insert("values".to_owned(), vals.clone());
                }
                if let Some(label) = override_val.get("label") {
                    param_obj.insert("label".to_owned(), label.clone());
                }
            }
        }
    }

    if let Some(alter_list) = alters.get(&cid) {
        for alter in alter_list {
            if let Some(adds) = alter.get("adds").and_then(Value::as_array) {
                for add in adds {
                    if let Some(props) = add.get("props").and_then(Value::as_array) {
                        let ctrl_obj = ctrl.as_object_mut().ok_or_else(|| {
                            AppError::Configuration("Control is not an object".to_owned())
                        })?;
                        let ctrl_props = ctrl_obj
                            .entry("props".to_owned())
                            .or_insert_with(|| Value::Array(Vec::new()));
                        if let Some(arr) = ctrl_props.as_array_mut() {
                            arr.extend(props.iter().cloned());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl(id: &str, children: Vec<Value>) -> Value {
        if children.is_empty() {
            json!({"id": id, "title": id})
        } else {
            json!({"id": id, "title": id, "controls": children})
        }
    }

    fn ids(controls: &[Value]) -> Vec<String> {
        let mut out = Vec::new();
        for c in controls {
            out.push(c["id"].as_str().unwrap_or_default().to_string());
            if let Some(sub) = c.get("controls").and_then(Value::as_array) {
                out.extend(ids(sub));
            }
        }
        out
    }

    fn run(import: Value, controls: Vec<Value>) -> Vec<String> {
        let sel = Selection::from_import(&import);
        let r = select_controls(
            &controls,
            &sel,
            Inherit::default(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .expect("selection");
        ids(&r)
    }

    fn tree() -> Vec<Value> {
        vec![
            ctrl("ac-1", vec![]),
            ctrl("ac-2", vec![ctrl("ac-2.1", vec![]), ctrl("ac-2.2", vec![])]),
        ]
    }

    #[test]
    fn with_ids_does_not_pull_unlisted_enhancements() {
        let got = run(
            json!({"include-controls": [{"with-ids": ["ac-2", "ac-2.1"]}]}),
            tree(),
        );
        assert_eq!(got, vec!["ac-2", "ac-2.1"]);
    }

    #[test]
    fn with_child_controls_includes_descendants_only_of_listed() {
        let got = run(
            json!({"include-controls": [{"with-ids": ["ac-2"], "with-child-controls": "yes"}]}),
            tree(),
        );
        assert_eq!(got, vec!["ac-2", "ac-2.1", "ac-2.2"]);
    }

    #[test]
    fn exclude_overrides_include_all() {
        let got = run(
            json!({"include-all": {}, "exclude-controls": [{"with-ids": ["ac-2.2"]}]}),
            tree(),
        );
        assert_eq!(got, vec!["ac-1", "ac-2", "ac-2.1"]);
    }

    #[test]
    fn matching_pattern_is_a_glob() {
        let got = run(
            json!({"include-controls": [{"matching": [{"pattern": "ac-2*"}]}]}),
            tree(),
        );
        assert_eq!(got, vec!["ac-2", "ac-2.1", "ac-2.2"]);
        assert!(glob_match("ac-?", "ac-1"));
        assert!(!glob_match("ac-?", "ac-10"));
    }

    #[test]
    fn selected_child_of_unselected_parent_is_promoted() {
        let got = run(
            json!({"include-controls": [{"with-ids": ["ac-2.2"]}]}),
            tree(),
        );
        assert_eq!(got, vec!["ac-2.2"]);
    }

    #[test]
    fn back_matter_fragment_import_resolves_json_rlink() {
        let dir = std::env::temp_dir().join(format!("mizan-resolver-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("tmp dir");
        fs::write(dir.join("cat.json"), "{}").expect("write");
        let profile = json!({
            "back-matter": {"resources": [{
                "uuid": "r1",
                "rlinks": [
                    {"href": "../../x/cat.xml", "media-type": "application/oscal.catalog+xml"},
                    {"href": "../../x/cat.json", "media-type": "application/oscal.catalog+json"}
                ]
            }]}
        });
        let obj = profile.as_object().expect("object");
        let p = resolve_import_href(obj, &dir, "#r1").expect("resolve");
        assert_eq!(p, dir.join("cat.json"));
        assert!(resolve_import_href(obj, &dir, "#missing").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn remote_import_is_rejected_offline() {
        let obj = Map::new();
        let err = resolve_import_href(&obj, Path::new("."), "https://example.org/cat.json");
        assert!(err.is_err());
    }
}
