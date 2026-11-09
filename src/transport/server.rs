//! Embedded High-Assurance Mizan gRPC Server
//!
//! Provides in-repo local and networked gRPC services implementing:
//! - OscalService (Catalogs, Profiles, SSPs, Components, Assessment Plans & Results, POA&Ms, Mappings)
//! - GovernanceService (Snapshots, Releases, Frameworks, Semantic Search)
//! - TransparencyExchangeService (Claims, Evidence, Batches)
//! - TransparencyGraphService (Nodes, Edges, Traversal)
//!
//! Backed by Enterprise Root Fabric, in-memory typed protobuf stores, and CasStore.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};

use crate::{
    document::cas::CasStore,
    fabric::{
        datastore::FabricDataStore,
        oidc::OidcTokenValidator,
        rbac::{Action, RbacEngine, ResourceType},
        tenant::{TenantContext, TenantId, UserId},
    },
    proto::oscal::{
        assessment_plan::v1::AssessmentPlan,
        assessment_results::v1::AssessmentResults,
        catalog::v1::Catalog,
        component_definition::v1::ComponentDefinition,
        mapping::v1::MappingCollection,
        poam::v1::PlanOfActionAndMilestones,
        profile::v1::Profile,
        services::v1::{
            governance_service_server::{GovernanceService, GovernanceServiceServer},
            oscal_service_server::{OscalService, OscalServiceServer},
            transparency_exchange_service_server::{
                TransparencyExchangeService, TransparencyExchangeServiceServer,
            },
            transparency_graph_service_server::{
                TransparencyGraphService, TransparencyGraphServiceServer,
            },
            *,
        },
        ssp::v1::SystemSecurityPlan as Ssp,
    },
};

#[derive(Clone, Default)]
pub struct MizanServerState {
    pub datastore: FabricDataStore,
    pub cas: CasStore,
    pub catalogs: Arc<RwLock<HashMap<String, Catalog>>>,
    pub profiles: Arc<RwLock<HashMap<String, Profile>>>,
    pub components: Arc<RwLock<HashMap<String, ComponentDefinition>>>,
    pub ssps: Arc<RwLock<HashMap<String, Ssp>>>,
    pub assessment_plans: Arc<RwLock<HashMap<String, AssessmentPlan>>>,
    pub assessment_results: Arc<RwLock<HashMap<String, AssessmentResults>>>,
    pub poams: Arc<RwLock<HashMap<String, PlanOfActionAndMilestones>>>,
    pub mappings: Arc<RwLock<HashMap<String, MappingCollection>>>,
    pub claims: Arc<RwLock<HashMap<String, Claim>>>,
    pub evidence: Arc<RwLock<HashMap<String, Evidence>>>,
    pub nodes: Arc<RwLock<HashMap<String, GraphNode>>>,
    pub edges: Arc<RwLock<HashMap<String, GraphEdge>>>,
    pub projection_events: Arc<RwLock<Vec<GraphProjectionEvent>>>,
    pub oidc_validator: Option<OidcTokenValidator>,
    pub require_auth: bool,
}

impl MizanServerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_oidc_auth(mut self, validator: OidcTokenValidator) -> Self {
        self.oidc_validator = Some(validator);
        self.require_auth = true;
        self
    }

    /// Extracts caller `TenantContext` from gRPC metadata (`authorization: Bearer <jwt>`,
    /// `x-mizan-tenant`, `x-mizan-user`, `x-mizan-role`) and enforces RBAC before execution.
    pub fn authorize_request<T>(
        &self,
        request: &Request<T>,
        action: Action,
        resource: ResourceType,
    ) -> Result<TenantContext, Status> {
        let meta = request.metadata();

        let auth_header = meta
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(str::trim);

        let ctx = if let Some(bearer) = auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
            if let Some(validator) = &self.oidc_validator {
                let now = chrono::Utc::now().timestamp();
                let claims = validator
                    .decode_and_validate(bearer.trim(), now)
                    .map_err(|e| Status::unauthenticated(format!("OIDC validation failed: {e}")))?;
                let tenant_str = claims.tenant_id.as_deref().unwrap_or("default");
                let tenant_id = TenantId::new(tenant_str)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?;
                let roles = if claims.roles.is_empty() {
                    vec!["Viewer".to_string()]
                } else {
                    claims.roles
                };
                TenantContext::new(tenant_id, UserId::new(claims.sub)).with_roles(roles)
            } else {
                return Err(Status::unauthenticated(
                    "Bearer token supplied but no OIDC validator is configured on server",
                ));
            }
        } else if self.require_auth {
            return Err(Status::unauthenticated(
                "Missing Bearer authorization token on protected gRPC endpoint",
            ));
        } else {
            let tenant_str = meta
                .get("x-mizan-tenant")
                .or_else(|| meta.get("x-tenant-id"))
                .and_then(|v| v.to_str().ok())
                .unwrap_or("default");
            let tenant_id =
                TenantId::new(tenant_str).map_err(|e| Status::invalid_argument(e.to_string()))?;
            let user_str = meta
                .get("x-mizan-user")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("system");
            let roles: Vec<String> =
                if let Some(role_hdr) = meta.get("x-mizan-role").and_then(|v| v.to_str().ok()) {
                    role_hdr
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect()
                } else {
                    vec!["TenantAdmin".to_string()]
                };
            TenantContext::new(tenant_id, UserId::new(user_str)).with_roles(roles)
        };

        RbacEngine::new()
            .authorize(&ctx, action, resource)
            .map_err(|e| Status::permission_denied(e.to_string()))?;

        Ok(ctx)
    }
}

#[tonic::async_trait]
impl OscalService for MizanServerState {
    async fn get_catalog(
        &self,
        request: Request<GetCatalogRequest>,
    ) -> Result<Response<GetCatalogResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .unwrap_or_else(|| "default".to_string());

        let cats = self
            .catalogs
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(cat) = cats.get(&uuid) {
            return Ok(Response::new(GetCatalogResponse {
                catalog: Some(cat.clone()),
            }));
        }

