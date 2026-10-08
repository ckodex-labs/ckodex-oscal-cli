use mizan_oscal::fabric::{
    JwtSvid, OidcClaims, OidcError, OidcProviderConfig, OidcTokenValidator, ResourceType,
    RootFabricEngine, SpiffeError, SpiffeId, SpireWorkloadAttestor, TenantContext, TenantId,
    TrustDomain, UserId,
};

#[test]
fn test_spiffe_spire_workload_identity_attestation() {
    let td = TrustDomain::new("meridian.runbase.io").expect("valid trust domain");
    let secret = b"spire-shared-fabric-secret-key-32b";
    let attestor = SpireWorkloadAttestor::new(td.clone()).with_shared_secret(secret);

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

    // Cryptographic JWT SVID validation
    let valid_svid = JwtSvid::sign_hs256(
        valid_id.clone(),
        vec!["mizan-workbench".to_string()],
        2100000000,
        1700000000,
        secret,
    )
    .expect("sign svid");
    assert!(attestor.verify_jwt_svid(&valid_svid, 1750000000).is_ok());

    // Rejection of alg: none JWT SVID
    let none_svid = JwtSvid {
        spiffe_id: valid_id,
        audience: vec!["mizan-workbench".to_string()],
        expires_at: 2100000000,
        issued_at: 1700000000,
        token: "eyJhbGciOiJub25lIn0.eyJzdWIiOiJzcGlmZmU6Ly9tZXJpZGlhbi5ydW5iYXNlLmlvL25zL3Byb2Qvc2EvbWl6YW4tYXVkaXRvciJ9.".to_string(),
    };
    let err_none = attestor.verify_jwt_svid(&none_svid, 1750000000);
    assert!(matches!(err_none, Err(SpiffeError::UnsafeAlgorithmNone)));

    // Rejection of corrupted signature on SVID
    let mut tampered_svid = valid_svid;
    tampered_svid.token.push_str("tampered");
    let err_tampered = attestor.verify_jwt_svid(&tampered_svid, 1750000000);
    assert!(matches!(err_tampered, Err(SpiffeError::InvalidSignature)));
}

#[test]
fn test_oidc_claims_and_role_extraction() {
    let secret = b"meridian-test-signing-secret-key-32b";
    let cfg = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench")
        .with_shared_secret(secret);
    let validator = OidcTokenValidator::new(cfg);

    let claims = OidcClaims {
        iss: "https://auth.runbase.io".to_string(),
        sub: "user_884".to_string(),
        aud: vec!["mizan-workbench".to_string()],
        exp: 2100000000,
        iat: 1700000000,
        email: Some("architect@runbase.io".to_string()),
        tenant_id: Some("corp-fintech".to_string()),
        groups: vec![],
        roles: vec!["ComplianceArchitect".to_string()],
        name: None,
    };

    // 1. Signed HS256 token verification passes
    let token = OidcTokenValidator::sign_hs256(&claims, secret).expect("token signature");
    let decoded = validator
        .decode_and_validate(&token, 1750000000)
        .expect("token decode");
    assert_eq!(decoded.sub, "user_884");
    assert_eq!(decoded.tenant_id, Some("corp-fintech".to_string()));
    assert_eq!(decoded.roles, vec!["ComplianceArchitect".to_string()]);

    // 2. Strict mode categorically rejects alg: none tokens
    let none_token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL2F1dGgucnVuYmFzZS5pbyIsInN1YiI6InVzZXJfODg0IiwiYXVkIjpbIm1pemFuLXdvcmtiZW5jaCJdLCJleHAiOjIxMDAwMDAwMDAsImlhdCI6MTcwMDAwMDAwMCwidGVuYW50X2lkIjoiY29ycC1maW50ZWNoIiwicm9sZXMiOlsiQ29tcGxpYW5jZUFyY2hpdGVjdCJdfQ.";
    let err_none = validator.decode_and_validate(none_token, 1750000000);
    assert!(matches!(err_none, Err(OidcError::UnsafeAlgorithmNone)));

    // 3. Forged signature fails closed with InvalidSignature
    let rogue_token =
        OidcTokenValidator::sign_hs256(&claims, b"attacker-fabricated-key").expect("rogue token");
    let err_rogue = validator.decode_and_validate(&rogue_token, 1750000000);
    assert!(matches!(err_rogue, Err(OidcError::InvalidSignature)));

    // 4. Missing signature fails closed with MissingSignature
    let missing_sig_token = format!(
        "{}.",
        token.rsplit_once('.').map(|(prefix, _)| prefix).unwrap()
    );
    let err_missing = validator.decode_and_validate(&missing_sig_token, 1750000000);
    assert!(matches!(err_missing, Err(OidcError::MissingSignature)));

    // 5. Unauthenticated inspection can inspect claims for pre-routing without authorization
    let (header, unauth_claims) =
        OidcTokenValidator::inspect_unauthenticated(&token).expect("inspect claims");
    assert_eq!(header.alg, "HS256");
    assert_eq!(unauth_claims.sub, "user_884");
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
    assert!(
        "https://meridian.runbase.io/ns/prod/sa/app"
            .parse::<SpiffeId>()
            .is_err()
    );
    // Empty trust domain
    assert!("spiffe:///ns/prod/sa/app".parse::<SpiffeId>().is_err());
    // Missing path
    assert!("spiffe://meridian.runbase.io".parse::<SpiffeId>().is_err());
    // Trust domain with invalid characters (port colon or spaces)
    assert!(
        "spiffe://meridian:8080/sa/test"
            .parse::<SpiffeId>()
            .is_err()
    );
    assert!(
        "spiffe://meridian domain/sa/test"
            .parse::<SpiffeId>()
            .is_err()
    );
}

#[test]
fn test_oidc_expired_and_issuer_mismatch() {
    let secret = b"test-verification-secret-32-bytes";
    let cfg = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench")
        .with_shared_secret(secret);
    let validator = OidcTokenValidator::new(cfg);

    // Expired token (exp: 1000)
    let expired_claims = OidcClaims {
        iss: "https://auth.runbase.io".to_string(),
        sub: "user_1".to_string(),
        aud: vec!["mizan-workbench".to_string()],
        exp: 1000,
        iat: 900,
        email: None,
        tenant_id: Some("default".to_string()),
        groups: vec![],
        roles: vec!["Viewer".to_string()],
        name: None,
    };
    let expired_token =
        OidcTokenValidator::sign_hs256(&expired_claims, secret).expect("sign expired");
    let err_exp = validator.decode_and_validate(&expired_token, 5000);
    assert!(matches!(err_exp, Err(OidcError::TokenExpired(1000, 5000))));

    // Issuer mismatch token (iss: https://rogue-auth.com)
    let rogue_claims = OidcClaims {
        iss: "https://rogue-auth.com".to_string(),
        sub: "user_1".to_string(),
        aud: vec!["mizan-workbench".to_string()],
        exp: 2100000000,
        iat: 1700000000,
        email: None,
        tenant_id: Some("default".to_string()),
        groups: vec![],
        roles: vec!["Viewer".to_string()],
        name: None,
    };
    let rogue_token = OidcTokenValidator::sign_hs256(&rogue_claims, secret).expect("sign rogue");
    let err_iss = validator.decode_and_validate(&rogue_token, 1750000000);
    assert!(matches!(err_iss, Err(OidcError::IssuerMismatch { .. })));

    // Malformed token without 3 segments
    assert!(matches!(
        validator.decode_and_validate("not-a-jwt", 1750000000),
        Err(OidcError::MalformedToken)
    ));
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
