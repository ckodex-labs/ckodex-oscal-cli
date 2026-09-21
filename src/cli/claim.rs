#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) async fn run_claim(config: &AppConfig, action: ClaimAction) -> Result<()> {
    if let ClaimAction::List { page_size, .. } = &action {
        validate_positive("page-size", *page_size)?;
    }
    let mut client = ReadOnlyClient::connect(config).await?;
    match action {
        ClaimAction::List {
            subject_digest,
            bom_kind,
            relation,
            trust_state,
            page_size,
            page_token,
        } => {
            let request = ListClaimsRequest {
                subject_digest: subject_digest.unwrap_or_default(),
                bom_kind: bom_kind.unwrap_or_default(),
                relation: relation.unwrap_or_default(),
                trust_state: trust_state.unwrap_or_default(),
                valid_after: None,
                page_size,
                page_token,
            };
            let response: ListClaimsResponse = client.list_claims(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.ListClaims",
                &request,
                &response,
            )?;
            if config.output == OutputFormat::Table {
                let rows = response
                    .claims
                    .iter()
                    .map(|claim| {
                        vec![
                            claim.id.clone(),
                            claim.r#type.clone(),
                            claim
                                .predicate
                                .as_ref()
                                .map_or_else(String::new, |value| value.relation.clone()),
                            claim
                                .subject
                                .as_ref()
                                .map_or_else(String::new, |value| value.digest.clone()),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(&["CLAIM", "TYPE", "RELATION", "SUBJECT DIGEST"], &rows);
                output::page_token(&response.next_page_token);
            } else {
                output::emit_message(
                    config.output,
                    "oscal.services.v1.ListClaimsResponse",
                    &response,
                )?;
            }
        }
        ClaimAction::Get { claim_id } => {
            let request = GetClaimRequest { claim_id };
            let response: GetClaimResponse = client.get_claim(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.GetClaim",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.GetClaimResponse",
                &response,
            )?;
        }
        ClaimAction::Events { claim_id } => {
            let request = ListVerificationEventsRequest { claim_id };
            let response: ListVerificationEventsResponse =
                client.list_verification_events(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.ListVerificationEvents",
                &request,
                &response,
            )?;
            if config.output == OutputFormat::Table {
                let rows = response
                    .events
                    .iter()
                    .map(|event| {
                        vec![
                            event.sequence.to_string(),
                            event.event_id.clone(),
                            event.trust_state.clone(),
                            event.event_hash.clone(),
                            event.diagnostics.join("; "),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(
                    &["SEQUENCE", "EVENT", "TRUST", "EVENT HASH", "DIAGNOSTICS"],
                    &rows,
                );
                println!("chain_valid  {}", response.chain_valid);
            } else {
                output::emit_message(
                    config.output,
                    "oscal.services.v1.ListVerificationEventsResponse",
                    &response,
                )?;
            }
        }
        ClaimAction::Receipt { claim_id } => {
            let request = ExportClaimReceiptRequest { claim_id };
            let response: ExportClaimReceiptResponse =
                client.export_claim_receipt(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.ExportClaimReceipt",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.ExportClaimReceiptResponse",
                &response,
            )?;
        }
        ClaimAction::Create { .. } | ClaimAction::Verify { .. } | ClaimAction::Sync { .. } => {
            return Err(AppError::Configuration(
                "claim write actions require network configuration".to_owned(),
            ));
        }
    }
    Ok(())
}
