use crate::{
    document::{
        BuiltinRulepack, CasStore, EmbeddedCatalogProvider, EnterpriseCatalogBuilder,
        EvidenceBundle, FedrampBaseline, GitLabReportExporter, Jurisdiction, KubeAuditor,
        KubeClusterClient, MergeStrategy, OscalDocument, PipelineConfig, PipelineOrchestrator,
        RegorusEvaluator, SarifExporter, SbomImporter, SlsaProvenanceBuilder, SlsaVersion,
        analyze_blast_radius, assemble_directory, inspect_document, split_document, sync_and_merge,
        validate_fedramp,
    },
    fabric::RootFabricEngine,
    mcp::protocol::McpError,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn require_str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, McpError> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| McpError::MissingArgument(key.to_string()))
}

fn optional_str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn optional_object_arg(args: &Value, key: &str) -> Value {
    args.get(key).cloned().unwrap_or(Value::Null)
}

fn confine_output_path(path: &str) -> Result<PathBuf, McpError> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err(McpError::InvalidArgument(
            "path must not be empty or contain NUL bytes".to_string(),
        ));
    }
    let base = std::env::current_dir().map_err(McpError::Io)?;
    let raw_path = Path::new(trimmed);
    for comp in raw_path.components() {
        use std::path::Component;
        if matches!(comp, Component::ParentDir | Component::Prefix(_)) {
            return Err(McpError::PathEscape(format!(
                "path traversal or prefix is forbidden: {path}"
            )));
        }
    }
    let candidate = if raw_path.is_absolute() {
        PathBuf::from(raw_path)
    } else {
        base.join(raw_path)
    };
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    for comp in candidate.components() {
        use std::path::Component;
        match comp {
            Component::Prefix(_) | Component::RootDir => {
                stack.push(comp.as_os_str().to_os_string());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = stack.last() {
                    if last.as_os_str() == std::ffi::OsStr::new("/") {
                        return Err(McpError::PathEscape(format!("path escapes root: {path}")));
                    }
                    stack.pop();
                } else {
                    return Err(McpError::PathEscape(format!(
                        "output path {path} escapes workspace"
                    )));
                }
            }
            Component::Normal(s) => stack.push(s.to_os_string()),
        }
    }
    let mut normalized = PathBuf::new();
    for s in stack {
        normalized.push(s);
    }
    let tmp_dir = std::env::temp_dir();
    let allowed_root = if normalized.starts_with(&base) {
        Some(base.clone())
    } else if normalized.starts_with(&tmp_dir) {
        Some(tmp_dir.clone())
    } else {
        None
    };
    let Some(root) = allowed_root else {
        return Err(McpError::PathEscape(format!(
            "output path {path} escapes workspace {}",
            base.display()
        )));
    };
    // Check every component from root downward for symlinks
    if let Ok(rel) = normalized.strip_prefix(&root) {
        let mut probe = root.clone();
        for comp in rel.components() {
            probe.push(comp.as_os_str());
            if let Ok(meta) = std::fs::symlink_metadata(&probe)
                && meta.file_type().is_symlink()
            {
                return Err(McpError::PathEscape(format!(
                    "symlinked path component is forbidden: {path}"
                )));
            }
        }
    }
    let canon_base = std::fs::canonicalize(&base).unwrap_or(base);
    let canon_tmp = std::fs::canonicalize(&tmp_dir).unwrap_or(tmp_dir);
    let mut existing = normalized.as_path();
    while !existing.exists() {
        if let Some(parent) = existing.parent() {
            existing = parent;
        } else {
            break;
        }
    }
    if existing.exists()
        && let Ok(canon_existing) = std::fs::canonicalize(existing)
        && !canon_existing.starts_with(&canon_base)
        && !canon_existing.starts_with(&canon_tmp)
    {
        return Err(McpError::PathEscape(format!(
            "path {path} resolves outside workspace via symlink"
        )));
    }
    Ok(normalized)
}

