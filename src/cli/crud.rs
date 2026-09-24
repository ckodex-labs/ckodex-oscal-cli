#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, CrudAction},
    document::{OscalDocument, schema::DocumentKind},
    error::{AppError, Result},
    output,
    proto::oscal::services::v1::*,
    transport::CrudClient,
    valence::CrudResult,
};
use prost::Message;
use prost_reflect::{
    DescriptorPool, DeserializeOptions, DynamicMessage, FieldDescriptor, Kind, MessageDescriptor,
};
use serde_json::{Map, Value};

macro_rules! crud_call {
    ($config:expr, $client:expr, $value:expr, $req:ty, $resp:ty, $method:ident, $req_name:expr, $resp_name:expr, $op_name:expr) => {{
        let request: $req = decode_from_json($req_name, $value)?;
        let result: CrudResult<$resp> = $client.$method(request.clone()).await?;
        capture_if_enabled($config, $op_name, &request, &result.data)?;
        output::emit_crud_result($config.output, $resp_name, &result, $config.valence)
    }};
}

pub async fn run_model_crud(
    config: &AppConfig,
    kind: DocumentKind,
    action: CrudAction,
) -> Result<()> {
    match action {
        CrudAction::Create { file } => {
            let doc = OscalDocument::from_file(&file)?;
            validate_kind(&doc, kind)?;
            let value = model_request_value(&doc)?;
            let mut client = CrudClient::connect(config).await?;
            handle_create(config, kind, &value, &mut client).await
        }
        CrudAction::Update { uuid, file } => {
            validate_non_empty("uuid", &uuid)?;
            let doc = OscalDocument::from_file(&file)?;
            validate_kind(&doc, kind)?;
            let value = update_request_value(&doc, &uuid)?;
            let mut client = CrudClient::connect(config).await?;
            handle_update(config, kind, &value, &mut client).await
        }
        CrudAction::Get { uuid } => {
            validate_non_empty("uuid", &uuid)?;
            let value = uuid_request_value(&uuid);
            let mut client = CrudClient::connect(config).await?;
            handle_get(config, kind, &value, &mut client).await
        }
        CrudAction::List {
            page_size,
            page_token,
        } => {
            validate_positive("page-size", page_size)?;
            let value = list_request_value(page_size, &page_token);
            let mut client = CrudClient::connect(config).await?;
            handle_list(config, kind, &value, &mut client).await
        }
        CrudAction::Delete { uuid } => {
            validate_non_empty("uuid", &uuid)?;
            let value = uuid_request_value(&uuid);
            let mut client = CrudClient::connect(config).await?;
            handle_delete(config, kind, &value, &mut client).await
        }
    }
}

fn validate_positive(name: &str, value: i32) -> Result<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(AppError::InvalidArgument(format!(
            "{name} must be greater than zero"
        )))
    }
}

fn validate_kind(doc: &OscalDocument, kind: DocumentKind) -> Result<()> {
    if doc.kind != kind {
        return Err(AppError::InvalidArgument(format!(
            "expected {kind:?} document, found {:?}",
            doc.kind
        )));
    }
    Ok(())
}

fn model_request_value(doc: &OscalDocument) -> Result<Value> {
    let inner = doc
        .value
        .get(doc.kind.root_key())
        .cloned()
        .ok_or_else(|| AppError::Configuration("missing OSCAL model root".to_owned()))?;
    Ok(Value::Object(Map::from_iter([(
        model_field_name(doc.kind).to_owned(),
        inner,
    )])))
}

fn update_request_value(doc: &OscalDocument, uuid: &str) -> Result<Value> {
    let inner = doc
        .value
        .get(doc.kind.root_key())
        .cloned()
        .ok_or_else(|| AppError::Configuration("missing OSCAL model root".to_owned()))?;
    Ok(Value::Object(Map::from_iter([
        ("uuid".to_owned(), uuid_value(uuid)),
        (model_field_name(doc.kind).to_owned(), inner),
    ])))
}

