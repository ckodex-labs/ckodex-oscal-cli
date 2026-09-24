use std::fs;

use tonic::{
    Request,
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
};

use crate::{
    config::AppConfig,
    error::{AppError, Result, redact_endpoint},
    health::{HealthCheckRequest, HealthCheckResponse},
    proto::oscal::{common::v1::Uuid, services::v1 as svc},
};

pub mod crud_client;
pub use crud_client::CrudClient;

pub(crate) const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

pub async fn connect_channel(config: &AppConfig) -> Result<Channel> {
    let mut endpoint = Endpoint::from_shared(config.endpoint.clone())
        .map_err(|error| AppError::Configuration(format!("invalid endpoint: {error}")))?
        .connect_timeout(config.timeout)
        .timeout(config.timeout);
    let https = config.endpoint.starts_with("https://");
    let tls_options = config.tls_domain.is_some()
        || config.ca_cert.is_some()
        || config.client_cert.is_some()
        || config.client_key.is_some();
    if tls_options && !https {
        return Err(AppError::Configuration(
            "TLS options require an https:// endpoint".to_owned(),
        ));
    }
    if https {
        let domain = config
            .tls_domain
            .clone()
            .or_else(|| infer_tls_domain(&config.endpoint))
            .ok_or_else(|| {
                AppError::Configuration(
                    "TLS endpoint requires a resolvable host or --tls-domain".to_owned(),
                )
            })?;
        let mut tls = ClientTlsConfig::new()
            .domain_name(domain)
            .with_webpki_roots();
        if let Some(path) = &config.ca_cert {
            let bytes = fs::read(path).map_err(|error| crate::error::io_error(path, error))?;
            tls = tls.ca_certificate(Certificate::from_pem(bytes));
        }
        if let (Some(cert_path), Some(key_path)) = (&config.client_cert, &config.client_key) {
            let cert =
                fs::read(cert_path).map_err(|error| crate::error::io_error(cert_path, error))?;
            let key =
                fs::read(key_path).map_err(|error| crate::error::io_error(key_path, error))?;
            tls = tls.identity(Identity::from_pem(cert, key));
        }
        endpoint = endpoint
            .tls_config(tls)
            .map_err(|error| AppError::Configuration(format!("invalid TLS config: {error}")))?;
    }
    endpoint
        .connect()
        .await
        .map_err(|error| AppError::Connection {
            endpoint: redact_endpoint(&config.endpoint),
            message: error.to_string(),
            hint: "check the endpoint, server availability, timeout, and TLS settings",
        })
}

pub(crate) fn attach_token<T>(request: &mut Request<T>, token: Option<&str>) -> Result<()> {
    if let Some(token) = token {
        let value = MetadataValue::try_from(format!("Bearer {token}"))
            .map_err(|error| AppError::Configuration(format!("invalid token: {error}")))?;
        request.metadata_mut().insert("authorization", value);
    }
    Ok(())
}

pub fn uuid(value: String) -> Uuid {
    Uuid { value }
}

pub(crate) fn infer_tls_domain(endpoint: &str) -> Option<String> {
    let authority = endpoint
        .strip_prefix("https://")?
        .split(['/', '?', '#'])
        .next()?;
    let host = authority.rsplit('@').next()?;
    if let Some(stripped) = host.strip_prefix('[') {
        return stripped.split(']').next().map(str::to_owned);
    }
    host.rsplit_once(':')
        .map_or_else(|| Some(host.to_owned()), |(name, _)| Some(name.to_owned()))
}

pub struct ReadOnlyClient {
    inner: CrudClient,
}