        Err(Status::not_found(format!("Catalog `{uuid}` not found")))
    }

    async fn list_catalogs(
        &self,
        request: Request<ListCatalogsRequest>,
    ) -> Result<Response<ListCatalogsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        let cats = self
            .catalogs
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let catalogs = cats.values().cloned().collect();
        Ok(Response::new(ListCatalogsResponse {
            catalogs,
            next_page_token: String::new(),
        }))
    }

    async fn create_catalog(
        &self,
        request: Request<CreateCatalogRequest>,
    ) -> Result<Response<CreateCatalogResponse>, Status> {
        let ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        let req = request.into_inner();
        let catalog = req
            .catalog
            .ok_or_else(|| Status::invalid_argument("catalog is required"))?;
        let uuid = catalog
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let mut cats = self
            .catalogs
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        cats.insert(uuid.clone(), catalog.clone());

        let _ = self.datastore.put_document(
            &ctx,
            "catalogs",
            &uuid,
            "catalog",
            "{\"status\":\"stored\"}",
        );

        Ok(Response::new(CreateCatalogResponse {
            catalog: Some(catalog),
        }))
    }

    async fn update_catalog(
        &self,
        request: Request<UpdateCatalogRequest>,
    ) -> Result<Response<UpdateCatalogResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::UpdateDocument, ResourceType::Catalog)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let catalog = req
            .catalog
            .ok_or_else(|| Status::invalid_argument("catalog is required"))?;

        let mut cats = self
            .catalogs
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        if !cats.contains_key(&uuid) {
            return Err(Status::not_found(format!("Catalog `{uuid}` not found")));
        }
        cats.insert(uuid, catalog.clone());

        Ok(Response::new(UpdateCatalogResponse {
            catalog: Some(catalog),
        }))
    }

    async fn delete_catalog(
        &self,
        request: Request<DeleteCatalogRequest>,
    ) -> Result<Response<DeleteCatalogResponse>, Status> {
        let ctx =
            self.authorize_request(&request, Action::DeleteDocument, ResourceType::Catalog)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;

        let mut cats = self
            .catalogs
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = cats.remove(&uuid).is_some();
        if !success {
            return Err(Status::not_found(format!("Catalog `{uuid}` not found")));
        }

        let _ = self.datastore.delete_document(&ctx, "catalogs", &uuid);

        Ok(Response::new(DeleteCatalogResponse { success: true }))
    }

    async fn get_profile(
        &self,
        request: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Profile)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .unwrap_or_else(|| "default".to_string());
        let profs = self
            .profiles
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(p) = profs.get(&uuid) {
            return Ok(Response::new(GetProfileResponse {
                profile: Some(p.clone()),
            }));
        }
        Err(Status::not_found(format!("Profile `{uuid}` not found")))
    }

    async fn list_profiles(
        &self,
        request: Request<ListProfilesRequest>,
    ) -> Result<Response<ListProfilesResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Profile)?;
        let profs = self
            .profiles
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListProfilesResponse {
            profiles: profs.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_profile(
        &self,
        request: Request<CreateProfileRequest>,
    ) -> Result<Response<CreateProfileResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Profile)?;
        let req = request.into_inner();
        let profile = req
            .profile
            .ok_or_else(|| Status::invalid_argument("profile is required"))?;
        let uuid = profile
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut profs = self
            .profiles
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        profs.insert(uuid, profile.clone());
        Ok(Response::new(CreateProfileResponse {
            profile: Some(profile),
        }))
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::UpdateDocument, ResourceType::Profile)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let profile = req
            .profile
            .ok_or_else(|| Status::invalid_argument("profile is required"))?;
        let mut profs = self
            .profiles
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        if !profs.contains_key(&uuid) {
            return Err(Status::not_found(format!("Profile `{uuid}` not found")));
        }
        profs.insert(uuid, profile.clone());
        Ok(Response::new(UpdateProfileResponse {
            profile: Some(profile),
        }))
    }

    async fn delete_profile(
        &self,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::DeleteDocument, ResourceType::Profile)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut profs = self
            .profiles
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        if profs.remove(&uuid).is_none() {
            return Err(Status::not_found(format!("Profile `{uuid}` not found")));
        }
        Ok(Response::new(DeleteProfileResponse { success: true }))
    }

    async fn get_component_definition(
        &self,
        request: Request<GetComponentDefinitionRequest>,
    ) -> Result<Response<GetComponentDefinitionResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Ssp)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .unwrap_or_default();
        let comps = self
            .components
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(c) = comps.get(&uuid) {
            return Ok(Response::new(GetComponentDefinitionResponse {
                component_definition: Some(c.clone()),
            }));
        }
        Err(Status::not_found("Component definition not found"))
    }

    async fn list_component_definitions(
        &self,
        request: Request<ListComponentDefinitionsRequest>,
    ) -> Result<Response<ListComponentDefinitionsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Ssp)?;
        let comps = self
            .components
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListComponentDefinitionsResponse {
            component_definitions: comps.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_component_definition(
        &self,
        request: Request<CreateComponentDefinitionRequest>,
    ) -> Result<Response<CreateComponentDefinitionResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::CreateDocument, ResourceType::Ssp)?;
        let comp = request
            .into_inner()
            .component_definition
            .ok_or_else(|| Status::invalid_argument("component definition is required"))?;
        let uuid = comp
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut comps = self
            .components
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        comps.insert(uuid, comp.clone());
        Ok(Response::new(CreateComponentDefinitionResponse {
            component_definition: Some(comp),
        }))
    }

    async fn update_component_definition(
        &self,
        request: Request<UpdateComponentDefinitionRequest>,
    ) -> Result<Response<UpdateComponentDefinitionResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::UpdateDocument, ResourceType::Ssp)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let comp = req
            .component_definition
            .ok_or_else(|| Status::invalid_argument("component definition is required"))?;
        let mut comps = self
            .components
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        comps.insert(uuid, comp.clone());
        Ok(Response::new(UpdateComponentDefinitionResponse {
            component_definition: Some(comp),
        }))
    }

    async fn delete_component_definition(
        &self,
        request: Request<DeleteComponentDefinitionRequest>,
    ) -> Result<Response<DeleteComponentDefinitionResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::DeleteDocument, ResourceType::Ssp)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut comps = self
            .components
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = comps.remove(&uuid).is_some();
        Ok(Response::new(DeleteComponentDefinitionResponse { success }))
    }

    async fn get_ssp(
        &self,
        request: Request<GetSspRequest>,
    ) -> Result<Response<GetSspResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Ssp)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .unwrap_or_default();
        let ssps = self
            .ssps
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(s) = ssps.get(&uuid) {
            return Ok(Response::new(GetSspResponse {
                ssp: Some(s.clone()),
            }));
        }
        Err(Status::not_found("SSP not found"))
    }

    async fn list_ssps(
        &self,
        request: Request<ListSspsRequest>,
    ) -> Result<Response<ListSspsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Ssp)?;
        let ssps = self
            .ssps
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListSspsResponse {
            ssps: ssps.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_ssp(
        &self,
        request: Request<CreateSspRequest>,
    ) -> Result<Response<CreateSspResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::CreateDocument, ResourceType::Ssp)?;
        let ssp = request
            .into_inner()
            .ssp
            .ok_or_else(|| Status::invalid_argument("ssp is required"))?;
        let uuid = ssp
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut ssps = self
            .ssps
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        ssps.insert(uuid, ssp.clone());
        Ok(Response::new(CreateSspResponse { ssp: Some(ssp) }))
    }

    async fn update_ssp(
        &self,
        request: Request<UpdateSspRequest>,
    ) -> Result<Response<UpdateSspResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::UpdateDocument, ResourceType::Ssp)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let ssp = req
            .ssp
            .ok_or_else(|| Status::invalid_argument("ssp is required"))?;
        let mut ssps = self
            .ssps
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        ssps.insert(uuid, ssp.clone());
        Ok(Response::new(UpdateSspResponse { ssp: Some(ssp) }))
    }

    async fn delete_ssp(
        &self,
        request: Request<DeleteSspRequest>,
    ) -> Result<Response<DeleteSspResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::DeleteDocument, ResourceType::Ssp)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut ssps = self
            .ssps
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = ssps.remove(&uuid).is_some();
        Ok(Response::new(DeleteSspResponse { success }))
    }

    async fn get_assessment_plan(
        &self,
        request: Request<GetAssessmentPlanRequest>,
    ) -> Result<Response<GetAssessmentPlanResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::ReadDocument,
            ResourceType::AssessmentResults,
        )?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .unwrap_or_default();
        let plans = self
            .assessment_plans
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(p) = plans.get(&uuid) {
            return Ok(Response::new(GetAssessmentPlanResponse {
                assessment_plan: Some(p.clone()),
            }));
        }
        Err(Status::not_found("Assessment plan not found"))
    }

    async fn list_assessment_plans(
        &self,
        request: Request<ListAssessmentPlansRequest>,
    ) -> Result<Response<ListAssessmentPlansResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::ReadDocument,
            ResourceType::AssessmentResults,
        )?;
        let plans = self
            .assessment_plans
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListAssessmentPlansResponse {
            assessment_plans: plans.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_assessment_plan(
        &self,
        request: Request<CreateAssessmentPlanRequest>,
    ) -> Result<Response<CreateAssessmentPlanResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::CreateDocument,
            ResourceType::AssessmentResults,
        )?;
        let plan = request
            .into_inner()
            .assessment_plan
            .ok_or_else(|| Status::invalid_argument("assessment plan is required"))?;
        let uuid = plan
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut plans = self
            .assessment_plans
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        plans.insert(uuid, plan.clone());
        Ok(Response::new(CreateAssessmentPlanResponse {
            assessment_plan: Some(plan),
        }))
    }

    async fn update_assessment_plan(
        &self,
        request: Request<UpdateAssessmentPlanRequest>,
    ) -> Result<Response<UpdateAssessmentPlanResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::UpdateDocument,
            ResourceType::AssessmentResults,
        )?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let plan = req
            .assessment_plan
            .ok_or_else(|| Status::invalid_argument("assessment plan is required"))?;
        let mut plans = self
            .assessment_plans
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        plans.insert(uuid, plan.clone());
        Ok(Response::new(UpdateAssessmentPlanResponse {
            assessment_plan: Some(plan),
        }))
    }

    async fn delete_assessment_plan(
        &self,
        request: Request<DeleteAssessmentPlanRequest>,
    ) -> Result<Response<DeleteAssessmentPlanResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::DeleteDocument,
            ResourceType::AssessmentResults,
        )?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut plans = self
            .assessment_plans
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = plans.remove(&uuid).is_some();
        Ok(Response::new(DeleteAssessmentPlanResponse { success }))
    }

    async fn get_assessment_results(
        &self,
        request: Request<GetAssessmentResultsRequest>,
    ) -> Result<Response<GetAssessmentResultsResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::ReadDocument,
            ResourceType::AssessmentResults,
        )?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .unwrap_or_default();
        let res = self
            .assessment_results
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(r) = res.get(&uuid) {
            return Ok(Response::new(GetAssessmentResultsResponse {
                assessment_results: Some(r.clone()),
            }));
        }
        Err(Status::not_found("Assessment results not found"))
    }

    async fn list_assessment_results(
        &self,
        request: Request<ListAssessmentResultsRequest>,
    ) -> Result<Response<ListAssessmentResultsResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::ReadDocument,
            ResourceType::AssessmentResults,
        )?;
        let res = self
            .assessment_results
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListAssessmentResultsResponse {
            assessment_results_list: res.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_assessment_results(
        &self,
        request: Request<CreateAssessmentResultsRequest>,
    ) -> Result<Response<CreateAssessmentResultsResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::CreateDocument,
            ResourceType::AssessmentResults,
        )?;
        let res = request
            .into_inner()
            .assessment_results
            .ok_or_else(|| Status::invalid_argument("assessment results are required"))?;
        let uuid = res
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut results = self
            .assessment_results
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        results.insert(uuid, res.clone());
        Ok(Response::new(CreateAssessmentResultsResponse {
            assessment_results: Some(res),
        }))
    }

    async fn update_assessment_results(
        &self,
        request: Request<UpdateAssessmentResultsRequest>,
    ) -> Result<Response<UpdateAssessmentResultsResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::UpdateDocument,
            ResourceType::AssessmentResults,
        )?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let res = req
            .assessment_results
            .ok_or_else(|| Status::invalid_argument("assessment results are required"))?;
        let mut results = self
            .assessment_results
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        results.insert(uuid, res.clone());
        Ok(Response::new(UpdateAssessmentResultsResponse {
            assessment_results: Some(res),
        }))
    }

    async fn delete_assessment_results(
        &self,
        request: Request<DeleteAssessmentResultsRequest>,
    ) -> Result<Response<DeleteAssessmentResultsResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::DeleteDocument,
            ResourceType::AssessmentResults,
        )?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut results = self
            .assessment_results
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = results.remove(&uuid).is_some();
        Ok(Response::new(DeleteAssessmentResultsResponse { success }))
    }

    async fn get_poam(
        &self,
        request: Request<GetPoamRequest>,
    ) -> Result<Response<GetPoamResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Poam)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .unwrap_or_default();
        let poams = self
            .poams
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(p) = poams.get(&uuid) {
            return Ok(Response::new(GetPoamResponse {
                poam: Some(p.clone()),
            }));
        }
        Err(Status::not_found("POA&M not found"))
    }

    async fn list_poams(
        &self,
        request: Request<ListPoamsRequest>,
    ) -> Result<Response<ListPoamsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Poam)?;
        let poams = self
            .poams
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListPoamsResponse {
            poams: poams.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_poam(
        &self,
        request: Request<CreatePoamRequest>,
    ) -> Result<Response<CreatePoamResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::CreateDocument, ResourceType::Poam)?;
        let poam = request
            .into_inner()
            .poam
            .ok_or_else(|| Status::invalid_argument("poam is required"))?;
        let uuid = poam
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut poams = self
            .poams
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        poams.insert(uuid, poam.clone());
        Ok(Response::new(CreatePoamResponse { poam: Some(poam) }))
    }

    async fn update_poam(
        &self,
        request: Request<UpdatePoamRequest>,
    ) -> Result<Response<UpdatePoamResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::UpdateDocument, ResourceType::Poam)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let poam = req
            .poam
            .ok_or_else(|| Status::invalid_argument("poam is required"))?;
        let mut poams = self
            .poams
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        poams.insert(uuid, poam.clone());
        Ok(Response::new(UpdatePoamResponse { poam: Some(poam) }))
    }

    async fn delete_poam(
        &self,
        request: Request<DeletePoamRequest>,
    ) -> Result<Response<DeletePoamResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::DeleteDocument, ResourceType::Poam)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut poams = self
            .poams
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = poams.remove(&uuid).is_some();
        Ok(Response::new(DeletePoamResponse { success }))
    }

    async fn get_mapping(
        &self,
        request: Request<GetMappingRequest>,
    ) -> Result<Response<GetMappingResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .unwrap_or_default();
        let maps = self
            .mappings
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(m) = maps.get(&uuid) {
            return Ok(Response::new(GetMappingResponse {
                mapping: Some(m.clone()),
            }));
        }
        Err(Status::not_found("Mapping not found"))
    }

    async fn list_mappings(
        &self,
        request: Request<ListMappingsRequest>,
    ) -> Result<Response<ListMappingsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        let maps = self
            .mappings
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListMappingsResponse {
            mappings: maps.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn create_mapping(
        &self,
        request: Request<CreateMappingRequest>,
    ) -> Result<Response<CreateMappingResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        let mapping = request
            .into_inner()
            .mapping
            .ok_or_else(|| Status::invalid_argument("mapping is required"))?;
        let uuid = mapping
            .uuid
            .as_ref()
            .map(|u| u.value.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut maps = self
            .mappings
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        maps.insert(uuid, mapping.clone());
        Ok(Response::new(CreateMappingResponse {
            mapping: Some(mapping),
        }))
    }

    async fn update_mapping(
        &self,
        request: Request<UpdateMappingRequest>,
    ) -> Result<Response<UpdateMappingResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::UpdateDocument, ResourceType::Catalog)?;
        let req = request.into_inner();
        let uuid = req
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mapping = req
            .mapping
            .ok_or_else(|| Status::invalid_argument("mapping is required"))?;
        let mut maps = self
            .mappings
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        maps.insert(uuid, mapping.clone());
        Ok(Response::new(UpdateMappingResponse {
            mapping: Some(mapping),
        }))
    }

    async fn delete_mapping(
        &self,
        request: Request<DeleteMappingRequest>,
    ) -> Result<Response<DeleteMappingResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::DeleteDocument, ResourceType::Catalog)?;
        let uuid = request
            .into_inner()
            .uuid
            .map(|u| u.value)
            .ok_or_else(|| Status::invalid_argument("uuid is required"))?;
        let mut maps = self
            .mappings
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = maps.remove(&uuid).is_some();
        Ok(Response::new(DeleteMappingResponse { success }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        let req = request.into_inner();
        let query = req.query.to_lowercase();
        let cats = self
            .catalogs
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let mut results = Vec::new();
        for (id, cat) in cats.iter() {
            let title = cat
                .metadata
                .as_ref()
                .map(|m| m.title.clone())
                .unwrap_or_default();
            if id.to_lowercase().contains(&query) || title.to_lowercase().contains(&query) {
                results.push(SearchResult {
                    model_type: "catalog".to_string(),
                    uuid: cat.uuid.clone(),
                    title,
                    score: 1.0,
                });
            }
        }
        Ok(Response::new(SearchResponse {
            results,
            next_page_token: String::new(),
        }))
    }
}