fn uuid_request_value(uuid: &str) -> Value {
    Value::Object(Map::from_iter([("uuid".to_owned(), uuid_value(uuid))]))
}

fn list_request_value(page_size: i32, page_token: &str) -> Value {
    Value::Object(Map::from_iter([
        ("page_size".to_owned(), Value::Number(page_size.into())),
        (
            "page_token".to_owned(),
            Value::String(page_token.to_owned()),
        ),
        ("filter".to_owned(), Value::String(String::new())),
    ]))
}

fn uuid_value(uuid: &str) -> Value {
    Value::Object(Map::from_iter([(
        "value".to_owned(),
        Value::String(uuid.to_owned()),
    )]))
}

fn model_field_name(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Catalog => "catalog",
        DocumentKind::Profile => "profile",
        DocumentKind::ComponentDefinition => "component_definition",
        DocumentKind::Ssp => "ssp",
        DocumentKind::AssessmentPlan => "assessment_plan",
        DocumentKind::AssessmentResults => "assessment_results",
        DocumentKind::Poam => "poam",
        DocumentKind::Mapping => "mapping",
        DocumentKind::Complete => unreachable!(),
    }
}

async fn handle_create(
    config: &AppConfig,
    kind: DocumentKind,
    value: &Value,
    client: &mut CrudClient,
) -> Result<()> {
    match kind {
        DocumentKind::Catalog => crud_call!(
            config,
            client,
            value,
            CreateCatalogRequest,
            CreateCatalogResponse,
            create_catalog,
            "oscal.services.v1.CreateCatalogRequest",
            "oscal.services.v1.CreateCatalogResponse",
            "OscalService.CreateCatalog"
        ),
        DocumentKind::Profile => crud_call!(
            config,
            client,
            value,
            CreateProfileRequest,
            CreateProfileResponse,
            create_profile,
            "oscal.services.v1.CreateProfileRequest",
            "oscal.services.v1.CreateProfileResponse",
            "OscalService.CreateProfile"
        ),
        DocumentKind::ComponentDefinition => crud_call!(
            config,
            client,
            value,
            CreateComponentDefinitionRequest,
            CreateComponentDefinitionResponse,
            create_component_definition,
            "oscal.services.v1.CreateComponentDefinitionRequest",
            "oscal.services.v1.CreateComponentDefinitionResponse",
            "OscalService.CreateComponentDefinition"
        ),
        DocumentKind::Ssp => crud_call!(
            config,
            client,
            value,
            CreateSspRequest,
            CreateSspResponse,
            create_ssp,
            "oscal.services.v1.CreateSspRequest",
            "oscal.services.v1.CreateSspResponse",
            "OscalService.CreateSsp"
        ),
        DocumentKind::AssessmentPlan => crud_call!(
            config,
            client,
            value,
            CreateAssessmentPlanRequest,
            CreateAssessmentPlanResponse,
            create_assessment_plan,
            "oscal.services.v1.CreateAssessmentPlanRequest",
            "oscal.services.v1.CreateAssessmentPlanResponse",
            "OscalService.CreateAssessmentPlan"
        ),
        DocumentKind::AssessmentResults => crud_call!(
            config,
            client,
            value,
            CreateAssessmentResultsRequest,
            CreateAssessmentResultsResponse,
            create_assessment_results,
            "oscal.services.v1.CreateAssessmentResultsRequest",
            "oscal.services.v1.CreateAssessmentResultsResponse",
            "OscalService.CreateAssessmentResults"
        ),
        DocumentKind::Poam => crud_call!(
            config,
            client,
            value,
            CreatePoamRequest,
            CreatePoamResponse,
            create_poam,
            "oscal.services.v1.CreatePoamRequest",
            "oscal.services.v1.CreatePoamResponse",
            "OscalService.CreatePoam"
        ),
        DocumentKind::Mapping => crud_call!(
            config,
            client,
            value,
            CreateMappingRequest,
            CreateMappingResponse,
            create_mapping,
            "oscal.services.v1.CreateMappingRequest",
            "oscal.services.v1.CreateMappingResponse",
            "OscalService.CreateMapping"
        ),
        DocumentKind::Complete => Err(AppError::InvalidArgument(
            "complete documents are not crud-supported".to_owned(),
        )),
    }
}

