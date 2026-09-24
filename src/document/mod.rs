pub mod attestation;
pub mod authoring;
pub mod blast_radius;
pub mod capsule;
pub mod cas;
pub mod catalog;
pub mod converter;
pub mod daemon;
pub mod dedup;
pub mod diff;
pub mod evidence;
pub mod export;
pub mod federation;
pub mod fedramp;
pub mod fix;
pub mod fsm;
pub mod init;
pub mod inspector;
pub mod k8s;
pub mod linter;
pub mod parser;
pub mod pipeline;
pub mod policy;
pub mod reconciler;
pub mod resolver;
pub mod sbom;
pub mod schema;
pub mod sync;
pub mod tabular;
pub mod tx;
pub mod validator;
pub mod waiver;

pub use attestation::{SlsaProvenanceBuilder, SlsaVerificationReport, SlsaVersion};
pub use authoring::{
    AssembleReport, SplitReport, TemplateReport, assemble_directory, scaffold_template,
    split_document,
};
pub use blast_radius::{BlastRadiusReport, ImpactedNode, analyze_blast_radius};
pub use capsule::{CapsuleExporter, CapsuleReport};
pub use cas::{CasStats, CasStore};
pub use catalog::{EmbeddedCatalogProvider, EnterpriseCatalogBuilder, Jurisdiction};
pub use converter::convert_document;
pub use daemon::{ComplianceDaemon, DaemonConfig, DaemonCycleReport};
pub use dedup::{DedupReport, deduplicate_document};
pub use diff::{DocumentDiffReport, diff_documents};
pub use evidence::{
    EvidenceBundle, EvidenceVerificationReport, ObservationProof, SignatureInfo,
    compute_merkle_root,
};
pub use export::{
    GitLabReportExporter, GitLabSecurityReport, SarifExporter, SarifReport, ShieldcnBadgeConfig,
    ShieldcnBadgeExporter,
};
pub use federation::{ComplianceFederator, FederationChainReport};
pub use fedramp::{FedrampBaseline, FedrampRuleFinding, FedrampValidationReport, validate_fedramp};
pub use fix::{FixEngine, FixReport};
pub use fsm::{ComplianceState, EvidenceLevel, FsmEvent, FsmHistory, FsmRuntime, TransitionRecord};
pub use init::{DiscoveredAsset, InitReport, RepoInitializer};
pub use inspector::{DocumentSummary, inspect_document};
pub use k8s::{KubeAuditReport, KubeAuditor, KubeClusterClient};
pub use linter::{LintReport, lint_document};
pub use parser::{FileFormat, OscalDocument};
pub use pipeline::{PipelineConfig, PipelineExecutionReport, PipelineOrchestrator};
pub use policy::{
    BuiltinRulepack, PolicyCompileReport, PolicyEvaluationResult, PolicyIngestReport,
    PolicyRuleMeta, PolicyTarget, RegorusEvaluator, compile_policies, ingest_policy_results,
};
pub use reconciler::{ReconciliationReport, ReconciliationVerdict, reconcile_compliance};
pub use resolver::resolve_profile;
pub use sbom::{SbomImporter, SbomSummary};
pub use schema::{DocumentKind, SchemaRegistry};
pub use sync::{MergeConflict, MergeReport, MergeStrategy, sync_and_merge};
pub use tabular::{export_matrix_csv, export_to_csv, import_from_csv, sync_matrix_csv};
pub use tx::{
    ComplianceTransactionManager, MutationKind, TransactionReport, TransactionSession,
    TransactionStatus, WalEntry, WriteAheadLog,
};
pub use validator::{DiagnosticLevel, ValidationOptions, ValidationReport, validate_document};
pub use waiver::{DerogationLease, WaiverManager, WaiverStatus};

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CATALOG: &str = r#"{
  "catalog": {
    "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
    "metadata": {
      "title": "NIST Special Publication 800-53 Revision 5 Sample",
      "published": "2020-09-23T00:00:00Z",
      "last-modified": "2020-09-23T00:00:00Z",
      "version": "5.0.0",
      "oscal-version": "1.2.3"
    },
    "controls": [
      {
        "id": "ac-1",
        "class": "SP800-53",
        "title": "Policy and Procedures",
        "params": [
          {
            "id": "ac-01_prm_1",
            "label": "organization-defined frequency"
          }
        ]
      },
      {
        "id": "ac-2",
        "class": "SP800-53",
        "title": "Account Management",
        "params": [
          {
            "id": "ac-02_prm_1",
            "label": "account managers"
          }
        ]
      }
    ]
  }
}"#;

    const SAMPLE_PROFILE: &str = r#"{
  "profile": {
    "uuid": "996e3828-5690-4dc6-8c29-37399ff87fa7",
    "metadata": {
      "title": "FedRAMP Moderate Baseline Profile",
      "published": "2021-01-01T00:00:00Z",
      "last-modified": "2021-01-01T00:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.3"
    },
    "imports": [
      {
        "href": "catalog.json",
        "include-controls": [
          {
            "with-ids": ["ac-1"]
          }
        ]
      }
    ],
    "modify": {
      "set-parameters": [
        {
          "param-id": "ac-01_prm_1",
          "values": ["at least annually"]
        }
      ]
    }
  }
}"#;

    #[test]
    fn test_parse_and_validate_catalog() {
        let doc = OscalDocument::from_str(SAMPLE_CATALOG, None).expect("catalog should parse");
        assert_eq!(doc.kind, DocumentKind::Catalog);
        assert_eq!(
            doc.title(),
            Some("NIST Special Publication 800-53 Revision 5 Sample")
        );

        let report = validate_document(&doc, &ValidationOptions::default())
            .expect("validation should succeed");
        assert!(
            report.is_valid,
            "Catalog should be valid. Diagnostics: {:?}",
            report.diagnostics
        );
        assert!(report.schema_valid);
        assert!(report.constraints_valid);
    }

    #[test]
    fn test_parse_and_validate_profile() {
        let doc = OscalDocument::from_str(SAMPLE_PROFILE, None).expect("profile should parse");
        assert_eq!(doc.kind, DocumentKind::Profile);

        let report = validate_document(&doc, &ValidationOptions::default())
            .expect("validation should succeed");
        assert!(
            report.is_valid,
            "Profile should be valid. Diagnostics: {:?}",
            report.diagnostics
        );
    }

    #[test]
    fn test_convert_json_to_yaml_and_back() {
        let doc_json = OscalDocument::from_str(SAMPLE_CATALOG, None).expect("catalog should parse");
        let yaml_str =
            convert_document(&doc_json, FileFormat::Yaml, None).expect("convert to YAML");
        assert!(yaml_str.contains("catalog:"));
        assert!(yaml_str.contains("title:"));

        let doc_yaml = OscalDocument::from_str(&yaml_str, None).expect("YAML catalog should parse");
        assert_eq!(doc_yaml.kind, DocumentKind::Catalog);
        assert_eq!(
            doc_yaml.uuid(),
            Some("8b788647-767a-4ecb-ba3a-f2b7f719602a")
        );
    }

    #[test]
    fn test_inspect_document() {
        let doc = OscalDocument::from_str(SAMPLE_CATALOG, None).expect("catalog should parse");
        let summary = inspect_document(&doc).expect("inspect should succeed");
        assert_eq!(summary.kind, "catalog");
        assert_eq!(summary.stats.total_controls, 2);
        assert_eq!(summary.stats.controls_by_family.get("AC"), Some(&2));
        assert_eq!(summary.stats.total_params, 2);
    }

    #[test]
    fn test_diff_documents() {
        let doc_a = OscalDocument::from_str(SAMPLE_CATALOG, None).expect("doc a");
        let mut modified = serde_json::from_str::<serde_json::Value>(SAMPLE_CATALOG).unwrap();
        // remove ac-2 and change title
        modified["catalog"]["metadata"]["title"] = serde_json::json!("Revised Catalog");
        modified["catalog"]["controls"]
            .as_array_mut()
            .unwrap()
            .pop();

        let doc_b =
            OscalDocument::from_str(&serde_json::to_string(&modified).unwrap(), None).unwrap();
        let diff = diff_documents(&doc_a, &doc_b).expect("diff should succeed");

        assert_eq!(diff.metadata_changes.len(), 1);
        assert_eq!(diff.metadata_changes[0].field, "title");
        assert_eq!(diff.controls_removed, vec!["ac-2"]);
        assert!(diff.controls_added.is_empty());
    }

    #[test]
    fn test_lint_and_fix() {
        let invalid_doc_str = r#"{
  "catalog": {
    "uuid": "invalid-uuid-format",
    "metadata": {
      "title": "Untimed Catalog",
      "version": "1.0.0"
    }
  }
}"#;
        let mut doc = OscalDocument::from_str(invalid_doc_str, None).expect("doc should parse");
        let report = lint_document(&mut doc, true).expect("lint and fix");
        assert!(report.fixed_count >= 2);

        // Check that UUID is now valid
        let fixed_uuid = doc.uuid().expect("uuid present");
        assert!(uuid::Uuid::parse_str(fixed_uuid).is_ok());

        // Check that last-modified was added
        let last_mod = doc.last_modified().expect("last-modified present");
        assert!(chrono::DateTime::parse_from_rfc3339(last_mod).is_ok());
    }

    #[test]
    fn test_profile_resolution() {
        let tmp_dir = std::env::temp_dir().join(format!("oscal-test-{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let cat_path = tmp_dir.join("catalog.json");
        let prof_path = tmp_dir.join("profile.json");
        let res_path = tmp_dir.join("resolved.json");

        std::fs::write(&cat_path, SAMPLE_CATALOG).unwrap();
        std::fs::write(&prof_path, SAMPLE_PROFILE).unwrap();

        let profile_doc = OscalDocument::from_file(&prof_path).unwrap();
        let resolved = resolve_profile(&profile_doc, Some(&res_path)).unwrap();

        assert_eq!(resolved.kind, DocumentKind::Catalog);
        let summary = inspect_document(&resolved).unwrap();
        // ac-1 selected, ac-2 excluded
        assert_eq!(summary.stats.total_controls, 1);

        // Verify parameter override was applied
        let ctrl = &resolved.root_object().unwrap()["controls"][0];
        assert_eq!(ctrl["id"], "ac-1");
        assert_eq!(ctrl["params"][0]["values"][0], "at least annually");

        std::fs::remove_dir_all(tmp_dir).unwrap();
    }

    #[test]
    fn test_blast_radius_analysis() {
        let cat_doc = OscalDocument::from_str(SAMPLE_CATALOG, None).unwrap();
        let prof_doc = OscalDocument::from_str(SAMPLE_PROFILE, None).unwrap();

        let report = analyze_blast_radius(&cat_doc, &[prof_doc], "ac-1", 4)
            .expect("blast radius analysis should succeed");

        assert_eq!(report.target_id, "ac-1");
        assert_eq!(report.target_kind, "Control");
        assert!(!report.direct_dependents.is_empty());
        assert!(report.documents_analyzed.len() >= 2);
    }

    #[test]
    fn test_deduplicate_document() {
        let duplicate_catalog = r#"{
  "catalog": {
    "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
    "metadata": {
      "title": "Duplicate Sample",
      "version": "1.0.0",
      "parties": [
        { "uuid": "00000000-0000-0000-0000-000000000001", "name": "Alice Corp", "email-addresses": ["alice@example.com"] },
        { "uuid": "00000000-0000-0000-0000-000000000002", "name": "Alice Corp", "email-addresses": ["alice@example.com"] }
      ]
    },
    "controls": [
      { "id": "ac-1", "title": "Access Control" },
      { "id": "ac-1", "title": "Access Control", "params": [{ "id": "ac-1_prm_1" }] }
    ]
  }
}"#;
        let doc = OscalDocument::from_str(duplicate_catalog, None).unwrap();
        let (deduped, report) = deduplicate_document(&doc, None).expect("dedup should succeed");

        assert_eq!(report.parties_deduplicated, 1);
        assert_eq!(report.controls_deduplicated, 1);
        assert_eq!(report.total_duplicates_removed, 2);

        let cat = deduped.root_object().unwrap();
        assert_eq!(cat["controls"].as_array().unwrap().len(), 1);
        assert_eq!(cat["metadata"]["parties"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_reconciliation() {
        let ssp_json = r#"{
  "system-security-plan": {
    "uuid": "11111111-1111-1111-1111-111111111111",
    "metadata": { "title": "Sample SSP", "version": "1.0.0" },
    "system-implementation": {
      "components": [
        { "uuid": "22222222-2222-2222-2222-222222222222", "type": "software", "title": "PostgreSQL", "props": [{ "name": "version", "value": "15.2" }] }
      ]
    }
  }
}"#;
        let ssp_doc = OscalDocument::from_str(ssp_json, None).unwrap();
        let inv_val = serde_json::json!({
            "components": [
                { "name": "PostgreSQL", "version": "15.4" },
                { "name": "Redis", "version": "7.0" }
            ]
        });

        let results_json = r#"{
  "assessment-results": {
    "uuid": "33333333-3333-3333-3333-333333333333",
    "metadata": { "title": "Audit", "version": "1.0.0" },
    "results": [
      {
        "uuid": "44444444-4444-4444-4444-444444444444",
        "title": "Result 1",
        "findings": [
          { "id": "find-01", "title": "Unpatched TLS", "props": [{ "name": "severity", "value": "high" }] }
        ]
      }
    ]
  }
}"#;
        let results_doc = OscalDocument::from_str(results_json, None).unwrap();
        let poam_json = r#"{
  "plan-of-action-and-milestones": {
    "uuid": "55555555-5555-5555-5555-555555555555",
    "metadata": { "title": "POA&M", "version": "1.0.0" },
    "poam-items": []
  }
}"#;
        let poam_doc = OscalDocument::from_str(poam_json, None).unwrap();

        let report = reconcile_compliance(
            Some(&ssp_doc),
            Some(&inv_val),
            Some(&results_doc),
            Some(&poam_doc),
        )
        .expect("reconciliation should run");

        assert_eq!(report.verdict, ReconciliationVerdict::CriticalUnmitigated);
        let comp_rec = report.component_reconciliation.unwrap();
        assert_eq!(comp_rec.matching_components, vec!["postgresql"]);
        assert_eq!(comp_rec.shadow_components.len(), 1); // Redis
        assert_eq!(comp_rec.version_drifts.len(), 1); // 15.2 vs 15.4

        let find_rec = report.finding_poam_reconciliation.unwrap();
        assert_eq!(find_rec.unmitigated_findings.len(), 1); // find-01
        assert!(!report.action_items.is_empty());
    }

    #[test]
    fn test_split_and_assemble_catalog() {
        let tmp_dir = std::env::temp_dir().join(format!("oscal-split-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let doc = OscalDocument::from_str(SAMPLE_CATALOG, None).unwrap();
        let split_rep = split_document(&doc, &tmp_dir).expect("split should succeed");
        assert!(split_rep.files_created >= 3);
        assert_eq!(split_rep.controls_split, 2);

        let (assembled, asm_rep) =
            assemble_directory(&tmp_dir, None).expect("assemble should succeed");
        assert_eq!(asm_rep.kind, "catalog");
        assert_eq!(asm_rep.controls_assembled, 2);
        assert!(asm_rep.is_valid);

        let cat_root = assembled.root_object().unwrap();
        assert_eq!(cat_root["controls"].as_array().unwrap().len(), 2);

        std::fs::remove_dir_all(tmp_dir).unwrap();
    }

    #[test]
    fn test_scaffold_template() {
        let (doc, rep) = scaffold_template("fedramp-moderate", "ssp", None, None, None)
            .expect("scaffold should succeed");
        assert_eq!(rep.kind, "system-security-plan");
        assert_eq!(doc.kind, DocumentKind::Ssp);
        assert!(doc.title().unwrap().contains("FedRAMP Moderate"));
    }

    #[test]
    fn test_policy_compile_and_ingest() {
        let tmp_dir =
            std::env::temp_dir().join(format!("oscal-policy-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let doc = OscalDocument::from_str(SAMPLE_CATALOG, None).unwrap();
        let compile_rep = compile_policies(&doc, PolicyTarget::All, &tmp_dir)
            .expect("policy compile should succeed");
        assert_eq!(compile_rep.policies_generated, 4); // 2 rego + 2 kyverno
        assert!(tmp_dir.join("ac_1.rego").exists());
        assert!(tmp_dir.join("kyverno_ac-1.yaml").exists());

        // Ingest mock log
        let mock_log = tmp_dir.join("eval.json");
        std::fs::write(
            &mock_log,
            r#"[
            {"control_id": "ac-1", "allowed": true, "message": "Policy passed"},
            {"control_id": "ac-2", "allowed": false, "message": "Account inactive for 90 days"}
        ]"#,
        )
        .unwrap();

        let (ar_doc, ingest_rep) =
            ingest_policy_results(&mock_log, "CI Run 1", None).expect("ingest should succeed");
        assert_eq!(ingest_rep.total_evaluated, 2);
        assert_eq!(ingest_rep.passed_checks, 1);
        assert_eq!(ingest_rep.failed_checks, 1);
        assert_eq!(ingest_rep.generated_findings, 1);
        assert_eq!(ar_doc.kind, DocumentKind::AssessmentResults);

        std::fs::remove_dir_all(tmp_dir).unwrap();
    }

    #[test]
    fn test_fedramp_validation() {
        let (ssp_doc, _) = scaffold_template("fedramp-moderate", "ssp", None, None, None).unwrap();
        let rep = validate_fedramp(&ssp_doc, FedrampBaseline::Moderate)
            .expect("fedramp validation should succeed");
        assert_eq!(rep.baseline, "Moderate");
        assert!(rep.total_rules_checked >= 4);
    }

    #[test]
    fn test_csv_export_and_import() {
        let cat_doc = OscalDocument::from_str(SAMPLE_CATALOG, None).unwrap();
        let csv_out = export_to_csv(&cat_doc, None).expect("export to csv should succeed");
        assert!(csv_out.contains("ac-1"));
        assert!(csv_out.contains("ac-2"));

        let imported_doc = import_from_csv(
            &csv_out,
            Some(DocumentKind::Catalog),
            Some("Imported"),
            None,
        )
        .expect("import from csv should succeed");
        assert_eq!(imported_doc.kind, DocumentKind::Catalog);
        let root = imported_doc.root_object().unwrap();
        let controls = root["controls"].as_array().unwrap();
        assert_eq!(controls.len(), 2);
    }

    #[test]
    fn test_three_way_merge_and_sync() {
        let base_json = r#"{
  "catalog": {
    "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
    "metadata": { "title": "Base Catalog", "version": "1.0.0" },
    "controls": [
      { "id": "ac-1", "title": "Policy and Procedures", "class": "SP800-53" },
      { "id": "ac-2", "title": "Account Management", "class": "SP800-53" }
    ]
  }
}"#;
        let base_doc = OscalDocument::from_str(base_json, None).unwrap();

        // Upstream adds ac-3 and updates ac-1
        let upstream_json = r#"{
  "catalog": {
    "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
    "metadata": { "title": "Base Catalog", "version": "1.1.0" },
    "controls": [
      { "id": "ac-1", "title": "Policy and Procedures (Upstream Updated)", "class": "SP800-53" },
      { "id": "ac-2", "title": "Account Management", "class": "SP800-53" },
      { "id": "ac-3", "title": "Access Enforcement", "class": "SP800-53" }
    ]
  }
}"#;
        let upstream_doc = OscalDocument::from_str(upstream_json, None).unwrap();

        // Local modifies ac-2 and adds custom-1
        let local_json = r#"{
  "catalog": {
    "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
    "metadata": { "title": "Base Catalog", "version": "1.0.0" },
    "controls": [
      { "id": "ac-1", "title": "Policy and Procedures", "class": "SP800-53" },
      { "id": "ac-2", "title": "Account Management (Local Customization)", "class": "SP800-53" },
      { "id": "custom-1", "title": "Internal Company Policy", "class": "CUSTOM" }
    ]
  }
}"#;
        let local_doc = OscalDocument::from_str(local_json, None).unwrap();

        let (merged_doc, report) = sync_and_merge(
            &base_doc,
            &upstream_doc,
            &local_doc,
            MergeStrategy::Manual,
            None,
            None,
        )
        .expect("sync and merge should succeed");

        assert_eq!(report.controls_merged, 4); // ac-1, ac-2, ac-3, custom-1
        assert_eq!(report.added_from_upstream, vec!["ac-3"]);
        assert_eq!(report.preserved_local_additions, vec!["custom-1"]);
        assert_eq!(report.updated_from_upstream, vec!["ac-1"]);
        assert_eq!(report.retained_local_modifications, vec!["ac-2"]);
        assert!(report.is_clean);

        let root = merged_doc.root_object().unwrap();
        let ctrls = root["controls"].as_array().unwrap();
        assert_eq!(ctrls.len(), 4);
    }

    #[tokio::test]
    async fn test_k8s_audit_and_assessment_emission() {
        let mut auditor = KubeAuditor::new(KubeClusterClient::new_mock());
        let result = auditor.audit_cluster(Some("production"), None, None).await;

        assert!(
            result.is_err(),
            "audit with a mock/disconnected client must now fail closed"
        );
    }
}