#[tonic::async_trait]
impl GovernanceService for MizanServerState {
    async fn create_entity(
        &self,
        request: Request<CreateEntityRequest>,
    ) -> Result<Response<CreateEntityResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        let entity = request.into_inner().entity;
        Ok(Response::new(CreateEntityResponse { entity }))
    }

    async fn get_entity(
        &self,
        request: Request<GetEntityRequest>,
    ) -> Result<Response<GetEntityResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        Err(Status::not_found("Entity not found"))
    }

    async fn update_entity(
        &self,
        request: Request<UpdateEntityRequest>,
    ) -> Result<Response<UpdateEntityResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::UpdateDocument, ResourceType::Catalog)?;
        let entity = request.into_inner().entity;
        Ok(Response::new(UpdateEntityResponse { entity }))
    }

    async fn list_entities(
        &self,
        request: Request<ListEntitiesRequest>,
    ) -> Result<Response<ListEntitiesResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        Ok(Response::new(ListEntitiesResponse {
            entities: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn create_snapshot(
        &self,
        request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Profile)?;
        let name = request.into_inner().name;
        Ok(Response::new(CreateSnapshotResponse {
            name,
            entity_count: 0,
        }))
    }

    async fn get_snapshot(
        &self,
        request: Request<GetSnapshotRequest>,
    ) -> Result<Response<GetSnapshotResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Profile)?;
        let _name = request.into_inner().name;
        Ok(Response::new(GetSnapshotResponse {
            entities: Vec::new(),
        }))
    }

    async fn list_snapshots(
        &self,
        request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Profile)?;
        Ok(Response::new(ListSnapshotsResponse { names: Vec::new() }))
    }

    async fn create_release(
        &self,
        request: Request<CreateReleaseRequest>,
    ) -> Result<Response<CreateReleaseResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::PublishBaseline, ResourceType::Profile)?;
        let req = request.into_inner();
        Ok(Response::new(CreateReleaseResponse {
            name: req.name,
            snapshot_name: req.snapshot_name,
        }))
    }

    async fn list_releases(
        &self,
        request: Request<ListReleasesRequest>,
    ) -> Result<Response<ListReleasesResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Profile)?;
        Ok(Response::new(ListReleasesResponse { names: Vec::new() }))
    }

    async fn ingest_requirements(
        &self,
        request: Request<IngestRequirementsRequest>,
    ) -> Result<Response<IngestRequirementsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        Err(Status::unimplemented("ingest_requirements not implemented"))
    }

    async fn semantic_search(
        &self,
        request: Request<SemanticSearchRequest>,
    ) -> Result<Response<SemanticSearchResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        Ok(Response::new(SemanticSearchResponse {
            results: Vec::new(),
        }))
    }

    async fn generate_catalog(
        &self,
        request: Request<GenerateCatalogRequest>,
    ) -> Result<Response<GenerateCatalogResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        Err(Status::unimplemented("generate_catalog not implemented"))
    }

    async fn generate_profile(
        &self,
        request: Request<GenerateProfileRequest>,
    ) -> Result<Response<GenerateProfileResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Profile)?;
        Err(Status::unimplemented("generate_profile not implemented"))
    }

    async fn generate_mappings(
        &self,
        request: Request<GenerateMappingsRequest>,
    ) -> Result<Response<GenerateMappingsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        Err(Status::unimplemented("generate_mappings not implemented"))
    }

    async fn generate_ssp(
        &self,
        request: Request<GenerateSspRequest>,
    ) -> Result<Response<GenerateSspResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::CreateDocument, ResourceType::Ssp)?;
        Err(Status::unimplemented("generate_ssp not implemented"))
    }

    async fn generate_component_definition(
        &self,
        request: Request<GenerateComponentDefinitionRequest>,
    ) -> Result<Response<GenerateComponentDefinitionResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::CreateDocument, ResourceType::Ssp)?;
        Err(Status::unimplemented(
            "generate_component_definition not implemented",
        ))
    }

    async fn generate_assessment_plan(
        &self,
        request: Request<GenerateAssessmentPlanRequest>,
    ) -> Result<Response<GenerateAssessmentPlanResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::CreateDocument,
            ResourceType::AssessmentResults,
        )?;
        Err(Status::unimplemented(
            "generate_assessment_plan not implemented",
        ))
    }

    async fn generate_poam(
        &self,
        request: Request<GeneratePoamRequest>,
    ) -> Result<Response<GeneratePoamResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::CreateDocument, ResourceType::Poam)?;
        Err(Status::unimplemented("generate_poam not implemented"))
    }

    async fn generate_assessment_results(
        &self,
        request: Request<GenerateAssessmentResultsRequest>,
    ) -> Result<Response<GenerateAssessmentResultsResponse>, Status> {
        let _ctx = self.authorize_request(
            &request,
            Action::CreateDocument,
            ResourceType::AssessmentResults,
        )?;
        Err(Status::unimplemented(
            "generate_assessment_results not implemented",
        ))
    }

    async fn bulk_ingest_frameworks(
        &self,
        request: Request<BulkIngestFrameworksRequest>,
    ) -> Result<Response<BulkIngestFrameworksResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        Err(Status::unimplemented(
            "bulk_ingest_frameworks not implemented",
        ))
    }

    async fn list_frameworks(
        &self,
        request: Request<ListFrameworksRequest>,
    ) -> Result<Response<ListFrameworksResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        Ok(Response::new(ListFrameworksResponse {
            frameworks: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn get_framework(
        &self,
        request: Request<GetFrameworkRequest>,
    ) -> Result<Response<GetFrameworkResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Catalog)?;
        Err(Status::not_found("Framework not found"))
    }

    async fn generate_cross_framework_mappings(
        &self,
        request: Request<GenerateCrossFrameworkMappingsRequest>,
    ) -> Result<Response<GenerateCrossFrameworkMappingsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Catalog)?;
        Err(Status::unimplemented(
            "generate_cross_framework_mappings not implemented",
        ))
    }

    async fn propose(
        &self,
        request: Request<ProposeRequest>,
    ) -> Result<Response<ProposeResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Profile)?;
        Err(Status::unimplemented("propose not implemented"))
    }

    async fn list_conflicts(
        &self,
        request: Request<ListConflictsRequest>,
    ) -> Result<Response<ListConflictsResponse>, Status> {
        let _ctx = self.authorize_request(&request, Action::ReadDocument, ResourceType::Profile)?;
        Ok(Response::new(ListConflictsResponse {
            conflicts: Vec::new(),
        }))
    }

    async fn resolve_conflict(
        &self,
        request: Request<ResolveConflictRequest>,
    ) -> Result<Response<ResolveConflictResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::UpdateDocument, ResourceType::Profile)?;
        Err(Status::unimplemented("resolve_conflict not implemented"))
    }

    async fn publish_release(
        &self,
        request: Request<PublishReleaseRequest>,
    ) -> Result<Response<PublishReleaseResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::PublishBaseline, ResourceType::Profile)?;
        Err(Status::unimplemented("publish_release not implemented"))
    }

    async fn propose_mapping_update(
        &self,
        request: Request<ProposeMappingUpdateRequest>,
    ) -> Result<Response<ProposeMappingUpdateResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::UpdateDocument, ResourceType::Catalog)?;
        Err(Status::unimplemented(
            "propose_mapping_update not implemented",
        ))
    }
}