async fn handle_update(
    config: &AppConfig,
    kind: DocumentKind,
    value: &Value,
    client: &mut CrudClient,
) -> Result<()> {
    match kind {
        DocumentKind::Catalog => crud_call!(
            config,
            client,
            value,
            UpdateCatalogRequest,
            UpdateCatalogResponse,
            update_catalog,
            "oscal.services.v1.UpdateCatalogRequest",
            "oscal.services.v1.UpdateCatalogResponse",
            "OscalService.UpdateCatalog"
        ),
        DocumentKind::Profile => crud_call!(
            config,
            client,
            value,
            UpdateProfileRequest,
            UpdateProfileResponse,
            update_profile,
            "oscal.services.v1.UpdateProfileRequest",
            "oscal.services.v1.UpdateProfileResponse",
            "OscalService.UpdateProfile"
        ),
        DocumentKind::ComponentDefinition => crud_call!(
            config,
            client,
            value,
            UpdateComponentDefinitionRequest,
            UpdateComponentDefinitionResponse,
            update_component_definition,
            "oscal.services.v1.UpdateComponentDefinitionRequest",
            "oscal.services.v1.UpdateComponentDefinitionResponse",
            "OscalService.UpdateComponentDefinition"
        ),
        DocumentKind::Ssp => crud_call!(
            config,
            client,
            value,
            UpdateSspRequest,
            UpdateSspResponse,
            update_ssp,
            "oscal.services.v1.UpdateSspRequest",
            "oscal.services.v1.UpdateSspResponse",
            "OscalService.UpdateSsp"
        ),
        DocumentKind::AssessmentPlan => crud_call!(
            config,
            client,
            value,
            UpdateAssessmentPlanRequest,
            UpdateAssessmentPlanResponse,
            update_assessment_plan,
            "oscal.services.v1.UpdateAssessmentPlanRequest",
            "oscal.services.v1.UpdateAssessmentPlanResponse",
            "OscalService.UpdateAssessmentPlan"
        ),
        DocumentKind::AssessmentResults => crud_call!(
            config,
            client,
            value,
            UpdateAssessmentResultsRequest,
            UpdateAssessmentResultsResponse,
            update_assessment_results,
            "oscal.services.v1.UpdateAssessmentResultsRequest",
            "oscal.services.v1.UpdateAssessmentResultsResponse",
            "OscalService.UpdateAssessmentResults"
        ),
        DocumentKind::Poam => crud_call!(
            config,
            client,
            value,
            UpdatePoamRequest,
            UpdatePoamResponse,
            update_poam,
            "oscal.services.v1.UpdatePoamRequest",
            "oscal.services.v1.UpdatePoamResponse",
            "OscalService.UpdatePoam"
        ),
        DocumentKind::Mapping => crud_call!(
            config,
            client,
            value,
            UpdateMappingRequest,
            UpdateMappingResponse,
            update_mapping,
            "oscal.services.v1.UpdateMappingRequest",
            "oscal.services.v1.UpdateMappingResponse",
            "OscalService.UpdateMapping"
        ),
        DocumentKind::Complete => Err(AppError::InvalidArgument(
            "complete documents are not crud-supported".to_owned(),
        )),
    }
}

