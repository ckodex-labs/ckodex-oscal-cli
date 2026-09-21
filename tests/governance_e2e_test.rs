use mizan_oscal::{
    document::{
        EmbeddedCatalogProvider, EnterpriseCatalogBuilder, Jurisdiction, PipelineConfig,
        PipelineOrchestrator,
    },
    fabric::{
        ResourceType, RootFabricEngine, SpiffeId, SpireWorkloadAttestor, TenantContext, TenantId,
        TenantMetadata, TrustDomain, UserId,
    },
};
use std::fs;

#[test]
fn test_e2e_tri_jurisdiction_catalog_federation() {
    // 1. US NIST 800-53
    let us_catalog =
        EmbeddedCatalogProvider::get_catalog(Jurisdiction::UsNist800_53Rev5).expect("us catalog");
    let us_json = serde_json::to_string(&us_catalog.value).unwrap();
    assert!(us_json.contains("NIST Special Publication 800-53"));

    // 2. CA CCCS ITSG-33
    let ca_catalog =
        EmbeddedCatalogProvider::get_catalog(Jurisdiction::CaItsg33Pbmm).expect("ca catalog");
    let ca_json = serde_json::to_string(&ca_catalog.value).unwrap();
    assert!(ca_json.contains("ITSG-33"));

    // 3. EU BSI C5 / ISO 27001
    let eu_catalog =
        EmbeddedCatalogProvider::get_catalog(Jurisdiction::EuEucsIso27001).expect("eu catalog");
    let eu_json = serde_json::to_string(&eu_catalog.value).unwrap();
    assert!(eu_json.contains("EUCS") || eu_json.contains("ISO"));

    // 4. Enterprise Extension
    let extended = EnterpriseCatalogBuilder::new("Fintech Global Hardened Baseline")
        .add_custom_control(
            "fin-1",
            "Multi-Party Cryptographic Approval",
            "Requires m-of-n threshold signatures for all compliance deployments.",
            "ac",
        )
        .add_custom_control(
            "fin-2",
            "Continuous Immutable Ledger Proofs",
            "Transactions must be anchored into append-only cryptographic CAS logs.",
            "au",
        )
        .build(None)
        .expect("build extended catalog");
    let ext_json = serde_json::to_string(&extended.value).unwrap();
    assert!(ext_json.contains("Fintech Global Hardened Baseline"));
    assert!(ext_json.contains("fin-1"));
    assert!(ext_json.contains("fin-2"));
}

#[test]
fn test_e2e_multi_tenant_compliance_lifecycle() {
    // Phase 1: Root Fabric Initialisation
    let td = TrustDomain::new("meridian.runbase.io").unwrap();
    let mut engine = RootFabricEngine::new().with_spiffe(td.clone());

    // Phase 2: Tenant Registration & Context
    let tenant_id = TenantId::new("fin-corp").unwrap();
    let user_id = UserId::new("ciso@fincorp.io");
    let ctx = TenantContext::new(tenant_id.clone(), user_id)
        .with_roles(vec!["ComplianceArchitect".to_string()]);

    let tenant_meta = TenantMetadata {
        tenant_id: tenant_id.clone(),
        display_name: "FinCorp Global Systems".to_string(),
        tier: "Enterprise".to_string(),
        max_storage_bytes: 500 * 1024 * 1024 * 1024,
        current_storage_bytes: 0,
        default_jurisdiction: "us".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    engine.tenant_manager.register_tenant(tenant_meta);

    // Phase 3: SVID Attestation
    let attestor = SpireWorkloadAttestor::new(td);
    let spiffe_id: SpiffeId = "spiffe://meridian.runbase.io/ns/fin-corp/sa/auditor"
        .parse()
        .unwrap();
    assert!(attestor.verify_spiffe_id(&spiffe_id).is_ok());

    // Phase 4: Storing System Security Plan (SSP) in Isolated DataStore
    let ssp_json = r#"{
        "system-security-plan": {
            "id": "ssp-fincorp-prod",
            "title": "FinCorp Production Core",
            "status": "APPROVED"
        }
    }"#;
    let stored_doc = engine
        .store_document_guarded(
            &ctx,
            "prod",
            "ssp-fincorp-prod",
            ResourceType::Ssp,
            ssp_json,
        )
        .expect("store ssp");

    assert_eq!(stored_doc.version, 1);

    // Phase 5: Executing Automated Compliance Pipeline
    let temp_out = std::env::temp_dir().join(format!("mizan-e2e-out-{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_out);

    let config = PipelineConfig {
        jurisdiction: Jurisdiction::UsNist800_53Rev5,
        sbom_path: None,
        workload_path: None,
        rule_ids: vec![
            "cis-k8s-5.2.1".to_string(),
            "cis-k8s-5.2.6".to_string(),
            "fedramp-ac-2".to_string(),
        ],
        output_dir: temp_out.clone(),
        subject_name: "fincorp-workload:v1.2.0".to_string(),
        subject_digest: "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .to_string(),
    };

    let report = PipelineOrchestrator::run(config).expect("run pipeline orchestrator");
    assert!(report.all_passed);
    assert!(report.slsa_provenance_path.exists());
    assert!(report.sarif_report_path.exists());
    assert!(report.gitlab_report_path.exists());

    let _ = fs::remove_dir_all(temp_out);
}
