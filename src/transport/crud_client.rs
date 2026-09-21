use tonic::{client::Grpc, codegen::http::uri::PathAndQuery, transport::Channel, Request};
use tonic_prost::ProstCodec;

use crate::{
    config::AppConfig,
    error::{AppError, Result},
    health::{HealthCheckRequest, HealthCheckResponse},
    policy::{Gate, GovernanceModel, MethodClass, OscalModel, RpcMethod},
    proto::oscal::services::v1 as svc,
    transport::{attach_token, connect_channel, MAX_MESSAGE_BYTES},
    valence::{CrudResult, Valence},
};

pub struct CrudClient {
    oscal: svc::oscal_service_client::OscalServiceClient<Channel>,
    governance: svc::governance_service_client::GovernanceServiceClient<Channel>,
    transparency:
        svc::transparency_exchange_service_client::TransparencyExchangeServiceClient<Channel>,
    graph: svc::transparency_graph_service_client::TransparencyGraphServiceClient<Channel>,
    health: Grpc<Channel>,
    gate: Gate,
    token: Option<String>,
}

impl CrudClient {
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        Self::connect_with_gate(
            config,
            Gate {
                read_only: config.read_only,
            },
        )
        .await
    }

    pub(crate) async fn connect_read_only(config: &AppConfig) -> Result<Self> {
        Self::connect_with_gate(config, Gate { read_only: true }).await
    }

    async fn connect_with_gate(config: &AppConfig, gate: Gate) -> Result<Self> {
        let channel = connect_channel(config).await?;
        Ok(Self {
            oscal: svc::oscal_service_client::OscalServiceClient::new(channel.clone())
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            governance: svc::governance_service_client::GovernanceServiceClient::new(
                channel.clone(),
            )
            .max_decoding_message_size(MAX_MESSAGE_BYTES)
            .max_encoding_message_size(MAX_MESSAGE_BYTES),
            transparency:
                svc::transparency_exchange_service_client::TransparencyExchangeServiceClient::new(
                    channel.clone(),
                )
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            graph: svc::transparency_graph_service_client::TransparencyGraphServiceClient::new(
                channel.clone(),
            )
            .max_decoding_message_size(MAX_MESSAGE_BYTES)
            .max_encoding_message_size(MAX_MESSAGE_BYTES),
            health: Grpc::new(channel)
                .max_decoding_message_size(MAX_MESSAGE_BYTES)
                .max_encoding_message_size(MAX_MESSAGE_BYTES),
            gate,
            token: config.token.clone(),
        })
    }

    fn request<T>(&self, method: RpcMethod, body: T) -> Result<Request<T>> {
        self.gate.permit(&method)?;
        let mut request = Request::new(body);
        attach_token(&mut request, self.token.as_deref())?;
        Ok(request)
    }
}

fn negative(method: &RpcMethod, status: tonic::Status) -> AppError {
    AppError::Negative {
        class: method.class().as_str(),
        status: status.to_string(),
    }
}

impl CrudClient {
    pub async fn health_check(
        &mut self,
        body: HealthCheckRequest,
    ) -> Result<CrudResult<HealthCheckResponse>> {
        let request = self.request(RpcMethod::HealthCheck, body)?;
        self.health.ready().await.map_err(|error| {
            AppError::Rpc(Box::new(tonic::Status::unknown(format!(
                "health transport not ready: {error}"
            ))))
        })?;
        Ok(CrudResult::new(
            Valence::Observed,
            self.health
                .unary(
                    request,
                    PathAndQuery::from_static("/grpc.health.v1.Health/Check"),
                    ProstCodec::default(),
                )
                .await?
                .into_inner(),
        ))
    }

