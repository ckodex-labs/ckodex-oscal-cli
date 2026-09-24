#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, ClaimAction, EvidenceAction},
    error::{AppError, Result},
    output,
    proto::oscal::services::v1::{
        Claim, CreateClaimRequest, CreateClaimResponse, Evidence, SyncClaimsRequest,
        SyncClaimsResponse, UploadEvidenceRequest, UploadEvidenceResponse, VerifyClaimRequest,
        VerifyClaimResponse,
    },
    transport::CrudClient,
};
use serde_json::Value;
use std::path::Path;

pub async fn run_claim(config: &AppConfig, action: ClaimAction) -> Result<()> {
    let mut client = CrudClient::connect(config).await?;
    match action {
        ClaimAction::Create { file } => {
            let value = read_document_value(&file)?;
            let claim_value = extract_message_value(value, "claim");
            let claim: Claim =
                super::crud::decode_from_json("oscal.services.v1.Claim", &claim_value)?;
            let request = CreateClaimRequest { claim: Some(claim) };
            let result: crate::valence::CrudResult<CreateClaimResponse> =
                client.create_claim(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.CreateClaim",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.CreateClaimResponse",
                &result,
                config.valence,
            )
        }
        ClaimAction::Verify { claim_id, checks } => {
            validate_non_empty("claim_id", &claim_id)?;
            let request = VerifyClaimRequest { claim_id, checks };
            let result: crate::valence::CrudResult<VerifyClaimResponse> =
                client.verify_claim(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.VerifyClaim",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.VerifyClaimResponse",
                &result,
                config.valence,
            )
        }
        ClaimAction::Sync {
            peer_endpoint,
            filter_subject,
            filter_bom_kind,
        } => {
            let request = SyncClaimsRequest {
                peer_endpoint,
                filter_subject,
                filter_bom_kind,
                since: None,
            };
            let result: crate::valence::CrudResult<SyncClaimsResponse> =
                client.sync_claims(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.SyncClaims",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.SyncClaimsResponse",
                &result,
                config.valence,
            )
        }
        _ => Err(AppError::Configuration(
            "claim read actions require a read subcommand".to_owned(),
        )),
    }
}

pub async fn run_evidence(config: &AppConfig, action: EvidenceAction) -> Result<()> {
    match action {
        EvidenceAction::Upload { file, blob } => {
            let value = read_document_value(&file)?;
            let evidence_value = extract_message_value(value, "evidence");
            let evidence: Evidence =
                super::crud::decode_from_json("oscal.services.v1.Evidence", &evidence_value)?;
            let blob_bytes = match blob {
                Some(path) => std::fs::read(&path).map_err(|e| crate::error::io_error(&path, e))?,
                None => Vec::new(),
            };
            let mut client = CrudClient::connect(config).await?;
            let request = UploadEvidenceRequest {
                evidence: Some(evidence),
                blob: blob_bytes,
            };
            let result: crate::valence::CrudResult<UploadEvidenceResponse> =
                client.upload_evidence(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.UploadEvidence",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.UploadEvidenceResponse",
                &result,
                config.valence,
            )
        }
        _ => Err(AppError::Configuration(
            "evidence read actions require a read subcommand".to_owned(),
        )),
    }
}

fn read_document_value<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path_ref = path.as_ref();
    let content =
        std::fs::read_to_string(path_ref).map_err(|e| crate::error::io_error(path_ref, e))?;
    let value: Value = if is_yaml(path_ref) {
        serde_yaml::from_str(&content)
            .map_err(|e| AppError::Configuration(format!("failed to parse YAML: {e}")))?
    } else {
        serde_json::from_str(&content).map_err(AppError::Serialization)?
    };
    Ok(value)
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "yaml" || e == "yml")
        .unwrap_or(false)
}

fn extract_message_value(value: Value, field: &str) -> Value {
    if let Value::Object(map) = &value
        && let Some(inner) = map.get(field)
    {
        return inner.clone();
    }
    value
}