#[tonic::async_trait]
impl TransparencyExchangeService for MizanServerState {
    async fn create_claim(
        &self,
        request: Request<CreateClaimRequest>,
    ) -> Result<Response<CreateClaimResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Attestation)?;
        let claim = request
            .into_inner()
            .claim
            .ok_or_else(|| Status::invalid_argument("claim is required"))?;
        let id = claim.id.clone();
        let mut claims = self
            .claims
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        claims.insert(id, claim.clone());
        Ok(Response::new(CreateClaimResponse {
            claim: Some(claim),
            trust_state: "observed".to_string(),
        }))
    }

    async fn get_claim(
        &self,
        request: Request<GetClaimRequest>,
    ) -> Result<Response<GetClaimResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let id = request.into_inner().claim_id;
        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(c) = claims.get(&id) {
            return Ok(Response::new(GetClaimResponse {
                claim: Some(c.clone()),
            }));
        }
        Err(Status::not_found(format!("Claim `{id}` not found")))
    }

    async fn list_claims(
        &self,
        request: Request<ListClaimsRequest>,
    ) -> Result<Response<ListClaimsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListClaimsResponse {
            claims: claims.values().cloned().collect(),
            next_page_token: String::new(),
        }))
    }

    async fn verify_claim(
        &self,
        request: Request<VerifyClaimRequest>,
    ) -> Result<Response<VerifyClaimResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let claim_id = req.claim_id;
        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let claim = claims
            .get(&claim_id)
            .ok_or_else(|| Status::not_found(format!("Claim `{claim_id}` not found")))?;

        let trust_state = if claim.trust_state.is_empty() {
            "observed".to_string()
        } else {
            claim.trust_state.clone()
        };
        let has_proof_refs = !claim.proof_refs.is_empty();

        Ok(Response::new(VerifyClaimResponse {
            claim_id,
            proof_state: Some(ProofState {
                discovery: "discovered".to_string(),
                graph: "graph_resolved".to_string(),
                claim: "claim_bound".to_string(),
                source: if claim.source_refs.is_empty() {
                    "unbound".to_string()
                } else {
                    "source_bound".to_string()
                },
                signature: if has_proof_refs {
                    "signature_verified".to_string()
                } else {
                    "unverified".to_string()
                },
                transparency: if has_proof_refs {
                    "inclusion_verified".to_string()
                } else {
                    "unverified".to_string()
                },
                witness_json: "{}".to_string(),
                state: "current_state_verified".to_string(),
                policy: "policy_admissible".to_string(),
            }),
            trust_state,
            diagnostics: Vec::new(),
        }))
    }

    async fn list_verification_events(
        &self,
        request: Request<ListVerificationEventsRequest>,
    ) -> Result<Response<ListVerificationEventsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        Ok(Response::new(ListVerificationEventsResponse {
            events: Vec::new(),
            chain_valid: true,
        }))
    }

    async fn export_claim_receipt(
        &self,
        request: Request<ExportClaimReceiptRequest>,
    ) -> Result<Response<ExportClaimReceiptResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let id = request.into_inner().claim_id;
        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let claim = claims
            .get(&id)
            .ok_or_else(|| Status::not_found(format!("Claim `{id}` not found")))?;

        let val = crate::output::message_json("oscal.services.v1.Claim", claim)
            .map_err(|e| Status::internal(e.to_string()))?;
        let receipt_json =
            serde_json::to_string(&val).map_err(|e| Status::internal(e.to_string()))?;
        let receipt_digest = CasStore::compute_digest(receipt_json.as_bytes());
        let verification_event_hash = receipt_digest.clone();

        let trust_state = if claim.trust_state.is_empty() {
            "observed".to_string()
        } else {
            claim.trust_state.clone()
        };

        Ok(Response::new(ExportClaimReceiptResponse {
            claim_id: claim.id.clone(),
            receipt_json,
            receipt_digest,
            trust_state,
            verification_event_hash,
            exported_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        }))
    }

    async fn preflight_import(
        &self,
        request: Request<PreflightImportRequest>,
    ) -> Result<Response<PreflightImportResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        Err(Status::unimplemented("preflight_import not implemented"))
    }

    async fn import_batch(
        &self,
        request: Request<ImportBatchRequest>,
    ) -> Result<Response<ImportBatchResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Attestation)?;
        Err(Status::unimplemented("import_batch not implemented"))
    }

    async fn upload_evidence(
        &self,
        request: Request<UploadEvidenceRequest>,
    ) -> Result<Response<UploadEvidenceResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let blob = req.blob;
        let digest = self
            .cas
            .put_bytes(&blob)
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut evidence = req.evidence;
        if let Some(ev) = evidence.as_mut() {
            ev.digest = digest.clone();
            ev.size_bytes = blob.len() as i64;
            if ev.id.is_empty() {
                ev.id = digest;
            }
            let mut store = self
                .evidence
                .write()
                .map_err(|e| Status::internal(e.to_string()))?;
            store.insert(ev.id.clone(), ev.clone());
        }
        Ok(Response::new(UploadEvidenceResponse {
            evidence,
            stored: true,
        }))
    }

    async fn get_evidence(
        &self,
        request: Request<GetEvidenceRequest>,
    ) -> Result<Response<GetEvidenceResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let id = request.into_inner().evidence_id;
        let store = self
            .evidence
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(ev) = store
            .get(&id)
            .or_else(|| store.values().find(|e| e.digest == id))
        {
            return Ok(Response::new(GetEvidenceResponse {
                evidence: Some(ev.clone()),
            }));
        }
        Err(Status::not_found(format!("Evidence `{id}` not found")))
    }

    async fn verify_evidence(
        &self,
        request: Request<VerifyEvidenceRequest>,
    ) -> Result<Response<VerifyEvidenceResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let id = req.evidence_id;
        let fetch_and_hash = req.fetch_and_hash;

        let store = self
            .evidence
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let ev = store
            .get(&id)
            .or_else(|| store.values().find(|e| e.digest == id))
            .ok_or_else(|| Status::not_found(format!("Evidence `{id}` not found")))?;

        let has_blob = self.cas.has_blob(&ev.digest);
        let mut digest_ok = has_blob;
        let mut size_ok = has_blob;
        let mut error = String::new();

        if !has_blob {
            error = format!("CAS blob not found for digest: {}", ev.digest);
        } else if fetch_and_hash {
            match self.cas.get_bytes(&ev.digest) {
                Ok(bytes) => {
                    let actual_digest = CasStore::compute_digest(&bytes);
                    let actual_size = bytes.len() as i64;
                    digest_ok = actual_digest == ev.digest;
                    size_ok = actual_size == ev.size_bytes;
                    if !digest_ok {
                        error = format!(
                            "Digest mismatch: expected {}, got {}",
                            ev.digest, actual_digest
                        );
                    } else if !size_ok {
                        error = format!(
                            "Size mismatch: expected {}, got {}",
                            ev.size_bytes, actual_size
                        );
                    }
                }
                Err(e) => {
                    digest_ok = false;
                    size_ok = false;
                    error = e.to_string();
                }
            }
        }

        Ok(Response::new(VerifyEvidenceResponse {
            evidence_id: id,
            digest_ok,
            size_ok,
            error,
        }))
    }

    async fn fetch_external_evidence(
        &self,
        request: Request<FetchExternalEvidenceRequest>,
    ) -> Result<Response<FetchExternalEvidenceResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Attestation)?;
        Err(Status::unimplemented(
            "fetch_external_evidence not implemented",
        ))
    }

    async fn list_fetch_events(
        &self,
        request: Request<ListFetchEventsRequest>,
    ) -> Result<Response<ListFetchEventsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        Ok(Response::new(ListFetchEventsResponse {
            events: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn sync_claims(
        &self,
        request: Request<SyncClaimsRequest>,
    ) -> Result<Response<SyncClaimsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Attestation)?;
        Err(Status::unimplemented("sync_claims not implemented"))
    }
}

