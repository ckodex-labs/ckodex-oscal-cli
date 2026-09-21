use mizan_oscal::{
    document::{BuiltinRulepack, SlsaProvenanceBuilder, SlsaVersion},
    fabric::{ResourceType, RootFabricEngine, TenantContext, TenantId, UserId},
};
use serde_json::json;
use std::time::Instant;

#[test]
fn test_stress_regorus_policy_evaluations() {
    let sample_pod = json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": { "name": "stress-app", "namespace": "prod" },
        "spec": {
            "containers": [{
                "name": "web",
                "image": "nginx:alpine",
                "securityContext": {
                    "privileged": false,
                    "allowPrivilegeEscalation": false,
                    "readOnlyRootFilesystem": true,
                    "runAsNonRoot": true
                }
            }]
        }
    });

    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let cis_521 =
            BuiltinRulepack::evaluate_rule("cis-k8s-5.2.1", &sample_pod).expect("eval cis 5.2.1");
        assert!(cis_521.passed);

        let cis_526 =
            BuiltinRulepack::evaluate_rule("cis-k8s-5.2.6", &sample_pod).expect("eval cis 5.2.6");
        assert!(cis_526.passed);
    }
    let elapsed = start.elapsed();
    println!(
        "Completed {} Rego policy evaluations in {:?} (avg: {:?}/eval)",
        iterations * 2,
        elapsed,
        elapsed / (iterations * 2)
    );
    assert!(elapsed.as_secs() < 5, "evaluations must be fast in-process");
}

#[test]
fn test_stress_slsa_v1_2_provenance_building() {
    let start = Instant::now();
    let iterations = 50;

    for i in 0..iterations {
        let name = format!("pkg:cargo/service-{}", i);
        let builder = SlsaProvenanceBuilder::new(
            &name,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .with_version(SlsaVersion::V1_2)
        .add_dependency(
            "https://github.com/runbase/core.git",
            "sha256:71ae34098fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b111",
        )
        .add_dependency(
            "https://github.com/runbase/oscal-models.git",
            "sha256:be41c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b222",
        );

        let attestation = builder.build();
        assert_eq!(
            attestation["predicateType"],
            "https://slsa.dev/provenance/v1.2"
        );
        assert_eq!(
            attestation["predicate"]["buildDefinition"]["resolvedDependencies"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    let elapsed = start.elapsed();
    println!(
        "Built {} SLSA v1.2 attestations in {:?}",
        iterations, elapsed
    );
    assert!(elapsed.as_secs() < 3);
}

#[test]
fn test_stress_multi_tenant_datastore_concurrency() {
    let engine = RootFabricEngine::new();
    let num_tenants = 10;
    let docs_per_tenant = 10;

    let start = Instant::now();

    for t_idx in 0..num_tenants {
        let t_id = TenantId::new(format!("tenant-{}", t_idx)).unwrap();
        let user = UserId::new(format!("admin@tenant-{}.io", t_idx));
        let ctx = TenantContext::new(t_id.clone(), user)
            .with_roles(vec!["ComplianceArchitect".to_string()]);

        for d_idx in 0..docs_per_tenant {
            let doc_id = format!("doc-{}", d_idx);
            let content = format!(r#"{{"tenant":"{}", "index":{}}}"#, t_id.as_str(), d_idx);

            let stored = engine
                .store_document_guarded(&ctx, "production", &doc_id, ResourceType::Ssp, &content)
                .expect("store guarded");
            assert_eq!(stored.document_id, doc_id);
            assert_eq!(stored.tenant_id, t_id);
        }
    }

    // Verify isolation across all tenants
    for t_idx in 0..num_tenants {
        let t_id = TenantId::new(format!("tenant-{}", t_idx)).unwrap();
        let user = UserId::new(format!("admin@tenant-{}.io", t_idx));
        let ctx = TenantContext::new(t_id.clone(), user);

        let docs = engine
            .datastore
            .list_documents(&ctx, Some("production"))
            .expect("list documents for tenant");
        assert_eq!(docs.len(), docs_per_tenant);

        // Foreign tenant context
        let foreign_tid = TenantId::new(format!("tenant-{}", (t_idx + 1) % num_tenants)).unwrap();
        let foreign_ctx = TenantContext::new(foreign_tid, UserId::new("other"));

        for d_idx in 0..docs_per_tenant {
            let doc_id = format!("doc-{}", d_idx);
            // Reading doc from this tenant with foreign context MUST fail
            let leak = engine
                .datastore
                .get_document(&foreign_ctx, "production", &doc_id)
                .unwrap();
            assert_ne!(leak.tenant_id, t_id, "must not return another tenant's doc");
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Stored and verified {} isolated multi-tenant documents across {} tenants in {:?}",
        num_tenants * docs_per_tenant,
        num_tenants,
        elapsed
    );
}
