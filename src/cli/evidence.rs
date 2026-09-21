#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_evidence_local(
    args: &crate::config::EvidenceArgs,
    format: OutputFormat,
) -> Result<()> {
    match &args.action {
        EvidenceAction::Bundle {
            file,
            level,
            evaluator,
            sign_as,
            output,
        } => {
            let doc = OscalDocument::from_file(file)?;
            let ev_level = match level.to_lowercase().as_str() {
                "e0" => EvidenceLevel::E0Unverified,
                "e1" => EvidenceLevel::E1SchemaValid,
                "e2" => EvidenceLevel::E2LinterPassed,
                "e4" => EvidenceLevel::E4AuditPassed,
                "e5" => EvidenceLevel::E5ContinuousMonitoring,
                _ => EvidenceLevel::E3FedrampPassed,
            };

            let mut bundle = EvidenceBundle::create_from_oscal_document(&doc, ev_level, evaluator)?;

            if let Some(signer) = sign_as {
                bundle.sign(signer);
            }

            if let Some(out_p) = output {
                bundle.save_to_file(out_p)?;
            }

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&bundle)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Cryptographic Evidence Bundle (E0-E5 Proof)");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Bundle ID:           {}", bundle.bundle_id);
                    println!("  Evidence Level:      {}", bundle.evidence_level);
                    println!("  Document UUID:       {}", bundle.document_uuid);
                    println!("  Document Digest:     {}", bundle.document_digest);
                    println!("  Merkle Root:         {}", bundle.merkle_root);
                    println!("  Observations:        {}", bundle.observations.len());
                    if let Some(sig) = &bundle.signature {
                        println!(
                            "  Signed By:           {} ({})",
                            sig.signed_by, sig.algorithm
                        );
                    }
                    if let Some(out) = output {
                        println!("  Saved Bundle:        {}", out.display());
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        EvidenceAction::VerifyBundle { bundle_file } => {
            let bundle = EvidenceBundle::load_from_file(bundle_file)?;
            let report = bundle.verify();

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Evidence Bundle Cryptographic Verification");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Bundle ID:           {}", report.bundle_id);
                    println!("  Evidence Level:      {}", report.evidence_level);
                    println!("  Document UUID:       {}", report.document_uuid);
                    println!("  Observations:        {}", report.observations_count);
                    println!("  Merkle Root:         {}", report.merkle_root);
                    println!(
                        "  Integrity Valid:     {}",
                        if report.is_valid { "YES" } else { "NO" }
                    );
                    println!(
                        "  Signature Verified:  {}",
                        if report.signature_verified {
                            "YES"
                        } else {
                            "NO / UNSIGNED"
                        }
                    );
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        _ => {
            return Err(AppError::Configuration(
                "Remote evidence operations require network configuration".to_string(),
            ));
        }
    }
    Ok(())
}

pub(super) async fn run_evidence(config: &AppConfig, action: EvidenceAction) -> Result<()> {
    let mut client = ReadOnlyClient::connect(config).await?;
    match action {
        EvidenceAction::FetchEvents {
            page_size,
            page_token,
        } => {
            validate_positive("page-size", page_size)?;
            let request = ListFetchEventsRequest {
                page_size,
                page_token,
            };
            let response = client.list_fetch_events(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.ListFetchEvents",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.ListFetchEventsResponse",
                &response,
            )?;
        }
        EvidenceAction::Get { evidence_id } => {
            let request = GetEvidenceRequest { evidence_id };
            let response: GetEvidenceResponse = client.get_evidence(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.GetEvidence",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.GetEvidenceResponse",
                &response,
            )?;
        }
        EvidenceAction::Verify {
            evidence_id,
            fetch_and_hash,
        } => {
            let request = VerifyEvidenceRequest {
                evidence_id,
                fetch_and_hash,
            };
            let response: VerifyEvidenceResponse = client.verify_evidence(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.VerifyEvidence",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.VerifyEvidenceResponse",
                &response,
            )?;
        }
        EvidenceAction::Bundle { .. } | EvidenceAction::VerifyBundle { .. } => {
            return Err(AppError::Configuration(
                "Evidence bundling and verification are local operations; run without server flags.".to_string(),
            ));
        }
        EvidenceAction::Upload { .. } => {
            return Err(AppError::Configuration(
                "evidence upload requires network configuration".to_owned(),
            ));
        }
    }
    Ok(())
}