async fn handle_get(
    config: &AppConfig,
    kind: DocumentKind,
    value: &Value,
    client: &mut CrudClient,
) -> Result<()> {
    match kind {
        DocumentKind::Catalog => crud_call!(
            config,
            client,
            value,
            GetCatalogRequest,
            GetCatalogResponse,
            get_catalog,
            "oscal.services.v1.GetCatalogRequest",
            "oscal.services.v1.GetCatalogResponse",
            "OscalService.GetCatalog"
        ),
        DocumentKind::Profile => crud_call!(
            config,
            client,
            value,
            GetProfileRequest,
            GetProfileResponse,
            get_profile,
            "oscal.services.v1.GetProfileRequest",
            "oscal.services.v1.GetProfileResponse",
            "OscalService.GetProfile"
        ),
        DocumentKind::ComponentDefinition => crud_call!(
            config,
            client,
            value,
            GetComponentDefinitionRequest,
            GetComponentDefinitionResponse,
            get_component_definition,
            "oscal.services.v1.GetComponentDefinitionRequest",
            "oscal.services.v1.GetComponentDefinitionResponse",
            "OscalService.GetComponentDefinition"
        ),
        DocumentKind::Ssp => crud_call!(
            config,
            client,
            value,
            GetSspRequest,
            GetSspResponse,
            get_ssp,
            "oscal.services.v1.GetSspRequest",
            "oscal.services.v1.GetSspResponse",
            "OscalService.GetSsp"
        ),
        DocumentKind::AssessmentPlan => crud_call!(
            config,
            client,
            value,
            GetAssessmentPlanRequest,
            GetAssessmentPlanResponse,
            get_assessment_plan,
            "oscal.services.v1.GetAssessmentPlanRequest",
            "oscal.services.v1.GetAssessmentPlanResponse",
            "OscalService.GetAssessmentPlan"
        ),
        DocumentKind::AssessmentResults => crud_call!(
            config,
            client,
            value,
            GetAssessmentResultsRequest,
            GetAssessmentResultsResponse,
            get_assessment_results,
            "oscal.services.v1.GetAssessmentResultsRequest",
            "oscal.services.v1.GetAssessmentResultsResponse",
            "OscalService.GetAssessmentResults"
        ),
        DocumentKind::Poam => crud_call!(
            config,
            client,
            value,
            GetPoamRequest,
            GetPoamResponse,
            get_poam,
            "oscal.services.v1.GetPoamRequest",
            "oscal.services.v1.GetPoamResponse",
            "OscalService.GetPoam"
        ),
        DocumentKind::Mapping => crud_call!(
            config,
            client,
            value,
            GetMappingRequest,
            GetMappingResponse,
            get_mapping,
            "oscal.services.v1.GetMappingRequest",
            "oscal.services.v1.GetMappingResponse",
            "OscalService.GetMapping"
        ),
        DocumentKind::Complete => Err(AppError::InvalidArgument(
            "complete documents are not crud-supported".to_owned(),
        )),
    }
}

