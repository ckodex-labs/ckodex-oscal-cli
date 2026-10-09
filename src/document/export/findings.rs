//! Extraction of assessment findings shared by the CI/CD exporters.
//!
//! SARIF and GitLab reports describe *findings*. Catalog controls are rule
//! definitions, not findings, so they must never be reported as results or
//! vulnerabilities. Findings come from `assessment-results.results[].findings[]`
//! in one of two shapes:
//!
//! * OSCAL 1.x findings: `target.target-id` and `target.status.state`
//!   (`satisfied` / `not-satisfied`), with `title` and `description`.
//! * Findings written by `mizan pipeline run`: `rule_id`, `status`
//!   (`FAILED` / `WAIVED`), `violation`, and for waivers `reason`.

use serde_json::Value;

use crate::document::parser::OscalDocument;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FindingDisposition {
    /// The requirement is not satisfied.
    Failed,
    /// Not satisfied, but covered by an accepted derogation. The failure is
    /// preserved; the waiver is attached as justification.
    Waived { justification: Option<String> },
    /// The requirement is satisfied.
    Satisfied,
    /// The status is absent or not recognised. Never treated as a pass or a failure.
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ExtractedFinding {
    pub rule_id: String,
    pub title: Option<String>,
    pub message: String,
    pub disposition: FindingDisposition,
}

fn str_at<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

fn extract_one(f: &Value) -> Option<ExtractedFinding> {
    // Pipeline shape.
    if let Some(rule_id) = str_at(f, "rule_id") {
        let disposition = match str_at(f, "status").map(str::to_ascii_uppercase).as_deref() {
            Some("FAILED" | "ERROR") => FindingDisposition::Failed,
            Some("WAIVED") => FindingDisposition::Waived {
                justification: str_at(f, "reason").map(String::from),
            },
            Some("PASSED") => FindingDisposition::Satisfied,
            _ => FindingDisposition::Unknown,
        };
        let message = str_at(f, "violation")
            .map_or_else(|| format!("Rule {rule_id}"), String::from);
        return Some(ExtractedFinding {
            rule_id: rule_id.to_string(),
            title: None,
            message,
            disposition,
        });
    }

    // OSCAL shape.
    let target = f.get("target")?;
    let rule_id = str_at(target, "target-id")?;
    let state = target
        .get("status")
        .and_then(|s| str_at(s, "state"))
        .map(str::to_ascii_lowercase);
    let disposition = match state.as_deref() {
        Some("not-satisfied") => FindingDisposition::Failed,
        Some("satisfied") => FindingDisposition::Satisfied,
        _ => FindingDisposition::Unknown,
    };
    let title = str_at(f, "title").map(String::from);
    let message = str_at(f, "description")
        .map(String::from)
        .or_else(|| title.clone())
        .unwrap_or_else(|| format!("Finding for {rule_id}"));
    Some(ExtractedFinding {
        rule_id: rule_id.to_string(),
        title,
        message,
        disposition,
    })
}

/// Returns every finding in an assessment-results document. Other document
/// kinds have no findings and yield an empty list.
#[must_use]
pub fn extract_findings(doc: &OscalDocument) -> Vec<ExtractedFinding> {
    let Some(root) = doc.root_object() else {
        return Vec::new();
    };
    root.get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("findings").and_then(Value::as_array))
        .flatten()
        .filter_map(extract_one)
        .collect()
}

/// Collects `(id, title)` for every control in a catalog, including controls
/// nested in groups and control enhancements.
#[must_use]
pub fn collect_catalog_controls(doc: &OscalDocument) -> Vec<(String, Option<String>)> {
    fn walk(
        controls: Option<&Value>,
        groups: Option<&Value>,
        out: &mut Vec<(String, Option<String>)>,
    ) {
        for c in controls.and_then(Value::as_array).into_iter().flatten() {
            if let Some(id) = str_at(c, "id") {
                out.push((id.to_string(), str_at(c, "title").map(String::from)));
            }
            walk(c.get("controls"), None, out);
        }
        for g in groups.and_then(Value::as_array).into_iter().flatten() {
            walk(g.get("controls"), g.get("groups"), out);
        }
    }
    let mut out = Vec::new();
    if let Some(root) = doc.root_object() {
        walk(root.get("controls"), root.get("groups"), &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_pipeline_and_oscal_shapes() {
        let doc = OscalDocument::from_value(
            serde_json::json!({
                "assessment-results": {
                    "uuid": "u",
                    "metadata": { "title": "t" },
                    "results": [{
                        "findings": [
                            { "rule_id": "r1", "status": "FAILED", "violation": "bad" },
                            { "rule_id": "r2", "status": "WAIVED", "violation": "meh", "reason": "accepted" },
                            { "uuid": "f", "title": "AC-2", "description": "missing",
                              "target": { "type": "objective-id", "target-id": "ac-2_obj",
                                          "status": { "state": "not-satisfied" } } },
                            { "target": { "target-id": "ac-3", "status": { "state": "satisfied" } } },
                            { "target": { "target-id": "ac-4" } }
                        ]
                    }]
                }
            }),
            None,
        )
        .unwrap();
        let f = extract_findings(&doc);
        assert_eq!(f.len(), 5);
        assert_eq!(f[0].disposition, FindingDisposition::Failed);
        assert_eq!(
            f[1].disposition,
            FindingDisposition::Waived {
                justification: Some("accepted".into())
            }
        );
        assert_eq!(f[2].disposition, FindingDisposition::Failed);
        assert_eq!(f[2].message, "missing");
        assert_eq!(f[3].disposition, FindingDisposition::Satisfied);
        assert_eq!(f[4].disposition, FindingDisposition::Unknown);
    }

    #[test]
    fn catalog_controls_are_collected_through_groups() {
        let doc = OscalDocument::from_value(
            serde_json::json!({
                "catalog": {
                    "uuid": "c",
                    "metadata": { "title": "t" },
                    "groups": [{ "id": "ac", "controls": [
                        { "id": "ac-1", "title": "P" },
                        { "id": "ac-2", "title": "A", "controls": [{ "id": "ac-2.1", "title": "E" }] }
                    ]}]
                }
            }),
            None,
        )
        .unwrap();
        let ids: Vec<_> = collect_catalog_controls(&doc).into_iter().map(|c| c.0).collect();
        assert_eq!(ids, vec!["ac-1", "ac-2", "ac-2.1"]);
        assert!(extract_findings(&doc).is_empty());
    }
}