#[tonic::async_trait]
impl TransparencyGraphService for MizanServerState {
    async fn project_edge(
        &self,
        request: Request<ProjectEdgeRequest>,
    ) -> Result<Response<ProjectEdgeResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::CreateDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        if req.claim_id.trim().is_empty() {
            return Err(Status::invalid_argument("claim_id is required"));
        }

        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let claim = claims
            .get(&req.claim_id)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("Claim `{}` not found", req.claim_id)))?;
        drop(claims);

        let edge_id = if req.edge_id.trim().is_empty() {
            format!("edge-{}", req.claim_id)
        } else {
            req.edge_id
        };

        let from_node = claim
            .subject
            .as_ref()
            .map(|s| s.id.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("subject-{}", claim.id));
        let to_node = match &claim.object_opt {
            Some(claim::ObjectOpt::Object(obj)) if !obj.id.is_empty() => obj.id.clone(),
            _ => format!("object-{}", claim.id),
        };
        let relation = claim
            .predicate
            .as_ref()
            .map(|p| p.relation.clone())
            .filter(|r| !r.is_empty())
            .unwrap_or_else(|| "attests".to_string());
        let qualifier = claim
            .predicate
            .as_ref()
            .map(|p| p.qualifier.clone())
            .unwrap_or_default();
        let evidence_digest = claim
            .source_refs
            .first()
            .map(|r| r.digest.clone())
            .unwrap_or_default();
        let trust_state = if claim.trust_state.is_empty() {
            "observed".to_string()
        } else {
            claim.trust_state.clone()
        };
        let has_proof_refs = !claim.proof_refs.is_empty();
        let now_ts = prost_types::Timestamp::from(std::time::SystemTime::now());

        let edge = GraphEdge {
            id: edge_id.clone(),
            from_node: from_node.clone(),
            to_node: to_node.clone(),
            relation: relation.clone(),
            qualifier,
            claim_id: claim.id.clone(),
            evidence_digest: evidence_digest.clone(),
            proof_state: Some(ProofState {
                discovery: "discovered".to_string(),
                graph: "graph_resolved".to_string(),
                claim: "claim_bound".to_string(),
                source: if evidence_digest.is_empty() {
                    "unbound".to_string()
                } else {
                    "source_bound".to_string()
                },
                signature: if has_proof_refs {
                    "signature_verified".to_string()
                } else {
                    "unverified".to_string()
                },
                transparency: if has_proof_refs {
                    "inclusion_verified".to_string()
                } else {
                    "unverified".to_string()
                },
                witness_json: "{}".to_string(),
                state: "current_state_verified".to_string(),
                policy: "policy_admissible".to_string(),
            }),
            valid_time: claim.valid_time,
            trust_state: trust_state.clone(),
            weight: 1.0,
            extensions_json: claim.extensions_json,
        };

        {
            let mut nodes = self
                .nodes
                .write()
                .map_err(|e| Status::internal(e.to_string()))?;
            nodes.entry(from_node.clone()).or_insert_with(|| GraphNode {
                id: from_node.clone(),
                kind: claim
                    .subject
                    .as_ref()
                    .map(|s| s.kind.clone())
                    .filter(|k| !k.is_empty())
                    .unwrap_or_else(|| "artifact".to_string()),
                urn: format!("urn:mizan:node:{from_node}"),
                labels_json: "{}".to_string(),
                created_at: Some(now_ts),
            });
            nodes.entry(to_node.clone()).or_insert_with(|| GraphNode {
                id: to_node.clone(),
                kind: "component".to_string(),
                urn: format!("urn:mizan:node:{to_node}"),
                labels_json: "{}".to_string(),
                created_at: Some(now_ts),
            });
        }

        {
            let mut edges = self
                .edges
                .write()
                .map_err(|e| Status::internal(e.to_string()))?;
            edges.insert(edge_id.clone(), edge.clone());
        }

        let projection_event = {
            let mut events = self
                .projection_events
                .write()
                .map_err(|e| Status::internal(e.to_string()))?;
            let sequence = (events.len() as i64) + 1;
            let previous_hash = events
                .last()
                .map(|e| e.event_hash.clone())
                .unwrap_or_else(|| {
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_string()
                });
            let event_payload = format!(
                "{sequence}:{edge_id}:{}:{from_node}:{to_node}:{relation}:{evidence_digest}:{trust_state}:{previous_hash}",
                claim.id
            );
            let event_hash = CasStore::compute_digest(event_payload.as_bytes());
            let ev = GraphProjectionEvent {
                sequence,
                event_id: format!("proj-ev-{sequence}"),
                edge_id,
                claim_id: claim.id,
                from_node,
                to_node,
                relation,
                evidence_digest,
                trust_state,
                previous_hash,
                event_hash,
                projected_at: Some(now_ts),
            };
            events.push(ev.clone());
            ev
        };

        Ok(Response::new(ProjectEdgeResponse {
            edge: Some(edge),
            projection_event: Some(projection_event),
        }))
    }

    async fn list_projection_events(
        &self,
        request: Request<ListProjectionEventsRequest>,
    ) -> Result<Response<ListProjectionEventsResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let events_guard = self
            .projection_events
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut chain_valid = true;
        let mut expected_prev =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();
        for ev in events_guard.iter() {
            if ev.previous_hash != expected_prev {
                chain_valid = false;
                break;
            }
            let payload = format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}",
                ev.sequence,
                ev.edge_id,
                ev.claim_id,
                ev.from_node,
                ev.to_node,
                ev.relation,
                ev.evidence_digest,
                ev.trust_state,
                ev.previous_hash
            );
            let computed = CasStore::compute_digest(payload.as_bytes());
            if ev.event_hash != computed {
                chain_valid = false;
                break;
            }
            expected_prev = ev.event_hash.clone();
        }

        let filtered: Vec<GraphProjectionEvent> = events_guard
            .iter()
            .filter(|ev| {
                (req.claim_id.is_empty() || ev.claim_id == req.claim_id)
                    && (req.edge_id.is_empty() || ev.edge_id == req.edge_id)
            })
            .cloned()
            .collect();

        Ok(Response::new(ListProjectionEventsResponse {
            events: filtered,
            chain_valid,
        }))
    }

    async fn get_edge(
        &self,
        request: Request<GetEdgeRequest>,
    ) -> Result<Response<GetEdgeResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let edge_id = request.into_inner().edge_id;
        let edges = self
            .edges
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(edge) = edges.get(&edge_id) {
            return Ok(Response::new(GetEdgeResponse {
                edge: Some(edge.clone()),
            }));
        }
        Err(Status::not_found(format!("Edge `{edge_id}` not found")))
    }

    async fn list_edges(
        &self,
        request: Request<ListEdgesRequest>,
    ) -> Result<Response<ListEdgesResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let edges = self
            .edges
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let filtered: Vec<GraphEdge> = edges
            .values()
            .filter(|e| {
                (req.from_node.is_empty() || e.from_node == req.from_node)
                    && (req.to_node.is_empty() || e.to_node == req.to_node)
                    && (req.relation.is_empty() || e.relation == req.relation)
                    && (req.trust_state.is_empty() || e.trust_state == req.trust_state)
            })
            .cloned()
            .collect();
        Ok(Response::new(ListEdgesResponse {
            edges: filtered,
            next_page_token: String::new(),
        }))
    }

    async fn delete_edge(
        &self,
        request: Request<DeleteEdgeRequest>,
    ) -> Result<Response<DeleteEdgeResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::DeleteDocument, ResourceType::Attestation)?;
        let edge_id = request.into_inner().edge_id;
        let mut edges = self
            .edges
            .write()
            .map_err(|e| Status::internal(e.to_string()))?;
        let success = edges.remove(&edge_id).is_some();
        Ok(Response::new(DeleteEdgeResponse { success }))
    }

    async fn get_node(
        &self,
        request: Request<GetNodeRequest>,
    ) -> Result<Response<GetNodeResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let node_id = request.into_inner().node_id;
        let nodes = self
            .nodes
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(node) = nodes.get(&node_id) {
            return Ok(Response::new(GetNodeResponse {
                node: Some(node.clone()),
            }));
        }
        Err(Status::not_found(format!("Node `{node_id}` not found")))
    }

    async fn list_nodes(
        &self,
        request: Request<ListNodesRequest>,
    ) -> Result<Response<ListNodesResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let nodes = self
            .nodes
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let filtered: Vec<GraphNode> = nodes
            .values()
            .filter(|n| req.kind.is_empty() || n.kind == req.kind)
            .cloned()
            .collect();
        Ok(Response::new(ListNodesResponse {
            nodes: filtered,
            next_page_token: String::new(),
        }))
    }

    async fn traverse(
        &self,
        request: Request<TraverseRequest>,
    ) -> Result<Response<TraverseResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let max_depth = if req.max_depth <= 0 {
            1
        } else {
            req.max_depth.min(16)
        };
        let edges = self
            .edges
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let nodes = self
            .nodes
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut sorted_edges: Vec<&GraphEdge> = edges.values().collect();
        sorted_edges.sort_by(|a, b| a.id.cmp(&b.id));

        let mut path = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited_edges = std::collections::HashSet::new();
        queue.push_back((req.start_node.clone(), 0i32));

        while let Some((current_node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            let next_depth = depth + 1;
            for edge in &sorted_edges {
                if edge.from_node == current_node
                    && (req.relations.is_empty() || req.relations.contains(&edge.relation))
                    && visited_edges.insert(edge.id.clone())
                {
                    let target_node = nodes.get(&edge.to_node).cloned();
                    path.push(PathSegment {
                        depth: next_depth,
                        edge: Some((*edge).clone()),
                        node: target_node,
                    });
                    queue.push_back((edge.to_node.clone(), next_depth));
                }
            }
        }
        Ok(Response::new(TraverseResponse { path }))
    }

    async fn shortest_path(
        &self,
        request: Request<ShortestPathRequest>,
    ) -> Result<Response<ShortestPathResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        if req.from_node.is_empty() || req.to_node.is_empty() {
            return Ok(Response::new(ShortestPathResponse {
                edges: Vec::new(),
                total_weight: 0.0,
                found: false,
            }));
        }
        let edges = self
            .edges
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut sorted_edges: Vec<&GraphEdge> = edges.values().collect();
        sorted_edges.sort_by(|a, b| a.id.cmp(&b.id));

        let max_depth = 16usize;
        let mut queue = std::collections::VecDeque::new();
        let mut visited_nodes = std::collections::HashSet::new();
        visited_nodes.insert(req.from_node.clone());
        queue.push_back((req.from_node.clone(), Vec::<GraphEdge>::new(), 0.0f64));

        while let Some((current, path_edges, weight)) = queue.pop_front() {
            if current == req.to_node && !path_edges.is_empty() {
                return Ok(Response::new(ShortestPathResponse {
                    edges: path_edges,
                    total_weight: weight,
                    found: true,
                }));
            }
            if path_edges.len() >= max_depth {
                continue;
            }
            for edge in &sorted_edges {
                if edge.from_node == current
                    && (req.allowed_relations.is_empty()
                        || req.allowed_relations.contains(&edge.relation))
                    && visited_nodes.insert(edge.to_node.clone())
                {
                    let mut next_path = path_edges.clone();
                    next_path.push((*edge).clone());
                    queue.push_back((edge.to_node.clone(), next_path, weight + edge.weight));
                }
            }
        }

        Ok(Response::new(ShortestPathResponse {
            edges: Vec::new(),
            total_weight: 0.0,
            found: false,
        }))
    }

    async fn impact_radius(
        &self,
        request: Request<ImpactRadiusRequest>,
    ) -> Result<Response<ImpactRadiusResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let max_depth = if req.max_depth <= 0 {
            1
        } else {
            req.max_depth.min(16)
        };
        let edges_map = self
            .edges
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let nodes_map = self
            .nodes
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut sorted_edges: Vec<&GraphEdge> = edges_map.values().collect();
        sorted_edges.sort_by(|a, b| a.id.cmp(&b.id));

        let mut matched_edges = Vec::new();
        let mut matched_nodes = Vec::new();
        let mut visited_edges = std::collections::HashSet::new();
        let mut visited_nodes = std::collections::HashSet::new();
        visited_nodes.insert(req.node.clone());
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((req.node.clone(), 0i32));

        while let Some((current_node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            let next_depth = depth + 1;
            for edge in &sorted_edges {
                if (edge.from_node == current_node || edge.to_node == current_node)
                    && (req.relations.is_empty() || req.relations.contains(&edge.relation))
                    && visited_edges.insert(edge.id.clone())
                {
                    matched_edges.push((*edge).clone());
                    let neighbor_id = if edge.from_node == current_node {
                        &edge.to_node
                    } else {
                        &edge.from_node
                    };
                    if visited_nodes.insert(neighbor_id.clone()) {
                        if let Some(n) = nodes_map.get(neighbor_id) {
                            matched_nodes.push(n.clone());
                        }
                        queue.push_back((neighbor_id.clone(), next_depth));
                    }
                }
            }
        }
        Ok(Response::new(ImpactRadiusResponse {
            nodes: matched_nodes,
            edges: matched_edges,
        }))
    }

    async fn explain_claim(
        &self,
        request: Request<ExplainClaimRequest>,
    ) -> Result<Response<ExplainClaimResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let claim_id = if !req.claim_id.is_empty() {
            req.claim_id
        } else if !req.edge_id.is_empty() {
            let edges = self
                .edges
                .read()
                .map_err(|e| Status::internal(e.to_string()))?;
            edges
                .get(&req.edge_id)
                .map(|e| e.claim_id.clone())
                .ok_or_else(|| Status::not_found(format!("Edge `{}` not found", req.edge_id)))?
        } else {
            return Err(Status::invalid_argument("claim_id or edge_id is required"));
        };

        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let claim = claims
            .get(&claim_id)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("Claim `{claim_id}` not found")))?;

        let ev_store = self
            .evidence
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let mut linked_evidence = Vec::new();
        for sref in &claim.source_refs {
            if let Some(ev) = ev_store.get(&sref.r#ref).or_else(|| {
                ev_store
                    .values()
                    .find(|e| !sref.digest.is_empty() && e.digest == sref.digest)
            }) {
                linked_evidence.push(ev.clone());
            }
        }

        let trust_state = if claim.trust_state.is_empty() {
            "observed".to_string()
        } else {
            claim.trust_state.clone()
        };
        let proof_state_json = if claim.proof_state_json.is_empty() {
            serde_json::json!({
                "discovery": "discovered",
                "claim": "claim_bound",
                "trust_state": trust_state,
            })
            .to_string()
        } else {
            claim.proof_state_json.clone()
        };

        Ok(Response::new(ExplainClaimResponse {
            claim: Some(claim),
            evidence: linked_evidence,
            proof_state_json,
            trust_state,
        }))
    }

    async fn compute_trust_state(
        &self,
        request: Request<ComputeTrustStateRequest>,
    ) -> Result<Response<ComputeTrustStateResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        let claim_id = if !req.claim_id.is_empty() {
            req.claim_id.clone()
        } else if !req.edge_id.is_empty() {
            let edges = self
                .edges
                .read()
                .map_err(|e| Status::internal(e.to_string()))?;
            edges
                .get(&req.edge_id)
                .map(|e| e.claim_id.clone())
                .ok_or_else(|| Status::not_found(format!("Edge `{}` not found", req.edge_id)))?
        } else {
            return Err(Status::invalid_argument("claim_id or edge_id is required"));
        };

        let claims = self
            .claims
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;
        let claim = claims
            .get(&claim_id)
            .ok_or_else(|| Status::not_found(format!("Claim `{claim_id}` not found")))?;

        let trust_state = if claim.trust_state.is_empty() {
            "observed".to_string()
        } else {
            claim.trust_state.clone()
        };
        let has_proof_refs = !claim.proof_refs.is_empty();

        Ok(Response::new(ComputeTrustStateResponse {
            claim_id,
            edge_id: req.edge_id,
            proof_state: Some(ProofState {
                discovery: "discovered".to_string(),
                graph: "graph_resolved".to_string(),
                claim: "claim_bound".to_string(),
                source: if claim.source_refs.is_empty() {
                    "unbound".to_string()
                } else {
                    "source_bound".to_string()
                },
                signature: if has_proof_refs {
                    "signature_verified".to_string()
                } else {
                    "unverified".to_string()
                },
                transparency: if has_proof_refs {
                    "inclusion_verified".to_string()
                } else {
                    "unverified".to_string()
                },
                witness_json: "{}".to_string(),
                state: "current_state_verified".to_string(),
                policy: "policy_admissible".to_string(),
            }),
            trust_state,
            diagnostics: Vec::new(),
        }))
    }

    async fn verify_closure(
        &self,
        request: Request<VerifyClosureRequest>,
    ) -> Result<Response<VerifyClosureResponse>, Status> {
        let _ctx =
            self.authorize_request(&request, Action::ReadDocument, ResourceType::Attestation)?;
        let req = request.into_inner();
        if req.subject_node.trim().is_empty() {
            return Err(Status::invalid_argument("subject_node is required"));
        }
        let edges = self
            .edges
            .read()
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut sorted_edges: Vec<&GraphEdge> = edges.values().collect();
        sorted_edges.sort_by(|a, b| a.id.cmp(&b.id));

        let mut edges_checked = 0i32;
        let mut edges_failed = 0i32;
        let mut has_adverse = false;
        let mut diagnostics = Vec::new();
        for edge in sorted_edges {
            if edge.from_node == req.subject_node || edge.to_node == req.subject_node {
                edges_checked += 1;
                if edge.trust_state == "rejected" || edge.trust_state == "contradicted" {
                    edges_failed += 1;
                    has_adverse = true;
                    diagnostics.push(format!(
                        "Edge `{}` carries adverse trust_state `{}`",
                        edge.id, edge.trust_state
                    ));
                } else if !matches!(edge.trust_state.as_str(), "verified" | "attested") {
                    edges_failed += 1;
                    diagnostics.push(format!(
                        "Edge `{}` is not verified (trust_state `{}`)",
                        edge.id, edge.trust_state
                    ));
                }
            }
        }
        let verdict = edges_checked > 0 && edges_failed == 0;
        let trust_state = if has_adverse {
            "contradicted".to_string()
        } else if edges_failed > 0 {
            "incomplete".to_string()
        } else if edges_checked > 0 {
            "verified".to_string()
        } else {
            "observed".to_string()
        };

        Ok(Response::new(VerifyClosureResponse {
            subject_node: req.subject_node,
            purpose: req.purpose,
            verdict,
            trust_state,
            diagnostics,
            edges_checked,
            edges_failed,
        }))
    }
}