impl ReadOnlyClient {
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        Ok(Self {
            inner: CrudClient::connect_read_only(config).await?,
        })
    }

    pub async fn health_check(&mut self, body: HealthCheckRequest) -> Result<HealthCheckResponse> {
        self.inner.health_check(body).await.map(|cr| cr.data)
    }

    pub async fn search(&mut self, body: svc::SearchRequest) -> Result<svc::SearchResponse> {
        self.inner.search(body).await.map(|cr| cr.data)
    }

    pub async fn semantic_search(
        &mut self,
        body: svc::SemanticSearchRequest,
    ) -> Result<svc::SemanticSearchResponse> {
        self.inner.semantic_search(body).await.map(|cr| cr.data)
    }

    pub async fn list_entities(
        &mut self,
        body: svc::ListEntitiesRequest,
    ) -> Result<svc::ListEntitiesResponse> {
        self.inner.list_entities(body).await.map(|cr| cr.data)
    }

    pub async fn get_entity(
        &mut self,
        body: svc::GetEntityRequest,
    ) -> Result<svc::GetEntityResponse> {
        self.inner.get_entity(body).await.map(|cr| cr.data)
    }

    pub async fn list_catalogs(
        &mut self,
        body: svc::ListCatalogsRequest,
    ) -> Result<svc::ListCatalogsResponse> {
        self.inner.list_catalogs(body).await.map(|cr| cr.data)
    }

    pub async fn get_catalog(
        &mut self,
        body: svc::GetCatalogRequest,
    ) -> Result<svc::GetCatalogResponse> {
        self.inner.get_catalog(body).await.map(|cr| cr.data)
    }

    pub async fn list_profiles(
        &mut self,
        body: svc::ListProfilesRequest,
    ) -> Result<svc::ListProfilesResponse> {
        self.inner.list_profiles(body).await.map(|cr| cr.data)
    }

    pub async fn get_profile(
        &mut self,
        body: svc::GetProfileRequest,
    ) -> Result<svc::GetProfileResponse> {
        self.inner.get_profile(body).await.map(|cr| cr.data)
    }

    pub async fn list_component_definitions(
        &mut self,
        body: svc::ListComponentDefinitionsRequest,
    ) -> Result<svc::ListComponentDefinitionsResponse> {
        self.inner
            .list_component_definitions(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn get_component_definition(
        &mut self,
        body: svc::GetComponentDefinitionRequest,
    ) -> Result<svc::GetComponentDefinitionResponse> {
        self.inner
            .get_component_definition(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn list_ssps(&mut self, body: svc::ListSspsRequest) -> Result<svc::ListSspsResponse> {
        self.inner.list_ssps(body).await.map(|cr| cr.data)
    }

    pub async fn get_ssp(&mut self, body: svc::GetSspRequest) -> Result<svc::GetSspResponse> {
        self.inner.get_ssp(body).await.map(|cr| cr.data)
    }

    pub async fn list_assessment_plans(
        &mut self,
        body: svc::ListAssessmentPlansRequest,
    ) -> Result<svc::ListAssessmentPlansResponse> {
        self.inner
            .list_assessment_plans(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn get_assessment_plan(
        &mut self,
        body: svc::GetAssessmentPlanRequest,
    ) -> Result<svc::GetAssessmentPlanResponse> {
        self.inner.get_assessment_plan(body).await.map(|cr| cr.data)
    }

    pub async fn list_assessment_results(
        &mut self,
        body: svc::ListAssessmentResultsRequest,
    ) -> Result<svc::ListAssessmentResultsResponse> {
        self.inner
            .list_assessment_results(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn get_assessment_results(
        &mut self,
        body: svc::GetAssessmentResultsRequest,
    ) -> Result<svc::GetAssessmentResultsResponse> {
        self.inner
            .get_assessment_results(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn list_poams(
        &mut self,
        body: svc::ListPoamsRequest,
    ) -> Result<svc::ListPoamsResponse> {
        self.inner.list_poams(body).await.map(|cr| cr.data)
    }

    pub async fn get_poam(&mut self, body: svc::GetPoamRequest) -> Result<svc::GetPoamResponse> {
        self.inner.get_poam(body).await.map(|cr| cr.data)
    }

    pub async fn list_mappings(
        &mut self,
        body: svc::ListMappingsRequest,
    ) -> Result<svc::ListMappingsResponse> {
        self.inner.list_mappings(body).await.map(|cr| cr.data)
    }

    pub async fn get_mapping(
        &mut self,
        body: svc::GetMappingRequest,
    ) -> Result<svc::GetMappingResponse> {
        self.inner.get_mapping(body).await.map(|cr| cr.data)
    }

    pub async fn list_frameworks(
        &mut self,
        body: svc::ListFrameworksRequest,
    ) -> Result<svc::ListFrameworksResponse> {
        self.inner.list_frameworks(body).await.map(|cr| cr.data)
    }

    pub async fn get_framework(
        &mut self,
        body: svc::GetFrameworkRequest,
    ) -> Result<svc::GetFrameworkResponse> {
        self.inner.get_framework(body).await.map(|cr| cr.data)
    }

    pub async fn list_snapshots(
        &mut self,
        body: svc::ListSnapshotsRequest,
    ) -> Result<svc::ListSnapshotsResponse> {
        self.inner.list_snapshots(body).await.map(|cr| cr.data)
    }

    pub async fn get_snapshot(
        &mut self,
        body: svc::GetSnapshotRequest,
    ) -> Result<svc::GetSnapshotResponse> {
        self.inner.get_snapshot(body).await.map(|cr| cr.data)
    }

    pub async fn list_releases(
        &mut self,
        body: svc::ListReleasesRequest,
    ) -> Result<svc::ListReleasesResponse> {
        self.inner.list_releases(body).await.map(|cr| cr.data)
    }

    pub async fn get_edge(&mut self, body: svc::GetEdgeRequest) -> Result<svc::GetEdgeResponse> {
        self.inner.get_edge(body).await.map(|cr| cr.data)
    }

    pub async fn list_edges(
        &mut self,
        body: svc::ListEdgesRequest,
    ) -> Result<svc::ListEdgesResponse> {
        self.inner.list_edges(body).await.map(|cr| cr.data)
    }

    pub async fn list_projection_events(
        &mut self,
        body: svc::ListProjectionEventsRequest,
    ) -> Result<svc::ListProjectionEventsResponse> {
        self.inner
            .list_projection_events(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn get_node(&mut self, body: svc::GetNodeRequest) -> Result<svc::GetNodeResponse> {
        self.inner.get_node(body).await.map(|cr| cr.data)
    }

    pub async fn list_nodes(
        &mut self,
        body: svc::ListNodesRequest,
    ) -> Result<svc::ListNodesResponse> {
        self.inner.list_nodes(body).await.map(|cr| cr.data)
    }

    pub async fn list_claims(
        &mut self,
        body: svc::ListClaimsRequest,
    ) -> Result<svc::ListClaimsResponse> {
        self.inner.list_claims(body).await.map(|cr| cr.data)
    }

    pub async fn get_claim(&mut self, body: svc::GetClaimRequest) -> Result<svc::GetClaimResponse> {
        self.inner.get_claim(body).await.map(|cr| cr.data)
    }

    pub async fn list_verification_events(
        &mut self,
        body: svc::ListVerificationEventsRequest,
    ) -> Result<svc::ListVerificationEventsResponse> {
        self.inner
            .list_verification_events(body)
            .await
            .map(|cr| cr.data)
    }

    pub async fn export_claim_receipt(
        &mut self,
        body: svc::ExportClaimReceiptRequest,
    ) -> Result<svc::ExportClaimReceiptResponse> {
        self.inner
            .export_claim_receipt(body)
            .await
            .map(|cr| cr.data)
    }

    #[allow(dead_code)]
    pub async fn list_fetch_events(
        &mut self,
        body: svc::ListFetchEventsRequest,
    ) -> Result<svc::ListFetchEventsResponse> {
        self.inner.list_fetch_events(body).await.map(|cr| cr.data)
    }

    pub async fn get_evidence(
        &mut self,
        body: svc::GetEvidenceRequest,
    ) -> Result<svc::GetEvidenceResponse> {
        self.inner.get_evidence(body).await.map(|cr| cr.data)
    }

    pub async fn verify_evidence(
        &mut self,
        body: svc::VerifyEvidenceRequest,
    ) -> Result<svc::VerifyEvidenceResponse> {
        self.inner.verify_evidence(body).await.map(|cr| cr.data)
    }

    pub async fn traverse(&mut self, body: svc::TraverseRequest) -> Result<svc::TraverseResponse> {
        self.inner.traverse(body).await.map(|cr| cr.data)
    }

    pub async fn shortest_path(
        &mut self,
        body: svc::ShortestPathRequest,
    ) -> Result<svc::ShortestPathResponse> {
        self.inner.shortest_path(body).await.map(|cr| cr.data)
    }

    pub async fn impact_radius(
        &mut self,
        body: svc::ImpactRadiusRequest,
    ) -> Result<svc::ImpactRadiusResponse> {
        self.inner.impact_radius(body).await.map(|cr| cr.data)
    }

    pub async fn explain_claim(
        &mut self,
        body: svc::ExplainClaimRequest,
    ) -> Result<svc::ExplainClaimResponse> {
        self.inner.explain_claim(body).await.map(|cr| cr.data)
    }

    pub async fn compute_trust_state(
        &mut self,
        body: svc::ComputeTrustStateRequest,
    ) -> Result<svc::ComputeTrustStateResponse> {
        self.inner.compute_trust_state(body).await.map(|cr| cr.data)
    }

    pub async fn verify_closure(
        &mut self,
        body: svc::VerifyClosureRequest,
    ) -> Result<svc::VerifyClosureResponse> {
        self.inner.verify_closure(body).await.map(|cr| cr.data)
    }
}

#[cfg(test)]
mod tests {
    use super::{attach_token, infer_tls_domain};
    use tonic::Request;

    #[test]
    fn tls_domain_ignores_userinfo_path_and_query() {
        assert_eq!(
            infer_tls_domain("https://operator:secret@example.test:443/rpc?token=hidden"),
            Some("example.test".to_owned())
        );
    }

    #[test]
    fn bearer_token_is_attached_as_authorization_metadata() {
        let mut request = Request::new(());
        attach_token(&mut request, Some("beta-token")).expect("token should be valid metadata");

        assert_eq!(
            request
                .metadata()
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer beta-token")
        );
    }
}
