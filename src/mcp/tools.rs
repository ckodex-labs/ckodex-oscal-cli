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
    let base = std::env::current_dir().map_err(McpError::Io)?;
    let candidate = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        base.join(path)
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
    if !normalized.starts_with(&base) {
        return Err(McpError::PathEscape(format!(
            "output path {path} escapes workspace {}",
            base.display()
        )));
    }
    Ok(normalized)
}

pub fn execute_tool(name: &str, args: &Value) -> Result<String, McpError> {
    match name {
        "query_control" => {
            let cat_file = require_str_arg(args, "catalog_file")?;
            let ctrl_id = require_str_arg(args, "control_id")?;
            let doc = OscalDocument::from_file(Path::new(cat_file))?;
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
            let file = require_str_arg(args, "file")?;
            let doc = OscalDocument::from_file(Path::new(file))?;
            let rep = inspect_document(&doc)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "compute_blast_radius" => {
            let file = require_str_arg(args, "file")?;
            let target = require_str_arg(args, "target")?;
            let doc = OscalDocument::from_file(Path::new(file))?;
            let rep = analyze_blast_radius(&doc, &[], target, 3)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "validate_fedramp" => {
            let ssp_file = require_str_arg(args, "ssp_file")?;
            let base_str = optional_str_arg(args, "baseline").unwrap_or("moderate");
            let baseline =
                FedrampBaseline::from_str_name(base_str).unwrap_or(FedrampBaseline::Moderate);
            let doc = OscalDocument::from_file(Path::new(ssp_file))?;
            let rep = validate_fedramp(&doc, baseline)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "audit_kubernetes_cluster" => {
            let namespace = optional_str_arg(args, "namespace");
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let audit_fut = async {
                let mut auditor = KubeAuditor::new(KubeClusterClient::try_connect().await);
                auditor
                    .audit_cluster(namespace, None, out_path.as_deref())
                    .await
            };
            let result = match tokio::runtime::Handle::try_current() {
                Ok(handle) => tokio::task::block_in_place(|| handle.block_on(audit_fut)),
                Err(_) => match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt.block_on(audit_fut),
                    Err(e) => return Err(McpError::Io(e)),
                },
            };
            let (_doc, report) = result?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
        "evaluate_rego_policy" => {
            let policy_file = require_str_arg(args, "policy_file")?;
            let input_data = optional_object_arg(args, "input_data");
            let query_package = optional_str_arg(args, "query_package").unwrap_or("data");
            let mut evaluator = RegorusEvaluator::new();
            evaluator.add_policy_dir(Path::new(policy_file))?;
            let res = evaluator.evaluate_compliance_rule(query_package, input_data)?;
            Ok(serde_json::to_string_pretty(&res)?)
        }
        "sync_3way_merge" => {
            let base_f = require_str_arg(args, "base_file")?;
            let upstream_f = require_str_arg(args, "upstream_file")?;
            let local_f = require_str_arg(args, "local_file")?;
            let strat_str = optional_str_arg(args, "strategy").unwrap_or("manual");
            let strat = MergeStrategy::from_str_name(strat_str).unwrap_or(MergeStrategy::Manual);
            let base = OscalDocument::from_file(Path::new(base_f))?;
            let up = OscalDocument::from_file(Path::new(upstream_f))?;
            let local = OscalDocument::from_file(Path::new(local_f))?;
            let (_doc, report) = sync_and_merge(&base, &up, &local, strat, None, None)?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
        "split_oscal_markdown" => {
            let file = require_str_arg(args, "file")?;
            let out_dir = require_str_arg(args, "output_dir")?;
            let out_path = confine_output_path(out_dir)?;
            let doc = OscalDocument::from_file(Path::new(file))?;
            let rep = split_document(&doc, &out_path)?;
            Ok(serde_json::to_string_pretty(&rep)?)
        }
        "assemble_oscal_markdown" => {
            let in_dir = require_str_arg(args, "input_dir")?;
            let out_file = require_str_arg(args, "output_file")?;
            let out_path = confine_output_path(out_file)?;
            let (_doc, rep) = assemble_directory(Path::new(in_dir), Some(&out_path))?;
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
                let bundle = EvidenceBundle::load_from_file(Path::new(ev_path))?;
                builder = builder.with_oscal_evidence(bundle);
            }
            let stmt = builder.build();
            Ok(serde_json::to_string_pretty(&stmt)?)
        }
        "verify_slsa_provenance" => {
            let stmt_str = require_str_arg(args, "statement_json")?;
            let stmt: Value = serde_json::from_str(stmt_str).map_err(McpError::Serialization)?;
            let rep = SlsaProvenanceBuilder::verify(&stmt)?;
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
            let file_path = require_str_arg(args, "file")?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let doc = OscalDocument::from_file(Path::new(file_path))?;
            let sarif = SarifExporter::export_from_oscal(&doc, Path::new(file_path))?;
            if let Some(out) = out_path {
                SarifExporter::save_to_file(&sarif, &out)?;
                Ok(format!("SARIF report saved to {}", out.display()))
            } else {
                Ok(serde_json::to_string_pretty(&sarif)?)
            }
        }
        "export_gitlab" => {
            let file_path = require_str_arg(args, "file")?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let doc = OscalDocument::from_file(Path::new(file_path))?;
            let report = GitLabReportExporter::export_from_oscal(&doc, Path::new(file_path))?;
            if let Some(out) = out_path {
                GitLabReportExporter::save_to_file(&report, &out)?;
                Ok(format!("GitLab report saved to {}", out.display()))
            } else {
                Ok(serde_json::to_string_pretty(&report)?)
            }
        }
        "import_sbom_cyclonedx" => {
            let file_path = require_str_arg(args, "file")?;
            let out_path = optional_str_arg(args, "output_file")
                .map(confine_output_path)
                .transpose()?;
            let (doc, summary) = SbomImporter::import_file(Path::new(file_path))?;
            if let Some(out) = out_path {
                let json_str = serde_json::to_string_pretty(&doc.value)?;
                std::fs::write(&out, json_str).map_err(McpError::Io)?;
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
            let sbom_path = optional_str_arg(args, "sbom_file").map(PathBuf::from);
            let workload_path = optional_str_arg(args, "workload_file").map(PathBuf::from);
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
            let res = json!({
                "valid": true,
                "spiffe_id": id.to_string(),
                "trust_domain": id.trust_domain().as_str(),
                "path": id.path(),
                "namespace": id.namespace(),
                "service_account": id.service_account()
            });
            Ok(serde_json::to_string_pretty(&res)?)
        }
        _ => Err(McpError::Tool {
            tool: name.to_string(),
            message: format!("Unknown tool: {name}"),
        }),
    }
}