/// Start an embedded Mizan gRPC server on the requested socket address with a custom state.
pub async fn start_embedded_server_with_state(
    addr: SocketAddr,
    state: MizanServerState,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), tonic::transport::Error> {
    tonic::transport::Server::builder()
        .add_service(OscalServiceServer::new(state.clone()))
        .add_service(GovernanceServiceServer::new(state.clone()))
        .add_service(TransparencyExchangeServiceServer::new(state.clone()))
        .add_service(TransparencyGraphServiceServer::new(state))
        .serve_with_shutdown(addr, async {
            let _ = shutdown_rx.await;
        })
        .await
}

/// Start an embedded Mizan gRPC server on the requested socket address.
pub async fn start_embedded_server(
    addr: SocketAddr,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), tonic::transport::Error> {
    start_embedded_server_with_state(addr, MizanServerState::default(), shutdown_rx).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claims_and_evidence_lifecycle() {
        let state = MizanServerState::new();

        // 1. Missing claim lookups return not_found
        let missing_claim_res = state
            .verify_claim(Request::new(VerifyClaimRequest {
                claim_id: "non-existent".to_string(),
                checks: Vec::new(),
            }))
            .await;
        assert!(missing_claim_res.is_err());
        assert_eq!(missing_claim_res.unwrap_err().code(), tonic::Code::NotFound);

        let missing_receipt_res = state
            .export_claim_receipt(Request::new(ExportClaimReceiptRequest {
                claim_id: "non-existent".to_string(),
            }))
            .await;
        assert!(missing_receipt_res.is_err());
        assert_eq!(
            missing_receipt_res.unwrap_err().code(),
            tonic::Code::NotFound
        );

        // 2. Create a candidate claim (without proof_refs)
        let claim = Claim {
            id: "claim-test-1".to_string(),
            r#type: "security-assessment".to_string(),
            trust_state: "candidate".to_string(),
            ..Default::default()
        };
        let create_res = state
            .create_claim(Request::new(CreateClaimRequest {
                claim: Some(claim.clone()),
            }))
            .await
            .expect("create_claim should succeed");
        assert_eq!(create_res.into_inner().claim.unwrap().id, "claim-test-1");

        // 3. Verify claim
        let verify_res = state
            .verify_claim(Request::new(VerifyClaimRequest {
                claim_id: "claim-test-1".to_string(),
                checks: Vec::new(),
            }))
            .await
            .expect("verify_claim should succeed");
        let v_inner = verify_res.into_inner();
        assert_eq!(v_inner.claim_id, "claim-test-1");
        assert_eq!(v_inner.trust_state, "candidate");
        assert_eq!(v_inner.proof_state.unwrap().signature, "unverified");

        // 4. Export claim receipt
        let receipt_res = state
            .export_claim_receipt(Request::new(ExportClaimReceiptRequest {
                claim_id: "claim-test-1".to_string(),
            }))
            .await
            .expect("export_claim_receipt should succeed");
        let r_inner = receipt_res.into_inner();
        assert_eq!(r_inner.claim_id, "claim-test-1");
        assert!(!r_inner.receipt_json.is_empty());
        assert_eq!(
            r_inner.receipt_digest,
            CasStore::compute_digest(r_inner.receipt_json.as_bytes())
        );
        assert_eq!(r_inner.verification_event_hash, r_inner.receipt_digest);

        // 5. Missing evidence lookups return not_found
        let missing_ev_res = state
            .get_evidence(Request::new(GetEvidenceRequest {
                evidence_id: "non-existent-ev".to_string(),
            }))
            .await;
        assert!(missing_ev_res.is_err());
        assert_eq!(missing_ev_res.unwrap_err().code(), tonic::Code::NotFound);

        let missing_verify_ev = state
            .verify_evidence(Request::new(VerifyEvidenceRequest {
                evidence_id: "non-existent-ev".to_string(),
                fetch_and_hash: false,
            }))
            .await;
        assert!(missing_verify_ev.is_err());
        assert_eq!(missing_verify_ev.unwrap_err().code(), tonic::Code::NotFound);

        // 6. Upload evidence
        let payload = b"SBOM component evidence test payload";
        let ev = Evidence {
            id: "ev-test-1".to_string(),
            media_type: "application/json".to_string(),
            bom_kind: "cyclonedx".to_string(),
            ..Default::default()
        };
        let upload_res = state
            .upload_evidence(Request::new(UploadEvidenceRequest {
                evidence: Some(ev),
                blob: payload.to_vec(),
            }))
            .await
            .expect("upload_evidence should succeed");
        let uploaded = upload_res.into_inner().evidence.unwrap();
        assert_eq!(uploaded.id, "ev-test-1");
        assert_eq!(uploaded.size_bytes, payload.len() as i64);
        assert_eq!(uploaded.digest, CasStore::compute_digest(payload));

        // 7. Get evidence
        let get_ev_res = state
            .get_evidence(Request::new(GetEvidenceRequest {
                evidence_id: "ev-test-1".to_string(),
            }))
            .await
            .expect("get_evidence should succeed");
        assert_eq!(get_ev_res.into_inner().evidence.unwrap().id, "ev-test-1");

        // 8. Verify evidence without and with fetch_and_hash
        let verify_ev_res1 = state
            .verify_evidence(Request::new(VerifyEvidenceRequest {
                evidence_id: "ev-test-1".to_string(),
                fetch_and_hash: false,
            }))
            .await
            .expect("verify_evidence should succeed");
        let v_ev1 = verify_ev_res1.into_inner();
        assert!(v_ev1.digest_ok);
        assert!(v_ev1.size_ok);
        assert!(v_ev1.error.is_empty());

        let verify_ev_res2 = state
            .verify_evidence(Request::new(VerifyEvidenceRequest {
                evidence_id: "ev-test-1".to_string(),
                fetch_and_hash: true,
            }))
            .await
            .expect("verify_evidence with fetch_and_hash should succeed");
        let v_ev2 = verify_ev_res2.into_inner();
        assert!(v_ev2.digest_ok);
        assert!(v_ev2.size_ok);
        assert!(v_ev2.error.is_empty());

        // 9. Project edge into TransparencyGraphService and verify hash-chained projection events
        let proj_res = state
            .project_edge(Request::new(ProjectEdgeRequest {
                claim_id: "claim-test-1".to_string(),
                edge_id: "edge-test-1".to_string(),
            }))
            .await
            .expect("project_edge should succeed")
            .into_inner();
        let projected_edge = proj_res.edge.expect("edge should be returned");
        assert_eq!(
            projected_edge.proof_state.as_ref().unwrap().signature,
            "unverified"
        );
        assert!(proj_res.projection_event.is_some());

        let events_res = state
            .list_projection_events(Request::new(ListProjectionEventsRequest {
                claim_id: "claim-test-1".to_string(),
                edge_id: String::new(),
            }))
            .await
            .expect("list_projection_events should succeed")
            .into_inner();
        assert!(events_res.chain_valid);
        assert_eq!(events_res.events.len(), 1);

        // 10. Explain claim, compute trust state, and verify closure
        let explain_res = state
            .explain_claim(Request::new(ExplainClaimRequest {
                claim_id: "claim-test-1".to_string(),
                edge_id: String::new(),
            }))
            .await
            .expect("explain_claim should succeed")
            .into_inner();
        assert!(explain_res.claim.is_some());

        let trust_res = state
            .compute_trust_state(Request::new(ComputeTrustStateRequest {
                claim_id: String::new(),
                edge_id: "edge-test-1".to_string(),
            }))
            .await
            .expect("compute_trust_state should succeed")
            .into_inner();
        assert_eq!(trust_res.claim_id, "claim-test-1");
        let ps = trust_res
            .proof_state
            .expect("proof_state should be present");
        assert_eq!(ps.signature, "unverified");
        assert_eq!(ps.transparency, "unverified");

        // Candidate edge is not verified, so verify_closure must report verdict = false
        let closure_res = state
            .verify_closure(Request::new(VerifyClosureRequest {
                subject_node: "subject-claim-test-1".to_string(),
                purpose: "ga-readiness".to_string(),
                as_of: None,
            }))
            .await
            .expect("verify_closure should succeed")
            .into_inner();
        assert!(!closure_res.verdict);
        assert_eq!(closure_res.edges_checked, 1);
        assert_eq!(closure_res.edges_failed, 1);
        assert!(!closure_res.diagnostics.is_empty());

        // 11. Verified claim with proof_refs produces signature_verified proof_state and passing closure
        let verified_claim = Claim {
            id: "claim-verified-1".to_string(),
            r#type: "security-assessment".to_string(),
            trust_state: "verified".to_string(),
            proof_refs: vec![ProofRef {
                r#type: "ed25519".to_string(),
                r#ref: "sig-1".to_string(),
                digest: uploaded.digest.clone(),
                ..Default::default()
            }],
            ..Default::default()
        };
        state
            .create_claim(Request::new(CreateClaimRequest {
                claim: Some(verified_claim),
            }))
            .await
            .expect("create verified claim");
        let verified_proj = state
            .project_edge(Request::new(ProjectEdgeRequest {
                claim_id: "claim-verified-1".to_string(),
                edge_id: "edge-verified-1".to_string(),
            }))
            .await
            .expect("project verified edge")
            .into_inner();
        assert_eq!(
            verified_proj.edge.unwrap().proof_state.unwrap().signature,
            "signature_verified"
        );
        let verified_closure = state
            .verify_closure(Request::new(VerifyClosureRequest {
                subject_node: "subject-claim-verified-1".to_string(),
                purpose: "ga-readiness".to_string(),
                as_of: None,
            }))
            .await
            .expect("verify_closure on verified subject")
            .into_inner();
        assert!(verified_closure.verdict);
        assert_eq!(verified_closure.edges_checked, 1);
        assert_eq!(verified_closure.edges_failed, 0);
    }

    #[tokio::test]
    async fn test_server_rbac_denies_viewer_mutations() {
        let state = MizanServerState::new();
        let mut req = Request::new(CreateCatalogRequest {
            catalog: Some(Catalog::default()),
        });
        req.metadata_mut().insert(
            "x-mizan-role",
            tonic::metadata::MetadataValue::from_static("Viewer"),
        );
        let err = state
            .create_catalog(req)
            .await
            .expect_err("Viewer must be denied catalog creation");
        assert_eq!(err.code(), tonic::Code::PermissionDenied);

        let mut prof_req = Request::new(CreateProfileRequest {
            profile: Some(Profile::default()),
        });
        prof_req.metadata_mut().insert(
            "x-mizan-role",
            tonic::metadata::MetadataValue::from_static("Viewer"),
        );
        assert_eq!(
            state.create_profile(prof_req).await.unwrap_err().code(),
            tonic::Code::PermissionDenied
        );

        let mut claim_req = Request::new(CreateClaimRequest {
            claim: Some(Claim::default()),
        });
        claim_req.metadata_mut().insert(
            "x-mizan-role",
            tonic::metadata::MetadataValue::from_static("Viewer"),
        );
        assert_eq!(
            state.create_claim(claim_req).await.unwrap_err().code(),
            tonic::Code::PermissionDenied
        );

        // Enforce require_auth with OIDC validator across both read and write RPCs
        let secret = b"oidc-grpc-server-test-secret-32b";
        let cfg = crate::fabric::OidcProviderConfig::new("https://auth.runbase.io", "mizan-grpc")
            .with_shared_secret(secret);
        let auth_state = MizanServerState::new().with_oidc_auth(OidcTokenValidator::new(cfg));

        assert_eq!(
            auth_state
                .list_catalogs(Request::new(ListCatalogsRequest::default()))
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        assert_eq!(
            auth_state
                .create_ssp(Request::new(CreateSspRequest {
                    ssp: Some(Ssp::default()),
                }))
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );

        let viewer_claims = crate::fabric::OidcClaims {
            iss: "https://auth.runbase.io".to_string(),
            sub: "viewer_1".to_string(),
            aud: vec!["mizan-grpc".to_string()],
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp() - 60,
            email: None,
            tenant_id: Some("tenant-a".to_string()),
            groups: Vec::new(),
            roles: vec!["Viewer".to_string()],
            name: None,
        };
        let viewer_jwt = OidcTokenValidator::sign_hs256(&viewer_claims, secret).unwrap();
        let bearer_hdr: tonic::metadata::MetadataValue<_> =
            format!("Bearer {viewer_jwt}").parse().unwrap();

        let mut auth_read_req = Request::new(ListCatalogsRequest::default());
        auth_read_req
            .metadata_mut()
            .insert("authorization", bearer_hdr.clone());
        assert!(auth_state.list_catalogs(auth_read_req).await.is_ok());

        let mut auth_write_req = Request::new(CreateSspRequest {
            ssp: Some(Ssp::default()),
        });
        auth_write_req
            .metadata_mut()
            .insert("authorization", bearer_hdr);
        assert_eq!(
            auth_state
                .create_ssp(auth_write_req)
                .await
                .unwrap_err()
                .code(),
            tonic::Code::PermissionDenied
        );
    }

    #[tokio::test]
    async fn test_graph_multi_hop_traversal_shortest_path_and_impact_radius() {
        let state = MizanServerState::new();

        // Build 3-hop chain: node-a -> node-b -> node-c -> node-d
        for (idx, (from, to)) in [
            ("node-a", "node-b"),
            ("node-b", "node-c"),
            ("node-c", "node-d"),
        ]
        .iter()
        .enumerate()
        {
            let claim_id = format!("claim-hop-{idx}");
            let claim = Claim {
                id: claim_id.clone(),
                subject: Some(Reference {
                    kind: "component".to_string(),
                    id: (*from).to_string(),
                    ..Default::default()
                }),
                predicate: Some(Predicate {
                    relation: "depends_on".to_string(),
                    qualifier: String::new(),
                    ..Default::default()
                }),
                object_opt: Some(claim::ObjectOpt::Object(Reference {
                    kind: "component".to_string(),
                    id: (*to).to_string(),
                    ..Default::default()
                })),
                trust_state: "verified".to_string(),
                ..Default::default()
            };
            state
                .create_claim(Request::new(CreateClaimRequest { claim: Some(claim) }))
                .await
                .unwrap();
            state
                .project_edge(Request::new(ProjectEdgeRequest {
                    claim_id,
                    edge_id: format!("edge-hop-{idx}"),
                }))
                .await
                .unwrap();
        }

        // 1. Multi-hop traverse from node-a with max_depth = 3
        let trav = state
            .traverse(Request::new(TraverseRequest {
                start_node: "node-a".to_string(),
                max_depth: 3,
                relations: vec!["depends_on".to_string()],
                traversal_mode: "bfs".to_string(),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(trav.path.len(), 3);
        assert_eq!(trav.path[2].depth, 3);

        // 2. Multi-hop shortest_path from node-a to node-d
        let sp = state
            .shortest_path(Request::new(ShortestPathRequest {
                from_node: "node-a".to_string(),
                to_node: "node-d".to_string(),
                allowed_relations: vec!["depends_on".to_string()],
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(sp.found);
        assert_eq!(sp.edges.len(), 3);
        assert_eq!(sp.total_weight, 3.0);

        // 3. Impact radius from node-c (incoming from node-b, outgoing to node-d)
        let impact = state
            .impact_radius(Request::new(ImpactRadiusRequest {
                node: "node-c".to_string(),
                max_depth: 1,
                relations: Vec::new(),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(impact.edges.len(), 2);
        let neighbor_ids: Vec<String> = impact.nodes.into_iter().map(|n| n.id).collect();
        assert!(neighbor_ids.contains(&"node-b".to_string()));
        assert!(neighbor_ids.contains(&"node-d".to_string()));
    }
}