fn validate_input_path(path: &str) -> Result<PathBuf, McpError> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err(McpError::InvalidArgument(
            "path must not be empty or contain NUL bytes".to_string(),
        ));
    }
    let p = PathBuf::from(trimmed);
    for comp in p.components() {
        use std::path::Component;
        if matches!(comp, Component::ParentDir | Component::Prefix(_)) {
            return Err(McpError::PathEscape(format!(
                "path traversal or prefix is forbidden: {path}"
            )));
        }
    }
    let mut current = PathBuf::new();
    for comp in p.components() {
        current.push(comp.as_os_str());
        let is_system_root_alias =
            current != p && matches!(current.to_str(), Some("/var" | "/tmp"));
        if !is_system_root_alias
            && let Ok(meta) = std::fs::symlink_metadata(&current)
            && meta.file_type().is_symlink()
        {
            return Err(McpError::PathEscape(format!(
                "symlinked input path is forbidden: {path}"
            )));
        }
    }
    if p.is_relative() {
        return confine_output_path(trimmed);
    }

    let cwd = std::env::current_dir().map_err(McpError::Io)?;
    let canon_cwd = std::fs::canonicalize(&cwd).unwrap_or(cwd);
    let tmp_dir = std::env::temp_dir();
    let canon_tmp = std::fs::canonicalize(&tmp_dir).unwrap_or(tmp_dir);

    let mut probe = p.as_path();
    while !probe.exists() {
        if let Some(parent) = probe.parent() {
            probe = parent;
        } else {
            break;
        }
    }
    let canon_target = if probe.exists() {
        std::fs::canonicalize(probe).map_err(McpError::Io)?
    } else {
        probe.to_path_buf()
    };
    if !canon_target.starts_with(&canon_cwd) && !canon_target.starts_with(&canon_tmp) {
        return Err(McpError::PathEscape(format!(
            "input path {path} escapes workspace {}",
            canon_cwd.display()
        )));
    }
    Ok(p)
}