async fn handle_list(
    config: &AppConfig,
    kind: DocumentKind,
    value: &Value,
    client: &mut CrudClient,
) -> Result<()> {
    match kind {
        DocumentKind::Catalog => crud_call!(
            config,
            client,
            value,
            ListCatalogsRequest,
            ListCatalogsResponse,
            list_catalogs,
            "oscal.services.v1.ListCatalogsRequest",
            "oscal.services.v1.ListCatalogsResponse",
            "OscalService.ListCatalogs"
        ),
        DocumentKind::Profile => crud_call!(
            config,
            client,
            value,
            ListProfilesRequest,
            ListProfilesResponse,
            list_profiles,
            "oscal.services.v1.ListProfilesRequest",
            "oscal.services.v1.ListProfilesResponse",
            "OscalService.ListProfiles"
        ),
        DocumentKind::ComponentDefinition => crud_call!(
            config,
            client,
            value,
            ListComponentDefinitionsRequest,
            ListComponentDefinitionsResponse,
            list_component_definitions,
            "oscal.services.v1.ListComponentDefinitionsRequest",
            "oscal.services.v1.ListComponentDefinitionsResponse",
            "OscalService.ListComponentDefinitions"
        ),
        DocumentKind::Ssp => crud_call!(
            config,
            client,
            value,
            ListSspsRequest,
            ListSspsResponse,
            list_ssps,
            "oscal.services.v1.ListSspsRequest",
            "oscal.services.v1.ListSspsResponse",
            "OscalService.ListSsps"
        ),
        DocumentKind::AssessmentPlan => crud_call!(
            config,
            client,
            value,
            ListAssessmentPlansRequest,
            ListAssessmentPlansResponse,
            list_assessment_plans,
            "oscal.services.v1.ListAssessmentPlansRequest",
            "oscal.services.v1.ListAssessmentPlansResponse",
            "OscalService.ListAssessmentPlans"
        ),
        DocumentKind::AssessmentResults => crud_call!(
            config,
            client,
            value,
            ListAssessmentResultsRequest,
            ListAssessmentResultsResponse,
            list_assessment_results,
            "oscal.services.v1.ListAssessmentResultsRequest",
            "oscal.services.v1.ListAssessmentResultsResponse",
            "OscalService.ListAssessmentResults"
        ),
        DocumentKind::Poam => crud_call!(
            config,
            client,
            value,
            ListPoamsRequest,
            ListPoamsResponse,
            list_poams,
            "oscal.services.v1.ListPoamsRequest",
            "oscal.services.v1.ListPoamsResponse",
            "OscalService.ListPoams"
        ),
        DocumentKind::Mapping => crud_call!(
            config,
            client,
            value,
            ListMappingsRequest,
            ListMappingsResponse,
            list_mappings,
            "oscal.services.v1.ListMappingsRequest",
            "oscal.services.v1.ListMappingsResponse",
            "OscalService.ListMappings"
        ),
        DocumentKind::Complete => Err(AppError::InvalidArgument(
            "complete documents are not crud-supported".to_owned(),
        )),
    }
}

async fn handle_delete(
    config: &AppConfig,
    kind: DocumentKind,
    value: &Value,
    client: &mut CrudClient,
) -> Result<()> {
    match kind {
        DocumentKind::Catalog => crud_call!(
            config,
            client,
            value,
            DeleteCatalogRequest,
            DeleteCatalogResponse,
            delete_catalog,
            "oscal.services.v1.DeleteCatalogRequest",
            "oscal.services.v1.DeleteCatalogResponse",
            "OscalService.DeleteCatalog"
        ),
        DocumentKind::Profile => crud_call!(
            config,
            client,
            value,
            DeleteProfileRequest,
            DeleteProfileResponse,
            delete_profile,
            "oscal.services.v1.DeleteProfileRequest",
            "oscal.services.v1.DeleteProfileResponse",
            "OscalService.DeleteProfile"
        ),
        DocumentKind::ComponentDefinition => crud_call!(
            config,
            client,
            value,
            DeleteComponentDefinitionRequest,
            DeleteComponentDefinitionResponse,
            delete_component_definition,
            "oscal.services.v1.DeleteComponentDefinitionRequest",
            "oscal.services.v1.DeleteComponentDefinitionResponse",
            "OscalService.DeleteComponentDefinition"
        ),
        DocumentKind::Ssp => crud_call!(
            config,
            client,
            value,
            DeleteSspRequest,
            DeleteSspResponse,
            delete_ssp,
            "oscal.services.v1.DeleteSspRequest",
            "oscal.services.v1.DeleteSspResponse",
            "OscalService.DeleteSsp"
        ),
        DocumentKind::AssessmentPlan => crud_call!(
            config,
            client,
            value,
            DeleteAssessmentPlanRequest,
            DeleteAssessmentPlanResponse,
            delete_assessment_plan,
            "oscal.services.v1.DeleteAssessmentPlanRequest",
            "oscal.services.v1.DeleteAssessmentPlanResponse",
            "OscalService.DeleteAssessmentPlan"
        ),
        DocumentKind::AssessmentResults => crud_call!(
            config,
            client,
            value,
            DeleteAssessmentResultsRequest,
            DeleteAssessmentResultsResponse,
            delete_assessment_results,
            "oscal.services.v1.DeleteAssessmentResultsRequest",
            "oscal.services.v1.DeleteAssessmentResultsResponse",
            "OscalService.DeleteAssessmentResults"
        ),
        DocumentKind::Poam => crud_call!(
            config,
            client,
            value,
            DeletePoamRequest,
            DeletePoamResponse,
            delete_poam,
            "oscal.services.v1.DeletePoamRequest",
            "oscal.services.v1.DeletePoamResponse",
            "OscalService.DeletePoam"
        ),
        DocumentKind::Mapping => crud_call!(
            config,
            client,
            value,
            DeleteMappingRequest,
            DeleteMappingResponse,
            delete_mapping,
            "oscal.services.v1.DeleteMappingRequest",
            "oscal.services.v1.DeleteMappingResponse",
            "OscalService.DeleteMapping"
        ),
        DocumentKind::Complete => Err(AppError::InvalidArgument(
            "complete documents are not crud-supported".to_owned(),
        )),
    }
}

