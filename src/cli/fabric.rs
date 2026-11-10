#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_fabric(args: &crate::config::FabricCliArgs, format: OutputFormat) -> Result<()> {
    use crate::config::{FabricAction, FabricDatastoreAction, FabricTenantAction};
    use crate::fabric::{
        OidcProviderConfig, OidcTokenValidator, RootFabricEngine, SpiffeId, TenantId,
        TenantMetadata, TrustDomain,
    };

    let mut engine = RootFabricEngine::new().with_spiffe(
        TrustDomain::new("meridian.runbase.io")
            .map_err(|e| AppError::Configuration(e.to_string()))?,
    );

    match &args.action {
        FabricAction::Status => {
            let tenants = engine.tenant_manager.list_tenants();
            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json = serde_json::json!({
                        "status": "OPERATIONAL",
                        "trust_domain": "meridian.runbase.io",
                        "tenants_count": tenants.len(),
                        "tenants": tenants,
                        "datastore_engine": "Isolated High-Assurance Partitioned",
                        "spiffe_spire": "ACTIVE",
                        "oidc_provider": "READY"
                    });
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                _ => {
                    println!("Mizan Root Fabric · Enterprise Multi-Tenant Status");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Fabric Status:      OPERATIONAL");
                    println!("  Trust Domain:       meridian.runbase.io");
                    println!("  Active Tenants:     {}", tenants.len());
                    println!("  DataStore Engine:   Partitioned High-Assurance Storage");
                    println!("  SPIFFE / SPIRE:     ACTIVE (spiffe://meridian.runbase.io)");
                    println!("  OIDC Federation:    READY (JWT / JWKS)");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
            Ok(())
        }
        FabricAction::Tenant { action } => match action {
            FabricTenantAction::List => {
                let tenants = engine.tenant_manager.list_tenants();
                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        println!("{}", serde_json::to_string_pretty(&tenants)?);
                    }
                    _ => {
                        println!(
                            "Registered Tenants in Root Fabric (Total: {})",
                            tenants.len()
                        );
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!(
                            "{:<18} {:<28} {:<12} {:<10}",
                            "TENANT ID", "DISPLAY NAME", "TIER", "QUOTA (GB)"
                        );
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        for t in tenants {
                            println!(
                                "{:<18} {:<28} {:<12} {:<10}",
                                t.tenant_id.as_str(),
                                t.display_name,
                                t.tier,
                                t.max_storage_bytes / (1024 * 1024 * 1024)
                            );
                        }
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
                Ok(())
            }
            FabricTenantAction::Create {
                id,
                name,
                tier,
                quota_gb,
                jurisdiction,
            } => {
                let tid = TenantId::new(id).map_err(|e| AppError::Configuration(e.to_string()))?;
                let metadata = TenantMetadata {
                    tenant_id: tid.clone(),
                    display_name: name.clone(),
                    tier: tier.clone(),
                    max_storage_bytes: quota_gb * 1024 * 1024 * 1024,
                    current_storage_bytes: 0,
                    default_jurisdiction: jurisdiction.clone(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                engine.tenant_manager.register_tenant(metadata.clone());

                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        println!("{}", serde_json::to_string_pretty(&metadata)?);
                    }
                    _ => {
                        println!("Tenant Created & Registered in Root Fabric");
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!("  Tenant ID:          {}", metadata.tenant_id.as_str());
                        println!("  Display Name:       {}", metadata.display_name);
                        println!("  Service Tier:       {}", metadata.tier);
                        println!("  Storage Quota:      {} GB", quota_gb);
                        println!("  Jurisdiction:       {}", metadata.default_jurisdiction);
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
                Ok(())
            }
            FabricTenantAction::Inspect { id } => {
                let tid = TenantId::new(id).map_err(|e| AppError::Configuration(e.to_string()))?;
                let tenant = engine
                    .tenant_manager
                    .get_tenant(&tid)
                    .map_err(|e| AppError::Configuration(e.to_string()))?;

                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        println!("{}", serde_json::to_string_pretty(&tenant)?);
                    }
                    _ => {
                        println!("Tenant Inspection · {}", tenant.tenant_id.as_str());
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!("  Display Name:       {}", tenant.display_name);
                        println!("  Tier:               {}", tenant.tier);
                        println!(
                            "  Max Quota:          {} GB",
                            tenant.max_storage_bytes / (1024 * 1024 * 1024)
                        );
                        println!(
                            "  Used Quota:         {} Bytes",
                            tenant.current_storage_bytes
                        );
                        println!("  Default Baseline:   {}", tenant.default_jurisdiction);
                        println!("  Created:            {}", tenant.created_at);
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
                Ok(())
            }
        },
        FabricAction::Spiffe { id, strict } => {
            let spiffe_id: SpiffeId = id
                .parse()
                .map_err(|e| AppError::Configuration(format!("{:?}", e)))?;
            let is_valid = engine
                .spire_attestor
                .as_ref()
                .is_none_or(|att| att.verify_spiffe_id(&spiffe_id).is_ok());

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json = serde_json::json!({
                        "spiffe_id": spiffe_id.to_string(),
                        "trust_domain": spiffe_id.trust_domain().as_str(),
                        "path": spiffe_id.path(),
                        "namespace": spiffe_id.namespace(),
                        "service_account": spiffe_id.service_account(),
                        "valid_for_trust_domain": is_valid
                    });
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                _ => {
                    println!("SPIFFE / SPIRE Workload Identity Verification");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  SPIFFE ID:          {}", spiffe_id);
                    println!(
                        "  Trust Domain:       {}",
                        spiffe_id.trust_domain().as_str()
                    );
                    println!("  Path:               {}", spiffe_id.path());
                    println!(
                        "  Namespace:          {}",
                        spiffe_id.namespace().unwrap_or("—")
                    );
                    println!(
                        "  Service Account:    {}",
                        spiffe_id.service_account().unwrap_or("—")
                    );
                    println!(
                        "  Trust Domain Match: {}",
                        if is_valid {
                            "YES · VERIFIED"
                        } else {
                            "NO · MISMATCH"
                        }
                    );
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
            if *strict && !is_valid {
                return Err(AppError::Negative {
                    class: "fabric-spiffe",
                    status: format!(
                        "SPIFFE ID `{spiffe_id}` does not match configured trust domain"
                    ),
                });
            }
            Ok(())
        }
        FabricAction::Oidc {
            token,
            issuer,
            audience,
            secret,
        } => {
            let mut config = OidcProviderConfig::new(issuer, audience);
            if let Some(sec) = secret {
                config = config.with_shared_secret(sec.as_bytes());
            }
            let validator = OidcTokenValidator::new(config);
            let now = chrono::Utc::now().timestamp();

            let claims = validator
                .decode_and_validate(token, now)
                .map_err(|e| AppError::Configuration(e.to_string()))?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json = serde_json::json!({
                        "sub": claims.sub,
                        "iss": claims.iss,
                        "aud": claims.aud,
                        "exp": claims.exp,
                        "iat": claims.iat,
                        "email": claims.email,
                        "tenant_id": claims.tenant_id.as_deref().unwrap_or("default"),
                        "roles": claims.roles,
                        "groups": claims.groups,
                        "status": "VALID"
                    });
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                _ => {
                    println!("OpenID Connect (OIDC) Claims Decoded & Validated");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Subject (sub):      {}", claims.sub);
                    println!("  Issuer (iss):       {}", claims.iss);
                    println!(
                        "  Email:              {}",
                        claims.email.as_deref().unwrap_or("—")
                    );
                    println!(
                        "  Tenant ID:          {}",
                        claims.tenant_id.as_deref().unwrap_or("default")
                    );
                    println!("  Roles:              {:?}", claims.roles);
                    println!("  Groups:             {:?}", claims.groups);
                    println!("  Expires At:         {}", claims.exp);
                    println!("  Status:             VALID & ACTIVE");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
            Ok(())
        }
        FabricAction::Datastore { action } => {
            let datastore = crate::fabric::FabricDataStore::new_persistent(
                std::path::PathBuf::from(".mizan/datastore"),
            )
            .unwrap_or_else(|_| engine.datastore.clone());

            match action {
                FabricDatastoreAction::Stats => {
                    let tenants = engine.tenant_manager.list_tenants();
                    let tenant_ids: Vec<&str> =
                        tenants.iter().map(|t| t.tenant_id.as_str()).collect();
                    let mut total_docs = 0usize;
                    let mut total_audit_entries = 0usize;
                    for t in &tenants {
                        let ctx = crate::fabric::TenantContext::new(
                            t.tenant_id.clone(),
                            crate::fabric::UserId::new("cli-stats"),
                        );
                        if let Ok(docs) = datastore.list_documents(&ctx, None) {
                            total_docs += docs.len();
                        }
                        if let Ok(audits) = datastore.get_audit_logs(&ctx) {
                            total_audit_entries += audits.len();
                        }
                    }

                    match format {
                        OutputFormat::Json | OutputFormat::Jsonl => {
                            let json = serde_json::json!({
                                "active_partitions": tenants.len(),
                                "partitions": tenant_ids,
                                "stored_documents_count": total_docs,
                                "audit_log_entries_count": total_audit_entries,
                                "storage_dir": datastore.storage_dir().map(|p| p.display().to_string()),
                                "isolation_engine": "Cryptographic TenantContext Boundaries",
                                "storage_encryption": "AES-256-GCM / SHA-256 CAS backing",
                                "audit_mode": "Append-Only Cryptographic Receipts"
                            });
                            println!("{}", serde_json::to_string_pretty(&json)?);
                        }
                        _ => {
                            println!(
                                "Mizan High-Assurance Isolated DataStore Partition Statistics"
                            );
                            println!(
                                "────────────────────────────────────────────────────────────────────────"
                            );
                            println!(
                                "  Active Partitions:  {} ({})",
                                tenants.len(),
                                tenant_ids.join(", ")
                            );
                            println!("  Stored Documents:   {}", total_docs);
                            println!("  Audit Log Entries:  {}", total_audit_entries);
                            println!(
                                "  Isolation Engine:   Cryptographic TenantContext Boundaries"
                            );
                            println!("  Storage Encryption: AES-256-GCM / SHA-256 CAS backing");
                            println!("  Audit Mode:         Append-Only Cryptographic Receipts");
                            println!(
                                "────────────────────────────────────────────────────────────────────────"
                            );
                        }
                    }
                    Ok(())
                }
                FabricDatastoreAction::List { tenant, namespace } => {
                    let tid = TenantId::new(tenant)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    let ctx = crate::fabric::TenantContext::new(
                        tid,
                        crate::fabric::UserId::new("cli-inspector"),
                    );
                    let docs = datastore
                        .list_documents(&ctx, namespace.as_deref())
                        .map_err(|e| AppError::Configuration(e.to_string()))?;

                    match format {
                        OutputFormat::Json | OutputFormat::Jsonl => {
                            println!("{}", serde_json::to_string_pretty(&docs)?);
                        }
                        _ => {
                            println!(
                                "Stored Documents in DataStore for Tenant: {} (Namespace: {}, Total: {})",
                                tenant,
                                namespace.as_deref().unwrap_or("ALL"),
                                docs.len()
                            );
                            println!(
                                "────────────────────────────────────────────────────────────────────────"
                            );
                            println!(
                                "{:<24} {:<16} {:<10} {:<12}",
                                "DOCUMENT ID", "RESOURCE TYPE", "VERSION", "DIGEST"
                            );
                            println!(
                                "────────────────────────────────────────────────────────────────────────"
                            );
                            if docs.is_empty() {
                                println!("(no documents stored in partition)");
                            } else {
                                for doc in &docs {
                                    let short_digest = if doc.sha256_digest.chars().count() > 8 {
                                        let prefix: String =
                                            doc.sha256_digest.chars().take(8).collect();
                                        format!("sha256:{prefix}…")
                                    } else {
                                        format!("sha256:{}", doc.sha256_digest)
                                    };
                                    println!(
                                        "{:<24} {:<16} {:<10} {:<12}",
                                        doc.document_id,
                                        doc.resource_type,
                                        doc.version,
                                        short_digest
                                    );
                                }
                            }
                            println!(
                                "────────────────────────────────────────────────────────────────────────"
                            );
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}