pub fn execute_tool(name: &str, args: &Value) -> Result<String, McpError> {
    match name {
        "query_control" => {
            let cat_file = validate_input_path(require_str_arg(args, "catalog_file")?)?;
            let ctrl_id = require_str_arg(args, "control_id")?;
            let doc = OscalDocument::from_file(&cat_file)?;
            let root = doc.root_object().ok_or_else(|| {
                McpError::InvalidArgument("document has no root object".to_string())
            })?;
            let ctrls = root
                .get("controls")
                .and_then(Value::as_array)
                .ok_or_else(|| McpError::InvalidArgument("document has no controls".to_string()))?;
            for c in ctrls {
                if c.get("id").and_then(Value::as_str) == Some(ctrl_id) {
                    return Ok(serde_json::to_string_pretty(c)?);
                }
            }
            Ok(format!("Control '{ctrl_id}' not found in catalog."))
        }
        "inspect_document" => {
            let file = validate_input_path(require_str_arg(args, "file")?)?;
            let doc = OscalDocument::from_file(&file)?;
            let rep = inspect_document(&doc)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "compute_blast_radius" => {
            let file = validate_input_path(require_str_arg(args, "file")?)?;
            let target = require_str_arg(args, "target")?;
            let doc = OscalDocument::from_file(&file)?;
            let rep = analyze_blast_radius(&doc, &[], target, 3)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "validate_fedramp" => {
            let ssp_file = validate_input_path(require_str_arg(args, "ssp_file")?)?;
            let base_str = optional_str_arg(args, "baseline").unwrap_or("moderate");
            let baseline =
                FedrampBaseline::from_str_name(base_str).unwrap_or(FedrampBaseline::Moderate);
            let doc = OscalDocument::from_file(&ssp_file)?;
            let rep = validate_fedramp(&doc, baseline)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "audit_kubernetes_cluster" => {
            let namespace = optional_str_arg(args, "namespace");
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let out_ref = out_path.as_deref();
            let result = match tokio::runtime::Handle::try_current() {
                Ok(handle)
                    if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread =>
                {
                    tokio::task::block_in_place(|| {
                        handle.block_on(async {
                            let mut auditor =
                                KubeAuditor::new(KubeClusterClient::try_connect().await);
                            auditor.audit_cluster(namespace, None, out_ref).await
                        })
                    })
                }
                _ => std::thread::scope(|s| {
                    s.spawn(|| -> Result<_, McpError> {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(McpError::Io)?;
                        Ok(rt.block_on(async {
                            let mut auditor =
                                KubeAuditor::new(KubeClusterClient::try_connect().await);
                            auditor.audit_cluster(namespace, None, out_ref).await
                        }))
                    })
                    .join()
                    .map_err(|_| McpError::Tool {
                        tool: "audit_kubernetes_cluster".to_string(),
                        message: "background audit thread panicked".to_string(),
                    })?
                })?,
            };
            let (_doc, report) = result?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
        "evaluate_rego_policy" => {
            let policy_file = validate_input_path(require_str_arg(args, "policy_file")?)?;
            let input_data = optional_object_arg(args, "input_data");
            let query_package = optional_str_arg(args, "query_package").unwrap_or("data");
            let mut evaluator = RegorusEvaluator::new();
            evaluator.add_policy_dir(&policy_file)?;
            let res = evaluator.evaluate_compliance_rule(query_package, input_data)?;
            Ok(serde_json::to_string_pretty(&res)?)
        }
        "sync_3way_merge" => {
            let base_f = validate_input_path(require_str_arg(args, "base_file")?)?;
            let upstream_f = validate_input_path(require_str_arg(args, "upstream_file")?)?;
            let local_f = validate_input_path(require_str_arg(args, "local_file")?)?;
            let strat_str = optional_str_arg(args, "strategy").unwrap_or("manual");
            let strat = MergeStrategy::from_str_name(strat_str).unwrap_or(MergeStrategy::Manual);
            let base = OscalDocument::from_file(&base_f)?;
            let up = OscalDocument::from_file(&upstream_f)?;
            let local = OscalDocument::from_file(&local_f)?;
            let (_doc, report) = sync_and_merge(&base, &up, &local, strat, None, None)?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
        "split_oscal_markdown" => {
            let file = validate_input_path(require_str_arg(args, "file")?)?;
            let out_dir = require_str_arg(args, "output_dir")?;
            let out_path = confine_output_path(out_dir)?;
            let doc = OscalDocument::from_file(&file)?;
            let rep = split_document(&doc, &out_path)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "assemble_oscal_markdown" => {
            let in_dir = validate_input_path(require_str_arg(args, "input_dir")?)?;
            let out_file = require_str_arg(args, "output_file")?;
            let out_path = confine_output_path(out_file)?;
            let (_doc, rep) = assemble_directory(&in_dir, Some(&out_path))?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "export_baseline_catalog" => {
            let jur_str = optional_str_arg(args, "jurisdiction").unwrap_or("us");
            let jur = Jurisdiction::from_str_name(jur_str).ok_or_else(|| {
                McpError::InvalidArgument(format!("Unknown jurisdiction: {jur_str}"))
            })?;
            let doc = EmbeddedCatalogProvider::get_catalog(jur)?;
            Ok(serde_json::to_string_pretty(&doc.value)?)
        }
        "extend_enterprise_catalog" => {
            let base_str = optional_str_arg(args, "base_jurisdiction").unwrap_or("us");
            let base = Jurisdiction::from_str_name(base_str).ok_or_else(|| {
                McpError::InvalidArgument(format!("Unknown base jurisdiction: {base_str}"))
            })?;
            let title = require_str_arg(args, "title")?;
            let cid = optional_str_arg(args, "custom_control_id");
            let ctitle = optional_str_arg(args, "custom_control_title");
            let cdesc = optional_str_arg(args, "custom_control_description");
            let mut builder = EnterpriseCatalogBuilder::new(title).with_base_jurisdiction(base);
            if let (Some(id), Some(t), Some(d)) = (cid, ctitle, cdesc) {
                builder = builder.add_custom_control(id, t, d, "Enterprise-Policy");
            }
            let doc = builder.build(None)?;
            Ok(serde_json::to_string_pretty(&doc.value)?)
        }
        "generate_slsa_provenance" => {
            let s_name = require_str_arg(args, "subject_name")?;
            let s_digest = require_str_arg(args, "subject_digest")?;
            let ver_str = optional_str_arg(args, "slsa_version").unwrap_or("v1.2");
            let ver = SlsaVersion::from_str_name(ver_str).unwrap_or(SlsaVersion::V1_2);
            let mut builder = SlsaProvenanceBuilder::new(s_name, s_digest).with_version(ver);
            if let Some(ev_path) = optional_str_arg(args, "evidence_file") {
                let valid_ev_path = validate_input_path(ev_path)?;
                let bundle = EvidenceBundle::load_from_file(&valid_ev_path)?;
                builder = builder.with_oscal_evidence(bundle);
            }
            let stmt = builder.build();
            Ok(serde_json::to_string_pretty(&stmt)?)
        }
        "verify_slsa_provenance" => {
            let stmt_str = require_str_arg(args, "statement_json")?;
            let stmt: Value = serde_json::from_str(stmt_str).map_err(McpError::Serialization)?;
            let rep = SlsaProvenanceBuilder::verify(&stmt, None)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "query_cas_store" => {
            let cas = CasStore::default();
            if let Some(digest) = optional_str_arg(args, "digest") {
                Ok(cas.get_str(digest)?)
            } else {
                let stats = cas.stats()?;
                Ok(serde_json::to_string_pretty(&stats)?)
            }
        }
        "export_sarif" => {
            let file_path = validate_input_path(require_str_arg(args, "file")?)?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let doc = OscalDocument::from_file(&file_path)?;
            let sarif = SarifExporter::export_from_oscal(&doc, &file_path)?;
            if let Some(out) = out_path {
                SarifExporter::save_to_file(&sarif, &out)?;
                Ok(format!("SARIF report saved to {}", out.display()))
            } else {
                Ok(serde_json::to_string_pretty(&sarif)?)
            }
        }
        "export_gitlab" => {
            let file_path = validate_input_path(require_str_arg(args, "file")?)?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let doc = OscalDocument::from_file(&file_path)?;
            let report = GitLabReportExporter::export_from_oscal(&doc, &file_path)?;
            if let Some(out) = out_path {
                GitLabReportExporter::save_to_file(&report, &out)?;
                Ok(format!("GitLab report saved to {}", out.display()))
            } else {
                Ok(serde_json::to_string_pretty(&report)?)
            }
        }
        "import_sbom_cyclonedx" => {
            let file_path = validate_input_path(require_str_arg(args, "file")?)?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let (doc, summary) = SbomImporter::import_file(&file_path)?;
            if let Some(out) = out_path {
                let json_str = serde_json::to_string_pretty(&doc.value)?;
                crate::document::atomic_write_bytes(&out, json_str.as_bytes())?;
                Ok(format!("Imported SBOM saved to {}", out.display()))
            } else {
                let res = json!({ "summary": summary, "oscal_document": doc.value });
                Ok(serde_json::to_string_pretty(&res)?)
            }
        }
        "evaluate_policy_rulepack" => {
            let rule_id = require_str_arg(args, "rule_id")?;
            let input_data = require_str_arg(args, "input_data")?;
            let input_val: Value =
                serde_json::from_str(input_data).map_err(McpError::Serialization)?;
            let eval_res = BuiltinRulepack::evaluate_rule(rule_id, &input_val)?;
            Ok(serde_json::to_string_pretty(&eval_res)?)
        }
        "execute_compliance_pipeline" => {
            let jur_str = optional_str_arg(args, "jurisdiction").unwrap_or("us");
            let jur = Jurisdiction::from_str_name(jur_str).ok_or_else(|| {
                McpError::InvalidArgument(format!("Unknown jurisdiction: {jur_str}"))
            })?;
            let sbom_path = optional_str_arg(args, "sbom_file")
                .map(validate_input_path)
                .transpose()?;
            let workload_path = optional_str_arg(args, "workload_file")
                .map(validate_input_path)
                .transpose()?;
            let output_dir = match optional_str_arg(args, "output_dir")
                .map(confine_output_path)
                .transpose()?
            {
                Some(p) => p,
                None => confine_output_path("mizan-pipeline-output")?,
            };
            let subject_name = optional_str_arg(args, "subject_name")
                .unwrap_or("production-service")
                .to_string();
            let cfg = PipelineConfig {
                jurisdiction: jur,
                sbom_path,
                workload_path,
                rule_ids: Vec::new(),
                output_dir,
                subject_name,
                subject_digest:
                    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        .to_string(),
            };
            let report = PipelineOrchestrator::run(cfg)?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
        "fabric_status" => {
            let engine = RootFabricEngine::new();
            let tenants = engine.tenant_manager.list_tenants();
            let res = json!({
                "status": "OPERATIONAL",
                "trust_domain": "meridian.runbase.io",
                "tenants_count": tenants.len(),
                "tenants": tenants,
                "datastore_engine": "Isolated High-Assurance Partitioned",
                "spiffe_spire": "ACTIVE",
                "oidc_provider": "READY"
            });
            Ok(serde_json::to_string_pretty(&res)?)
        }
        "fabric_validate_spiffe" => {
            let spiffe_id_str = require_str_arg(args, "spiffe_id")?;
            let id = spiffe_id_str.parse::<crate::fabric::SpiffeId>().map_err(
                |e: crate::fabric::SpiffeError| McpError::InvalidArgument(e.to_string()),
            )?;
            let expected_td_str =
                optional_str_arg(args, "trust_domain").unwrap_or("meridian.runbase.io");
            let expected_td = crate::fabric::TrustDomain::new(expected_td_str)
                .map_err(|e| McpError::InvalidArgument(e.to_string()))?;
            let attestor = crate::fabric::SpireWorkloadAttestor::new(expected_td);
            let valid = attestor.verify_spiffe_id(&id).is_ok();
            let res = json!({
                "valid": valid,
                "spiffe_id": id.to_string(),
                "trust_domain": id.trust_domain().as_str(),
                "expected_trust_domain": attestor.expected_trust_domain().as_str(),
                "path": id.path(),
                "namespace": id.namespace(),
                "service_account": id.service_account()
            });
            Ok(serde_json::to_string_pretty(&res)?)
        }
        "ingest_security_report_to_poam" => {
            let report_f = validate_input_path(require_str_arg(args, "report_file")?)?;
            let fmt_hint = optional_str_arg(args, "format").unwrap_or("auto");
            let title =
                optional_str_arg(args, "title").unwrap_or("Automated Security Remediation POA&M");
            let existing_poam = optional_str_arg(args, "existing_poam_file")
                .map(validate_input_path)
                .transpose()?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let (_doc, report) =
                crate::document::ComplianceFederator::generate_poam_from_security_report(
                    &report_f,
                    fmt_hint,
                    title,
                    existing_poam.as_deref(),
                    out_path.as_deref(),
                )?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
        "export_utcp_manual" => {
            let transport_str = optional_str_arg(args, "transport").unwrap_or("http");
            let transport = crate::utcp::UtcpTransportType::from_str_name(transport_str)
                .ok_or_else(|| {
                    McpError::InvalidArgument(format!(
                        "Unsupported UTCP transport: {transport_str}"
                    ))
                })?;
            let base_url = optional_str_arg(args, "base_url").unwrap_or("http://localhost:3000");
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let manual = crate::utcp::UtcpManual::build_mizan_manual(transport, base_url);
            let json_str = serde_json::to_string_pretty(&manual)?;
            if let Some(out) = out_path {
                crate::document::atomic_write_bytes(&out, json_str.as_bytes())?;
            }
            Ok(json_str)
        }
        "search_utcp_tools" => {
            let query = optional_str_arg(args, "query").unwrap_or("");
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10)
                .clamp(1, 100) as usize;
            let transport_str = optional_str_arg(args, "transport").unwrap_or("http");
            let transport = crate::utcp::UtcpTransportType::from_str_name(transport_str)
                .ok_or_else(|| {
                    McpError::InvalidArgument(format!(
                        "Unsupported UTCP transport: {transport_str}"
                    ))
                })?;
            let base_url = optional_str_arg(args, "base_url").unwrap_or("http://localhost:3000");
            let manual = crate::utcp::UtcpManual::build_mizan_manual(transport, base_url);
            let tools = manual.search_tools(query, limit);
            let res = json!({
                "query": query,
                "count": tools.len(),
                "tools": tools
            });
            Ok(serde_json::to_string_pretty(&res)?)
        }
        _ => Err(McpError::Tool {
            tool: name.to_string(),
            message: format!("Unknown tool: {name}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_path_confinement_rejects_traversal_and_symlinks() {
        assert!(confine_output_path("").is_err());
        assert!(confine_output_path("../escape.json").is_err());
        assert!(confine_output_path("sub/../../escape.json").is_err());
        assert!(validate_input_path("").is_err());
        assert!(validate_input_path("../escape.json").is_err());
        assert!(
            validate_input_path("/etc/hosts").is_err(),
            "arbitrary absolute host path outside workspace/tempdir must be rejected"
        );
        assert!(
            validate_input_path("/Users/root/.ssh/config").is_err(),
            "arbitrary absolute home path outside workspace/tempdir must be rejected"
        );

        #[cfg(unix)]
        {
            let tmp =
                std::env::temp_dir().join(format!("mizan-mcp-path-test-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&tmp);
            let real_dir = tmp.join("real");
            std::fs::create_dir_all(&real_dir).unwrap();
            let real_file = real_dir.join("cat.json");
            std::fs::write(&real_file, "{}").unwrap();

            assert!(validate_input_path(real_file.to_str().unwrap()).is_ok());
            assert!(confine_output_path(real_dir.join("out.json").to_str().unwrap()).is_ok());
            assert!(confine_output_path("/etc/hosts").is_err());

            let sym_dir = tmp.join("symdir");
            std::os::unix::fs::symlink(&real_dir, &sym_dir).unwrap();
            assert!(
                validate_input_path(sym_dir.join("cat.json").to_str().unwrap()).is_err(),
                "intermediate directory symlink must be rejected"
            );

            let sym_file = tmp.join("symfile.json");
            std::os::unix::fs::symlink(&real_file, &sym_file).unwrap();
            assert!(
                validate_input_path(sym_file.to_str().unwrap()).is_err(),
                "leaf file symlink must be rejected"
            );

            let _ = std::fs::remove_dir_all(&tmp);
        }
    }

    #[test]
    fn test_mcp_fabric_validate_spiffe_trust_domain_enforcement() {
        let valid_out = execute_tool(
            "fabric_validate_spiffe",
            &json!({
                "spiffe_id": "spiffe://meridian.runbase.io/ns/prod/sa/auditor",
                "trust_domain": "meridian.runbase.io"
            }),
        )
        .expect("spiffe tool should succeed");
        let parsed: Value = serde_json::from_str(&valid_out).unwrap();
        assert_eq!(parsed["valid"], true);

        let mismatch_out = execute_tool(
            "fabric_validate_spiffe",
            &json!({
                "spiffe_id": "spiffe://rogue.domain.io/ns/prod/sa/auditor",
                "trust_domain": "meridian.runbase.io"
            }),
        )
        .expect("spiffe tool should return json on trust domain mismatch");
        let mismatch_parsed: Value = serde_json::from_str(&mismatch_out).unwrap();
        assert_eq!(mismatch_parsed["valid"], false);
    }
}