    pub async fn search(
        &mut self,
        body: svc::SearchRequest,
    ) -> Result<CrudResult<svc::SearchResponse>> {
        let method = RpcMethod::OscalSearch;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .search(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn semantic_search(
        &mut self,
        body: svc::SemanticSearchRequest,
    ) -> Result<CrudResult<svc::SemanticSearchResponse>> {
        let method = RpcMethod::GovernanceSemanticSearch;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .semantic_search(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_entities(
        &mut self,
        body: svc::ListEntitiesRequest,
    ) -> Result<CrudResult<svc::ListEntitiesResponse>> {
        let method = RpcMethod::GovernanceList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .list_entities(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_entity(
        &mut self,
        body: svc::GetEntityRequest,
    ) -> Result<CrudResult<svc::GetEntityResponse>> {
        let method = RpcMethod::GovernanceGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .get_entity(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_catalogs(
        &mut self,
        body: svc::ListCatalogsRequest,
    ) -> Result<CrudResult<svc::ListCatalogsResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_catalogs(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_catalog(
        &mut self,
        body: svc::GetCatalogRequest,
    ) -> Result<CrudResult<svc::GetCatalogResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_catalog(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_profiles(
        &mut self,
        body: svc::ListProfilesRequest,
    ) -> Result<CrudResult<svc::ListProfilesResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_profiles(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_profile(
        &mut self,
        body: svc::GetProfileRequest,
    ) -> Result<CrudResult<svc::GetProfileResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_profile(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_component_definitions(
        &mut self,
        body: svc::ListComponentDefinitionsRequest,
    ) -> Result<CrudResult<svc::ListComponentDefinitionsResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_component_definitions(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_component_definition(
        &mut self,
        body: svc::GetComponentDefinitionRequest,
    ) -> Result<CrudResult<svc::GetComponentDefinitionResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_component_definition(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_ssps(
        &mut self,
        body: svc::ListSspsRequest,
    ) -> Result<CrudResult<svc::ListSspsResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_ssps(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_ssp(
        &mut self,
        body: svc::GetSspRequest,
    ) -> Result<CrudResult<svc::GetSspResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_ssp(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_assessment_plans(
        &mut self,
        body: svc::ListAssessmentPlansRequest,
    ) -> Result<CrudResult<svc::ListAssessmentPlansResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_assessment_plans(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_assessment_plan(
        &mut self,
        body: svc::GetAssessmentPlanRequest,
    ) -> Result<CrudResult<svc::GetAssessmentPlanResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_assessment_plan(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_assessment_results(
        &mut self,
        body: svc::ListAssessmentResultsRequest,
    ) -> Result<CrudResult<svc::ListAssessmentResultsResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_assessment_results(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_assessment_results(
        &mut self,
        body: svc::GetAssessmentResultsRequest,
    ) -> Result<CrudResult<svc::GetAssessmentResultsResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_assessment_results(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_poams(
        &mut self,
        body: svc::ListPoamsRequest,
    ) -> Result<CrudResult<svc::ListPoamsResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_poams(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_poam(
        &mut self,
        body: svc::GetPoamRequest,
    ) -> Result<CrudResult<svc::GetPoamResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_poam(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_mappings(
        &mut self,
        body: svc::ListMappingsRequest,
    ) -> Result<CrudResult<svc::ListMappingsResponse>> {
        let method = RpcMethod::OscalList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .list_mappings(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_mapping(
        &mut self,
        body: svc::GetMappingRequest,
    ) -> Result<CrudResult<svc::GetMappingResponse>> {
        let method = RpcMethod::OscalGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.oscal
                .get_mapping(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_frameworks(
        &mut self,
        body: svc::ListFrameworksRequest,
    ) -> Result<CrudResult<svc::ListFrameworksResponse>> {
        let method = RpcMethod::GovernanceList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .list_frameworks(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_framework(
        &mut self,
        body: svc::GetFrameworkRequest,
    ) -> Result<CrudResult<svc::GetFrameworkResponse>> {
        let method = RpcMethod::GovernanceGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .get_framework(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_snapshots(
        &mut self,
        body: svc::ListSnapshotsRequest,
    ) -> Result<CrudResult<svc::ListSnapshotsResponse>> {
        let method = RpcMethod::GovernanceList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .list_snapshots(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_snapshot(
        &mut self,
        body: svc::GetSnapshotRequest,
    ) -> Result<CrudResult<svc::GetSnapshotResponse>> {
        let method = RpcMethod::GovernanceGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .get_snapshot(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_releases(
        &mut self,
        body: svc::ListReleasesRequest,
    ) -> Result<CrudResult<svc::ListReleasesResponse>> {
        let method = RpcMethod::GovernanceList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.governance
                .list_releases(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_edge(
        &mut self,
        body: svc::GetEdgeRequest,
    ) -> Result<CrudResult<svc::GetEdgeResponse>> {
        let method = RpcMethod::GraphGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.graph
                .get_edge(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_edges(
        &mut self,
        body: svc::ListEdgesRequest,
    ) -> Result<CrudResult<svc::ListEdgesResponse>> {
        let method = RpcMethod::GraphList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.graph
                .list_edges(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_projection_events(
        &mut self,
        body: svc::ListProjectionEventsRequest,
    ) -> Result<CrudResult<svc::ListProjectionEventsResponse>> {
        let method = RpcMethod::GraphProjectionEvents;
        Ok(CrudResult::new(
            Valence::Observed,
            self.graph
                .list_projection_events(self.request(method, body)?)
                .await
                .map_err(|status| {
                    AppError::from_unimplemented(
                        "TransparencyGraphService.ListProjectionEvents",
                        status,
                    )
                })?
                .into_inner(),
        ))
    }

    pub async fn get_node(
        &mut self,
        body: svc::GetNodeRequest,
    ) -> Result<CrudResult<svc::GetNodeResponse>> {
        let method = RpcMethod::GraphGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.graph
                .get_node(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_nodes(
        &mut self,
        body: svc::ListNodesRequest,
    ) -> Result<CrudResult<svc::ListNodesResponse>> {
        let method = RpcMethod::GraphList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.graph
                .list_nodes(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_claims(
        &mut self,
        body: svc::ListClaimsRequest,
    ) -> Result<CrudResult<svc::ListClaimsResponse>> {
        let method = RpcMethod::TransparencyList;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .list_claims(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_claim(
        &mut self,
        body: svc::GetClaimRequest,
    ) -> Result<CrudResult<svc::GetClaimResponse>> {
        let method = RpcMethod::TransparencyGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .get_claim(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn list_verification_events(
        &mut self,
        body: svc::ListVerificationEventsRequest,
    ) -> Result<CrudResult<svc::ListVerificationEventsResponse>> {
        let method = RpcMethod::TransparencyVerificationEvents;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .list_verification_events(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn export_claim_receipt(
        &mut self,
        body: svc::ExportClaimReceiptRequest,
    ) -> Result<CrudResult<svc::ExportClaimReceiptResponse>> {
        let method = RpcMethod::TransparencyReceipt;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .export_claim_receipt(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    #[allow(dead_code)]
    pub async fn fetch_external_evidence(
        &mut self,
        body: svc::FetchExternalEvidenceRequest,
    ) -> Result<CrudResult<svc::FetchExternalEvidenceResponse>> {
        let method = RpcMethod::TransparencyFetch;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .fetch_external_evidence(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    #[allow(dead_code)]
    pub async fn list_fetch_events(
        &mut self,
        body: svc::ListFetchEventsRequest,
    ) -> Result<CrudResult<svc::ListFetchEventsResponse>> {
        let method = RpcMethod::TransparencyFetchEvents;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .list_fetch_events(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn get_evidence(
        &mut self,
        body: svc::GetEvidenceRequest,
    ) -> Result<CrudResult<svc::GetEvidenceResponse>> {
        let method = RpcMethod::TransparencyGet;
        Ok(CrudResult::new(
            Valence::Observed,
            self.transparency
                .get_evidence(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn verify_evidence(
        &mut self,
        body: svc::VerifyEvidenceRequest,
    ) -> Result<CrudResult<svc::VerifyEvidenceResponse>> {
        let method = RpcMethod::TransparencyVerify;
        let response = self
            .transparency
            .verify_evidence(self.request(method, body)?)
            .await?
            .into_inner();
        let valence = if response.digest_ok && response.size_ok && response.error.is_empty() {
            Valence::Attested
        } else if !response.error.is_empty() {
            Valence::Contradicted
        } else {
            Valence::Quarantined
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn verify_claim(
        &mut self,
        body: svc::VerifyClaimRequest,
    ) -> Result<CrudResult<svc::VerifyClaimResponse>> {
        let method = RpcMethod::TransparencyVerify;
        let response = self
            .transparency
            .verify_claim(self.request(method, body)?)
            .await?
            .into_inner();
        let valence = if response.diagnostics.is_empty() && response.trust_state == "verified" {
            Valence::Attested
        } else if !response.diagnostics.is_empty() {
            Valence::Contradicted
        } else {
            Valence::Quarantined
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn traverse(
        &mut self,
        body: svc::TraverseRequest,
    ) -> Result<CrudResult<svc::TraverseResponse>> {
        let method = RpcMethod::GraphAnalysis;
        Ok(CrudResult::new(
            Valence::Inferred,
            self.graph
                .traverse(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn shortest_path(
        &mut self,
        body: svc::ShortestPathRequest,
    ) -> Result<CrudResult<svc::ShortestPathResponse>> {
        let method = RpcMethod::GraphAnalysis;
        Ok(CrudResult::new(
            Valence::Inferred,
            self.graph
                .shortest_path(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn impact_radius(
        &mut self,
        body: svc::ImpactRadiusRequest,
    ) -> Result<CrudResult<svc::ImpactRadiusResponse>> {
        let method = RpcMethod::GraphAnalysis;
        Ok(CrudResult::new(
            Valence::Inferred,
            self.graph
                .impact_radius(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn explain_claim(
        &mut self,
        body: svc::ExplainClaimRequest,
    ) -> Result<CrudResult<svc::ExplainClaimResponse>> {
        let method = RpcMethod::GraphAnalysis;
        Ok(CrudResult::new(
            Valence::Inferred,
            self.graph
                .explain_claim(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn compute_trust_state(
        &mut self,
        body: svc::ComputeTrustStateRequest,
    ) -> Result<CrudResult<svc::ComputeTrustStateResponse>> {
        let method = RpcMethod::GraphAnalysis;
        Ok(CrudResult::new(
            Valence::Inferred,
            self.graph
                .compute_trust_state(self.request(method, body)?)
                .await?
                .into_inner(),
        ))
    }

    pub async fn verify_closure(
        &mut self,
        body: svc::VerifyClosureRequest,
    ) -> Result<CrudResult<svc::VerifyClosureResponse>> {
        let method = RpcMethod::GraphAnalysis;
        let response = self
            .graph
            .verify_closure(self.request(method, body)?)
            .await?
            .into_inner();
        let valence = if response.verdict {
            Valence::Attested
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    // Oscal write methods
    pub async fn create_catalog(
        &mut self,
        body: svc::CreateCatalogRequest,
    ) -> Result<CrudResult<svc::CreateCatalogResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::Catalog,
        };
        let response = self
            .oscal
            .create_catalog(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.catalog.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_catalog(
        &mut self,
        body: svc::UpdateCatalogRequest,
    ) -> Result<CrudResult<svc::UpdateCatalogResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::Catalog,
        };
        let response = self
            .oscal
            .update_catalog(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.catalog.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_catalog(
        &mut self,
        body: svc::DeleteCatalogRequest,
    ) -> Result<CrudResult<svc::DeleteCatalogResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::Catalog,
        };
        let response = self
            .oscal
            .delete_catalog(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_profile(
        &mut self,
        body: svc::CreateProfileRequest,
    ) -> Result<CrudResult<svc::CreateProfileResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::Profile,
        };
        let response = self
            .oscal
            .create_profile(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.profile.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_profile(
        &mut self,
        body: svc::UpdateProfileRequest,
    ) -> Result<CrudResult<svc::UpdateProfileResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::Profile,
        };
        let response = self
            .oscal
            .update_profile(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.profile.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_profile(
        &mut self,
        body: svc::DeleteProfileRequest,
    ) -> Result<CrudResult<svc::DeleteProfileResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::Profile,
        };
        let response = self
            .oscal
            .delete_profile(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_component_definition(
        &mut self,
        body: svc::CreateComponentDefinitionRequest,
    ) -> Result<CrudResult<svc::CreateComponentDefinitionResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::ComponentDefinition,
        };
        let response = self
            .oscal
            .create_component_definition(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.component_definition.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_component_definition(
        &mut self,
        body: svc::UpdateComponentDefinitionRequest,
    ) -> Result<CrudResult<svc::UpdateComponentDefinitionResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::ComponentDefinition,
        };
        let response = self
            .oscal
            .update_component_definition(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.component_definition.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_component_definition(
        &mut self,
        body: svc::DeleteComponentDefinitionRequest,
    ) -> Result<CrudResult<svc::DeleteComponentDefinitionResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::ComponentDefinition,
        };
        let response = self
            .oscal
            .delete_component_definition(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_ssp(
        &mut self,
        body: svc::CreateSspRequest,
    ) -> Result<CrudResult<svc::CreateSspResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::Ssp,
        };
        let response = self
            .oscal
            .create_ssp(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.ssp.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_ssp(
        &mut self,
        body: svc::UpdateSspRequest,
    ) -> Result<CrudResult<svc::UpdateSspResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::Ssp,
        };
        let response = self
            .oscal
            .update_ssp(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.ssp.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_ssp(
        &mut self,
        body: svc::DeleteSspRequest,
    ) -> Result<CrudResult<svc::DeleteSspResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::Ssp,
        };
        let response = self
            .oscal
            .delete_ssp(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_assessment_plan(
        &mut self,
        body: svc::CreateAssessmentPlanRequest,
    ) -> Result<CrudResult<svc::CreateAssessmentPlanResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::AssessmentPlan,
        };
        let response = self
            .oscal
            .create_assessment_plan(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.assessment_plan.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_assessment_plan(
        &mut self,
        body: svc::UpdateAssessmentPlanRequest,
    ) -> Result<CrudResult<svc::UpdateAssessmentPlanResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::AssessmentPlan,
        };
        let response = self
            .oscal
            .update_assessment_plan(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.assessment_plan.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_assessment_plan(
        &mut self,
        body: svc::DeleteAssessmentPlanRequest,
    ) -> Result<CrudResult<svc::DeleteAssessmentPlanResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::AssessmentPlan,
        };
        let response = self
            .oscal
            .delete_assessment_plan(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_assessment_results(
        &mut self,
        body: svc::CreateAssessmentResultsRequest,
    ) -> Result<CrudResult<svc::CreateAssessmentResultsResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::AssessmentResults,
        };
        let response = self
            .oscal
            .create_assessment_results(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.assessment_results.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_assessment_results(
        &mut self,
        body: svc::UpdateAssessmentResultsRequest,
    ) -> Result<CrudResult<svc::UpdateAssessmentResultsResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::AssessmentResults,
        };
        let response = self
            .oscal
            .update_assessment_results(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.assessment_results.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_assessment_results(
        &mut self,
        body: svc::DeleteAssessmentResultsRequest,
    ) -> Result<CrudResult<svc::DeleteAssessmentResultsResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::AssessmentResults,
        };
        let response = self
            .oscal
            .delete_assessment_results(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_poam(
        &mut self,
        body: svc::CreatePoamRequest,
    ) -> Result<CrudResult<svc::CreatePoamResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::Poam,
        };
        let response = self
            .oscal
            .create_poam(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.poam.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_poam(
        &mut self,
        body: svc::UpdatePoamRequest,
    ) -> Result<CrudResult<svc::UpdatePoamResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::Poam,
        };
        let response = self
            .oscal
            .update_poam(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.poam.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_poam(
        &mut self,
        body: svc::DeletePoamRequest,
    ) -> Result<CrudResult<svc::DeletePoamResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::Poam,
        };
        let response = self
            .oscal
            .delete_poam(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_mapping(
        &mut self,
        body: svc::CreateMappingRequest,
    ) -> Result<CrudResult<svc::CreateMappingResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Create,
            model: OscalModel::Mapping,
        };
        let response = self
            .oscal
            .create_mapping(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.mapping.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_mapping(
        &mut self,
        body: svc::UpdateMappingRequest,
    ) -> Result<CrudResult<svc::UpdateMappingResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Update,
            model: OscalModel::Mapping,
        };
        let response = self
            .oscal
            .update_mapping(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.mapping.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_mapping(
        &mut self,
        body: svc::DeleteMappingRequest,
    ) -> Result<CrudResult<svc::DeleteMappingResponse>> {
        let method = RpcMethod::OscalCrud {
            class: MethodClass::Delete,
            model: OscalModel::Mapping,
        };
        let response = self
            .oscal
            .delete_mapping(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    // Governance write methods
    pub async fn create_entity(
        &mut self,
        body: svc::CreateEntityRequest,
    ) -> Result<CrudResult<svc::CreateEntityResponse>> {
        let method = RpcMethod::GovernanceCrud {
            class: MethodClass::Create,
            model: GovernanceModel::Entity,
        };
        let response = self
            .governance
            .create_entity(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.entity.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn update_entity(
        &mut self,
        body: svc::UpdateEntityRequest,
    ) -> Result<CrudResult<svc::UpdateEntityResponse>> {
        let method = RpcMethod::GovernanceCrud {
            class: MethodClass::Update,
            model: GovernanceModel::Entity,
        };
        let response = self
            .governance
            .update_entity(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.entity.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_snapshot(
        &mut self,
        body: svc::CreateSnapshotRequest,
    ) -> Result<CrudResult<svc::CreateSnapshotResponse>> {
        let method = RpcMethod::GovernanceCrud {
            class: MethodClass::Create,
            model: GovernanceModel::Snapshot,
        };
        let response = self
            .governance
            .create_snapshot(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.name.is_empty() {
            Valence::Contradicted
        } else {
            Valence::Claimed
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn create_release(
        &mut self,
        body: svc::CreateReleaseRequest,
    ) -> Result<CrudResult<svc::CreateReleaseResponse>> {
        let method = RpcMethod::GovernanceCrud {
            class: MethodClass::Create,
            model: GovernanceModel::Release,
        };
        let response = self
            .governance
            .create_release(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.name.is_empty() || response.snapshot_name.is_empty() {
            Valence::Contradicted
        } else {
            Valence::Claimed
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn ingest_requirements(
        &mut self,
        body: svc::IngestRequirementsRequest,
    ) -> Result<CrudResult<svc::IngestRequirementsResponse>> {
        let method = RpcMethod::GovernanceCrud {
            class: MethodClass::Create,
            model: GovernanceModel::Framework,
        };
        let response = self
            .governance
            .ingest_requirements(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.conflicts > 0 {
            Valence::Contradicted
        } else {
            Valence::Claimed
        };
        Ok(CrudResult::new(valence, response))
    }

    // Transparency write methods
    pub async fn create_claim(
        &mut self,
        body: svc::CreateClaimRequest,
    ) -> Result<CrudResult<svc::CreateClaimResponse>> {
        let method = RpcMethod::ClaimWrite;
        let response = self
            .transparency
            .create_claim(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.claim.is_some() {
            if response.trust_state == "verified" {
                Valence::Attested
            } else {
                Valence::Claimed
            }
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn upload_evidence(
        &mut self,
        body: svc::UploadEvidenceRequest,
    ) -> Result<CrudResult<svc::UploadEvidenceResponse>> {
        let method = RpcMethod::EvidenceWrite;
        let response = self
            .transparency
            .upload_evidence(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.stored && response.evidence.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn sync_claims(
        &mut self,
        body: svc::SyncClaimsRequest,
    ) -> Result<CrudResult<svc::SyncClaimsResponse>> {
        let method = RpcMethod::ClaimWrite;
        let response = self
            .transparency
            .sync_claims(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.failed == 0 {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn import_batch(
        &mut self,
        body: svc::ImportBatchRequest,
    ) -> Result<CrudResult<svc::ImportBatchResponse>> {
        let method = RpcMethod::EvidenceWrite;
        let response = self
            .transparency
            .import_batch(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.committed
            && !response
                .results
                .iter()
                .any(|result| result.status == "invalid")
        {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn preflight_import(
        &mut self,
        body: svc::PreflightImportRequest,
    ) -> Result<CrudResult<svc::PreflightImportResponse>> {
        let method = RpcMethod::EvidenceWrite;
        let response = self
            .transparency
            .preflight_import(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.valid
            && !response
                .results
                .iter()
                .any(|result| result.status == "invalid")
        {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    // Graph write methods
    pub async fn project_edge(
        &mut self,
        body: svc::ProjectEdgeRequest,
    ) -> Result<CrudResult<svc::ProjectEdgeResponse>> {
        let method = RpcMethod::GraphEdgeWrite;
        let response = self
            .graph
            .project_edge(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.edge.is_some() && response.projection_event.is_some() {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }

    pub async fn delete_edge(
        &mut self,
        body: svc::DeleteEdgeRequest,
    ) -> Result<CrudResult<svc::DeleteEdgeResponse>> {
        let method = RpcMethod::GraphEdgeWrite;
        let response = self
            .graph
            .delete_edge(self.request(method, body)?)
            .await
            .map_err(|status| negative(&method, status))?
            .into_inner();
        let valence = if response.success {
            Valence::Claimed
        } else {
            Valence::Contradicted
        };
        Ok(CrudResult::new(valence, response))
    }
}
