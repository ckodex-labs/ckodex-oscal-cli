use mizan_oscal::fabric::{
    OidcProviderConfig, OidcTokenValidator, ResourceType, RootFabricEngine, SpiffeId,
    SpireWorkloadAttestor, TenantContext, TenantId, TrustDomain, UserId,
};

#[test]
fn test_spiffe_spire_workload_identity_attestation() {
    let td = TrustDomain::new("meridian.runbase.io").expect("valid trust domain");
    let attestor = SpireWorkloadAttestor::new(td.clone());

    let valid_id: SpiffeId = "spiffe://meridian.runbase.io/ns/prod/sa/mizan-auditor"
        .parse()
        .expect("parse valid spiffe id");

    assert_eq!(valid_id.trust_domain(), &td);
    assert_eq!(valid_id.namespace(), Some("prod"));
    assert_eq!(valid_id.service_account(), Some("mizan-auditor"));
    assert!(attestor.verify_spiffe_id(&valid_id).is_ok());

    let external_id: SpiffeId = "spiffe://untrusted-domain.com/sa/rogue"
        .parse()
        .expect("parse external id");
    assert!(attestor.verify_spiffe_id(&external_id).is_err());
}

#[test]
fn test_oidc_claims_and_role_extraction() {
    let cfg = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench");
    let validator = OidcTokenValidator::new(cfg);

    // Payload: {"iss":"https://auth.runbase.io","sub":"user_884","aud":["mizan-workbench"],"exp":2100000000,"iat":1700000000,"tenant_id":"corp-fintech","roles":["ComplianceArchitect"]}
    let token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL2F1dGgucnVuYmFzZS5pbyIsInN1YiI6InVzZXJfODg0IiwiYXVkIjpbIm1pemFuLXdvcmtiZW5jaCJdLCJleHAiOjIxMDAwMDAwMDAsImlhdCI6MTcwMDAwMDAwMCwidGVuYW50X2lkIjoiY29ycC1maW50ZWNoIiwicm9sZXMiOlsiQ29tcGxpYW5jZUFyY2hpdGVjdCJdfQ.";

    let claims = validator
        .decode_and_validate(token, 1750000000)
        .expect("token decode");
    assert_eq!(claims.sub, "user_884");
    assert_eq!(claims.tenant_id, Some("corp-fintech".to_string()));
    assert_eq!(claims.roles, vec!["ComplianceArchitect".to_string()]);
}

#[test]
fn test_multi_tenant_datastore_isolation_and_rbac() {
    let engine = RootFabricEngine::new();

    let tenant_a = TenantId::new("corp-alpha").unwrap();
    let user_a = UserId::new("alice@alpha.io");
    let ctx_a = TenantContext::new(tenant_a.clone(), user_a)
        .with_roles(vec!["ComplianceArchitect".to_string()]);

    let tenant_b = TenantId::new("corp-beta").unwrap();
    let user_b = UserId::new("bob@beta.io");
    let ctx_b =
        TenantContext::new(tenant_b.clone(), user_b).with_roles(vec!["Auditor".to_string()]);

    // 1. Tenant A writes a catalog to its isolated partition
    let stored = engine
        .store_document_guarded(
            &ctx_a,
            "security",
            "cat-alpha-01",
            ResourceType::Catalog,
            r#"{"title":"Alpha Proprietary Baseline"}"#,
        )
        .expect("tenant A store catalog");

    assert_eq!(stored.document_id, "cat-alpha-01");
    assert_eq!(stored.tenant_id, tenant_a);

    // 2. Tenant A can read it back
    let read_back = engine
        .datastore
        .get_document(&ctx_a, "security", "cat-alpha-01")
        .expect("tenant A read");
    assert_eq!(read_back.document_id, "cat-alpha-01");

    // 3. Tenant B CANNOT read Tenant A's document
    let cross_read_err = engine
        .datastore
        .get_document(&ctx_b, "security", "cat-alpha-01");
    assert!(
        cross_read_err.is_err(),
        "tenant B must not see tenant A document"
    );

    // 4. RBAC check: Tenant B (Auditor) CANNOT store a Catalog (only Architect/Admin can)
    let rbac_deny = engine.store_document_guarded(
        &ctx_b,
        "security",
        "cat-beta-01",
        ResourceType::Catalog,
        r#"{"title":"Beta Catalog"}"#,
    );
    assert!(rbac_deny.is_err(), "auditor cannot create catalog");
}

#[test]
fn test_spiffe_malformed_uris_and_schemes() {
    // Missing spiffe:// scheme
    assert!("https://meridian.runbase.io/ns/prod/sa/app"
        .parse::<SpiffeId>()
        .is_err());
    // Empty trust domain
    assert!("spiffe:///ns/prod/sa/app".parse::<SpiffeId>().is_err());
    // Missing path
    assert!("spiffe://meridian.runbase.io".parse::<SpiffeId>().is_err());
    // Trust domain with invalid characters (port colon or spaces)
    assert!("spiffe://meridian:8080/sa/test"
        .parse::<SpiffeId>()
        .is_err());
    assert!("spiffe://meridian domain/sa/test"
        .parse::<SpiffeId>()
        .is_err());
}

#[test]
fn test_oidc_expired_and_issuer_mismatch() {
    let cfg = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench");
    let validator = OidcTokenValidator::new(cfg);

    // Expired token (exp: 1000)
    let expired_token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL2F1dGgucnVuYmFzZS5pbyIsInN1YiI6InVzZXJfMSIsImF1ZCI6WyJtaXphbi13b3JrYmVuY2giXSwiZXhwIjoxMDAwLCJpYXQiOjkwMCwidGVuYW50X2lkIjoiZGVmYXVsdCIsInJvbGVzIjpbIlZpZXdlciJdfQ.";
    assert!(validator.decode_and_validate(expired_token, 5000).is_err());

    // Issuer mismatch token (iss: https://rogue-auth.com)
    let rogue_token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL3JvZ3VlLWF1dGguY29tIiwic3ViIjoidXNlcl8xIiwiYXVkIjpbIm1pemFuLXdvcmtiZW5jaCJdLCJleHAiOjIxMDAwMDAwMDAsImlhdCI6MTcwMDAwMDAwMCwidGVuYW50X2lkIjoiZGVmYXVsdCIsInJvbGVzIjpbIlZpZXdlciJdfQ.";
    assert!(validator
        .decode_and_validate(rogue_token, 1750000000)
        .is_err());

    // Malformed token without 3 segments
    assert!(validator
        .decode_and_validate("not-a-jwt", 1750000000)
        .is_err());
}

#[test]
fn test_tenant_context_isolation_assertion() {
    let t1 = TenantId::new("tenant-one").unwrap();
    let t2 = TenantId::new("tenant-two").unwrap();

    let ctx1 = TenantContext::new(t1.clone(), UserId::new("user-1"));

    // Same tenant assert passes
    assert!(ctx1.assert_same_tenant(&t1).is_ok());

    // Cross tenant assert fails closed with AccessDenied
    assert!(ctx1.assert_same_tenant(&t2).is_err());
}