pub(crate) fn decode_from_json<T: Message + Default>(full_name: &str, value: &Value) -> Result<T> {
    let pool = DescriptorPool::decode(crate::PROTO_DESCRIPTOR_SET)
        .map_err(|e| AppError::Descriptor(e.to_string()))?;
    let descriptor = pool.get_message_by_name(full_name).ok_or_else(|| {
        AppError::Descriptor(format!("message descriptor not found: {full_name}"))
    })?;
    let normalized = normalize_message(value, &descriptor);
    let json = serde_json::to_string(&normalized).map_err(AppError::Serialization)?;
    let dynamic = DynamicMessage::deserialize_with_options(
        descriptor.clone(),
        &mut serde_json::Deserializer::from_str(&json),
        &DeserializeOptions::new().deny_unknown_fields(false),
    )
    .map_err(|e| AppError::Descriptor(format!("{full_name}: {e}")))?;
    let bytes = dynamic.encode_to_vec();
    T::decode(bytes.as_slice()).map_err(AppError::Protobuf)
}

pub(crate) fn normalize_message(value: &Value, descriptor: &MessageDescriptor) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = Map::with_capacity(map.len());
            for (key, val) in map {
                let snake = key.replace('-', "_");
                let field = descriptor
                    .get_field_by_name(&snake)
                    .or_else(|| descriptor.get_field_by_json_name(&snake))
                    .or_else(|| descriptor.get_field_by_name(key))
                    .or_else(|| descriptor.get_field_by_json_name(key));
                let normalized = if let Some(field) = field {
                    normalize_field(val, &field)
                } else {
                    normalize_unknown(val)
                };
                out.insert(snake, normalized);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_unknown).collect()),
        other => other.clone(),
    }
}

fn normalize_field(value: &Value, field: &FieldDescriptor) -> Value {
    if field.is_list() {
        match value {
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| normalize_field_element(v, field))
                    .collect(),
            ),
            _ => Value::Array(vec![normalize_field_element(value, field)]),
        }
    } else {
        normalize_field_element(value, field)
    }
}

fn normalize_field_element(value: &Value, field: &FieldDescriptor) -> Value {
    if let Kind::Message(message_desc) = field.kind() {
        if is_wrapper_message(&message_desc) {
            if let Value::Object(_) = value {
                return normalize_message(value, &message_desc);
            }
            return Value::Object(Map::from_iter([("value".to_owned(), value.clone())]));
        }
        if let Value::Object(_) = value {
            return normalize_message(value, &message_desc);
        }
    }
    value.clone()
}

fn is_wrapper_message(descriptor: &MessageDescriptor) -> bool {
    !descriptor.full_name().starts_with("google.protobuf.")
        && descriptor.fields().count() == 1
        && descriptor.get_field_by_name("value").is_some()
}

fn normalize_unknown(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = Map::with_capacity(map.len());
            for (key, val) in map {
                out.insert(key.replace('-', "_"), normalize_unknown(val));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_unknown).collect()),
        other => other.clone(),
    }
}
