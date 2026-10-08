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
    fabric::datastore::FabricDataStore,
    fabric::tenant::TenantContext,
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
}

impl MizanServerState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tonic::async_trait]
impl OscalService for MizanServerState {
    async fn get_catalog(
        &self,
        request: Request<GetCatalogRequest>,
    ) -> Result<Response<GetCatalogResponse>, Status> {
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
        _request: Request<ListCatalogsRequest>,
    ) -> Result<Response<ListCatalogsResponse>, Status> {
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

        let ctx = TenantContext::default();
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

        let ctx = TenantContext::default();
        let _ = self.datastore.delete_document(&ctx, "catalogs", &uuid);

        Ok(Response::new(DeleteCatalogResponse { success: true }))
    }

    async fn get_profile(
        &self,
        request: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileResponse>, Status> {
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
        _request: Request<ListProfilesRequest>,
    ) -> Result<Response<ListProfilesResponse>, Status> {
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
        _request: Request<ListComponentDefinitionsRequest>,
    ) -> Result<Response<ListComponentDefinitionsResponse>, Status> {
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
        _request: Request<ListSspsRequest>,
    ) -> Result<Response<ListSspsResponse>, Status> {
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
        _request: Request<ListAssessmentPlansRequest>,
    ) -> Result<Response<ListAssessmentPlansResponse>, Status> {
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
        _request: Request<ListAssessmentResultsRequest>,
    ) -> Result<Response<ListAssessmentResultsResponse>, Status> {
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
        _request: Request<ListPoamsRequest>,
    ) -> Result<Response<ListPoamsResponse>, Status> {
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
        _request: Request<ListMappingsRequest>,
    ) -> Result<Response<ListMappingsResponse>, Status> {
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
        let entity = request.into_inner().entity;
        Ok(Response::new(CreateEntityResponse { entity }))
    }

    async fn get_entity(
        &self,
        _request: Request<GetEntityRequest>,
    ) -> Result<Response<GetEntityResponse>, Status> {
        Err(Status::not_found("Entity not found"))
    }

    async fn update_entity(
        &self,
        request: Request<UpdateEntityRequest>,
    ) -> Result<Response<UpdateEntityResponse>, Status> {
        let entity = request.into_inner().entity;
        Ok(Response::new(UpdateEntityResponse { entity }))
    }

    async fn list_entities(
        &self,
        _request: Request<ListEntitiesRequest>,
    ) -> Result<Response<ListEntitiesResponse>, Status> {
        Ok(Response::new(ListEntitiesResponse {
            entities: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn create_snapshot(
        &self,
        request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
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
        let _name = request.into_inner().name;
        Ok(Response::new(GetSnapshotResponse {
            entities: Vec::new(),
        }))
    }

    async fn list_snapshots(
        &self,
        _request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        Ok(Response::new(ListSnapshotsResponse { names: Vec::new() }))
    }

    async fn create_release(
        &self,
        request: Request<CreateReleaseRequest>,
    ) -> Result<Response<CreateReleaseResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(CreateReleaseResponse {
            name: req.name,
            snapshot_name: req.snapshot_name,
        }))
    }

    async fn list_releases(
        &self,
        _request: Request<ListReleasesRequest>,
    ) -> Result<Response<ListReleasesResponse>, Status> {
        Ok(Response::new(ListReleasesResponse { names: Vec::new() }))
    }

    async fn ingest_requirements(
        &self,
        _request: Request<IngestRequirementsRequest>,
    ) -> Result<Response<IngestRequirementsResponse>, Status> {
        Err(Status::unimplemented("ingest_requirements not implemented"))
    }

    async fn semantic_search(
        &self,
        _request: Request<SemanticSearchRequest>,
    ) -> Result<Response<SemanticSearchResponse>, Status> {
        Ok(Response::new(SemanticSearchResponse {
            results: Vec::new(),
        }))
    }

    async fn generate_catalog(
        &self,
        _request: Request<GenerateCatalogRequest>,
    ) -> Result<Response<GenerateCatalogResponse>, Status> {
        Err(Status::unimplemented("generate_catalog not implemented"))
    }

    async fn generate_profile(
        &self,
        _request: Request<GenerateProfileRequest>,
    ) -> Result<Response<GenerateProfileResponse>, Status> {
        Err(Status::unimplemented("generate_profile not implemented"))
    }

    async fn generate_mappings(
        &self,
        _request: Request<GenerateMappingsRequest>,
    ) -> Result<Response<GenerateMappingsResponse>, Status> {
        Err(Status::unimplemented("generate_mappings not implemented"))
    }

    async fn generate_ssp(
        &self,
        _request: Request<GenerateSspRequest>,
    ) -> Result<Response<GenerateSspResponse>, Status> {
        Err(Status::unimplemented("generate_ssp not implemented"))
    }

    async fn generate_component_definition(
        &self,
        _request: Request<GenerateComponentDefinitionRequest>,
    ) -> Result<Response<GenerateComponentDefinitionResponse>, Status> {
        Err(Status::unimplemented(
            "generate_component_definition not implemented",
        ))
    }

    async fn generate_assessment_plan(
        &self,
        _request: Request<GenerateAssessmentPlanRequest>,
    ) -> Result<Response<GenerateAssessmentPlanResponse>, Status> {
        Err(Status::unimplemented(
            "generate_assessment_plan not implemented",
        ))
    }

    async fn generate_poam(
        &self,
        _request: Request<GeneratePoamRequest>,
    ) -> Result<Response<GeneratePoamResponse>, Status> {
        Err(Status::unimplemented("generate_poam not implemented"))
    }

    async fn generate_assessment_results(
        &self,
        _request: Request<GenerateAssessmentResultsRequest>,
    ) -> Result<Response<GenerateAssessmentResultsResponse>, Status> {
        Err(Status::unimplemented(
            "generate_assessment_results not implemented",
        ))
    }

    async fn bulk_ingest_frameworks(
        &self,
        _request: Request<BulkIngestFrameworksRequest>,
    ) -> Result<Response<BulkIngestFrameworksResponse>, Status> {
        Err(Status::unimplemented(
            "bulk_ingest_frameworks not implemented",
        ))
    }

    async fn list_frameworks(
        &self,
        _request: Request<ListFrameworksRequest>,
    ) -> Result<Response<ListFrameworksResponse>, Status> {
        Ok(Response::new(ListFrameworksResponse {
            frameworks: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn get_framework(
        &self,
        _request: Request<GetFrameworkRequest>,
    ) -> Result<Response<GetFrameworkResponse>, Status> {
        Err(Status::not_found("Framework not found"))
    }

    async fn generate_cross_framework_mappings(
        &self,
        _request: Request<GenerateCrossFrameworkMappingsRequest>,
    ) -> Result<Response<GenerateCrossFrameworkMappingsResponse>, Status> {
        Err(Status::unimplemented(
            "generate_cross_framework_mappings not implemented",
        ))
    }

    async fn propose(
        &self,
        _request: Request<ProposeRequest>,
    ) -> Result<Response<ProposeResponse>, Status> {
        Err(Status::unimplemented("propose not implemented"))
    }

    async fn list_conflicts(
        &self,
        _request: Request<ListConflictsRequest>,
    ) -> Result<Response<ListConflictsResponse>, Status> {
        Ok(Response::new(ListConflictsResponse {
            conflicts: Vec::new(),
        }))
    }

    async fn resolve_conflict(
        &self,
        _request: Request<ResolveConflictRequest>,
    ) -> Result<Response<ResolveConflictResponse>, Status> {
        Err(Status::unimplemented("resolve_conflict not implemented"))
    }

    async fn publish_release(
        &self,
        _request: Request<PublishReleaseRequest>,
    ) -> Result<Response<PublishReleaseResponse>, Status> {
        Err(Status::unimplemented("publish_release not implemented"))
    }

    async fn propose_mapping_update(
        &self,
        _request: Request<ProposeMappingUpdateRequest>,
    ) -> Result<Response<ProposeMappingUpdateResponse>, Status> {
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
        _request: Request<ListClaimsRequest>,
    ) -> Result<Response<ListClaimsResponse>, Status> {
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

        Ok(Response::new(VerifyClaimResponse {
            claim_id,
            proof_state: None,
            trust_state,
            diagnostics: Vec::new(),
        }))
    }

    async fn list_verification_events(
        &self,
        _request: Request<ListVerificationEventsRequest>,
    ) -> Result<Response<ListVerificationEventsResponse>, Status> {
        Ok(Response::new(ListVerificationEventsResponse {
            events: Vec::new(),
            chain_valid: true,
        }))
    }

    async fn export_claim_receipt(
        &self,
        request: Request<ExportClaimReceiptRequest>,
    ) -> Result<Response<ExportClaimReceiptResponse>, Status> {
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
        _request: Request<PreflightImportRequest>,
    ) -> Result<Response<PreflightImportResponse>, Status> {
        Err(Status::unimplemented("preflight_import not implemented"))
    }

    async fn import_batch(
        &self,
        _request: Request<ImportBatchRequest>,
    ) -> Result<Response<ImportBatchResponse>, Status> {
        Err(Status::unimplemented("import_batch not implemented"))
    }

    async fn upload_evidence(
        &self,
        request: Request<UploadEvidenceRequest>,
    ) -> Result<Response<UploadEvidenceResponse>, Status> {
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
        _request: Request<FetchExternalEvidenceRequest>,
    ) -> Result<Response<FetchExternalEvidenceResponse>, Status> {
        Err(Status::unimplemented(
            "fetch_external_evidence not implemented",
        ))
    }

    async fn list_fetch_events(
        &self,
        _request: Request<ListFetchEventsRequest>,
    ) -> Result<Response<ListFetchEventsResponse>, Status> {
        Ok(Response::new(ListFetchEventsResponse {
            events: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn sync_claims(
        &self,
        _request: Request<SyncClaimsRequest>,
    ) -> Result<Response<SyncClaimsResponse>, Status> {
        Err(Status::unimplemented("sync_claims not implemented"))
    }
}

#[tonic::async_trait]
impl TransparencyGraphService for MizanServerState {
    async fn project_edge(
        &self,
        _request: Request<ProjectEdgeRequest>,
    ) -> Result<Response<ProjectEdgeResponse>, Status> {
        Err(Status::unimplemented("project_edge not implemented"))
    }

    async fn list_projection_events(
        &self,
        _request: Request<ListProjectionEventsRequest>,
    ) -> Result<Response<ListProjectionEventsResponse>, Status> {
        Ok(Response::new(ListProjectionEventsResponse {
            events: Vec::new(),
            chain_valid: true,
        }))
    }

    async fn get_edge(
        &self,
        _request: Request<GetEdgeRequest>,
    ) -> Result<Response<GetEdgeResponse>, Status> {
        Err(Status::not_found("Edge not found"))
    }

    async fn list_edges(
        &self,
        _request: Request<ListEdgesRequest>,
    ) -> Result<Response<ListEdgesResponse>, Status> {
        Ok(Response::new(ListEdgesResponse {
            edges: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn delete_edge(
        &self,
        _request: Request<DeleteEdgeRequest>,
    ) -> Result<Response<DeleteEdgeResponse>, Status> {
        Ok(Response::new(DeleteEdgeResponse { success: true }))
    }

    async fn get_node(
        &self,
        _request: Request<GetNodeRequest>,
    ) -> Result<Response<GetNodeResponse>, Status> {
        Err(Status::not_found("Node not found"))
    }

    async fn list_nodes(
        &self,
        _request: Request<ListNodesRequest>,
    ) -> Result<Response<ListNodesResponse>, Status> {
        Ok(Response::new(ListNodesResponse {
            nodes: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn traverse(
        &self,
        _request: Request<TraverseRequest>,
    ) -> Result<Response<TraverseResponse>, Status> {
        Ok(Response::new(TraverseResponse { path: Vec::new() }))
    }

    async fn shortest_path(
        &self,
        _request: Request<ShortestPathRequest>,
    ) -> Result<Response<ShortestPathResponse>, Status> {
        Ok(Response::new(ShortestPathResponse {
            edges: Vec::new(),
            total_weight: 0.0,
            found: false,
        }))
    }

    async fn impact_radius(
        &self,
        _request: Request<ImpactRadiusRequest>,
    ) -> Result<Response<ImpactRadiusResponse>, Status> {
        Ok(Response::new(ImpactRadiusResponse {
            nodes: Vec::new(),
            edges: Vec::new(),
        }))
    }

    async fn explain_claim(
        &self,
        _request: Request<ExplainClaimRequest>,
    ) -> Result<Response<ExplainClaimResponse>, Status> {
        Err(Status::unimplemented("explain_claim not implemented"))
    }

    async fn compute_trust_state(
        &self,
        _request: Request<ComputeTrustStateRequest>,
    ) -> Result<Response<ComputeTrustStateResponse>, Status> {
        Err(Status::unimplemented("compute_trust_state not implemented"))
    }

    async fn verify_closure(
        &self,
        _request: Request<VerifyClosureRequest>,
    ) -> Result<Response<VerifyClosureResponse>, Status> {
        Err(Status::unimplemented("verify_closure not implemented"))
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

        // 2. Create a claim
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
    }
}
