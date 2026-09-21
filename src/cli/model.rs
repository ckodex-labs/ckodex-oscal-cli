#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) async fn run_model(config: &AppConfig, action: ModelAction) -> Result<()> {
    if let ModelAction::List { page_size, .. } = &action {
        validate_positive("page-size", *page_size)?;
    }
    let mut client = ReadOnlyClient::connect(config).await?;
    match action {
        ModelAction::List {
            model,
            page_size,
            filter,
            page_token,
        } => match model {
            ModelKind::Catalog => {
                let request = ListCatalogsRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_catalogs(request.clone()).await?;
                capture_if_enabled(config, "OscalService.ListCatalogs", &request, &response)?;
                let rows = response
                    .catalogs
                    .iter()
                    .map(|item| {
                        resource_row(
                            "catalog",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.controls.len() + item.groups.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListCatalogsResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::Profile => {
                let request = ListProfilesRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_profiles(request.clone()).await?;
                capture_if_enabled(config, "OscalService.ListProfiles", &request, &response)?;
                let rows = response
                    .profiles
                    .iter()
                    .map(|item| {
                        resource_row(
                            "profile",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.imports.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListProfilesResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::ComponentDefinition => {
                let request = ListComponentDefinitionsRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_component_definitions(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "OscalService.ListComponentDefinitions",
                    &request,
                    &response,
                )?;
                let rows = response
                    .component_definitions
                    .iter()
                    .map(|item| {
                        resource_row(
                            "component-definition",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.components.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListComponentDefinitionsResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::Ssp => {
                let request = ListSspsRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_ssps(request.clone()).await?;
                capture_if_enabled(config, "OscalService.ListSsps", &request, &response)?;
                let rows = response
                    .ssps
                    .iter()
                    .map(|item| {
                        resource_row(
                            "ssp",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.control_implementation
                                .as_ref()
                                .map_or(0, |v| v.implemented_requirements.len()),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListSspsResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::AssessmentPlan => {
                let request = ListAssessmentPlansRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_assessment_plans(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "OscalService.ListAssessmentPlans",
                    &request,
                    &response,
                )?;
                let rows = response
                    .assessment_plans
                    .iter()
                    .map(|item| {
                        resource_row(
                            "assessment-plan",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.tasks.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListAssessmentPlansResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::AssessmentResults => {
                let request = ListAssessmentResultsRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_assessment_results(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "OscalService.ListAssessmentResults",
                    &request,
                    &response,
                )?;
                let rows = response
                    .assessment_results_list
                    .iter()
                    .map(|item| {
                        resource_row(
                            "assessment-results",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.results.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListAssessmentResultsResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::Poam => {
                let request = ListPoamsRequest {
                    page_size,
                    page_token: page_token.clone(),
                    filter,
                };
                let response = client.list_poams(request.clone()).await?;
                capture_if_enabled(config, "OscalService.ListPoams", &request, &response)?;
                let rows = response
                    .poams
                    .iter()
                    .map(|item| {
                        resource_row(
                            "poam",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            item.poam_items.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListPoamsResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
            ModelKind::Mapping => {
                let request = ListMappingsRequest {
                    page_size,
                    page_token,
                    filter,
                };
                let response = client.list_mappings(request.clone()).await?;
                capture_if_enabled(config, "OscalService.ListMappings", &request, &response)?;
                let rows = response
                    .mappings
                    .iter()
                    .map(|item| {
                        let count = mapping_count(item);
                        resource_row(
                            "mapping",
                            item.uuid
                                .as_ref()
                                .map_or_else(String::new, |v| v.value.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.title.clone()),
                            item.metadata
                                .as_ref()
                                .map_or_else(String::new, |v| v.version.clone()),
                            count,
                        )
                    })
                    .collect::<Vec<_>>();
                emit_collection(
                    config.output,
                    "oscal.services.v1.ListMappingsResponse",
                    &response,
                    rows,
                    &response.next_page_token,
                )?;
            }
        },
        ModelAction::Get { model, uuid: value } => {
            let key = uuid(value);
            match model {
                ModelKind::Catalog => {
                    get_model(
                        config,
                        "OscalService.GetCatalog",
                        "oscal.services.v1.GetCatalogResponse",
                        GetCatalogRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_catalog(GetCatalogRequest { uuid: Some(key) }),
                    )
                    .await?
                }
                ModelKind::Profile => {
                    get_model(
                        config,
                        "OscalService.GetProfile",
                        "oscal.services.v1.GetProfileResponse",
                        GetProfileRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_profile(GetProfileRequest { uuid: Some(key) }),
                    )
                    .await?
                }
                ModelKind::ComponentDefinition => {
                    get_model(
                        config,
                        "OscalService.GetComponentDefinition",
                        "oscal.services.v1.GetComponentDefinitionResponse",
                        GetComponentDefinitionRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_component_definition(GetComponentDefinitionRequest {
                            uuid: Some(key),
                        }),
                    )
                    .await?
                }
                ModelKind::Ssp => {
                    get_model(
                        config,
                        "OscalService.GetSsp",
                        "oscal.services.v1.GetSspResponse",
                        GetSspRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_ssp(GetSspRequest { uuid: Some(key) }),
                    )
                    .await?
                }
                ModelKind::AssessmentPlan => {
                    get_model(
                        config,
                        "OscalService.GetAssessmentPlan",
                        "oscal.services.v1.GetAssessmentPlanResponse",
                        GetAssessmentPlanRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_assessment_plan(GetAssessmentPlanRequest { uuid: Some(key) }),
                    )
                    .await?
                }
                ModelKind::AssessmentResults => {
                    get_model(
                        config,
                        "OscalService.GetAssessmentResults",
                        "oscal.services.v1.GetAssessmentResultsResponse",
                        GetAssessmentResultsRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_assessment_results(GetAssessmentResultsRequest {
                            uuid: Some(key),
                        }),
                    )
                    .await?
                }
                ModelKind::Poam => {
                    get_model(
                        config,
                        "OscalService.GetPoam",
                        "oscal.services.v1.GetPoamResponse",
                        GetPoamRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_poam(GetPoamRequest { uuid: Some(key) }),
                    )
                    .await?
                }
                ModelKind::Mapping => {
                    get_model(
                        config,
                        "OscalService.GetMapping",
                        "oscal.services.v1.GetMappingResponse",
                        GetMappingRequest {
                            uuid: Some(key.clone()),
                        },
                        client.get_mapping(GetMappingRequest { uuid: Some(key) }),
                    )
                    .await?
                }
            }
        }
    }
    Ok(())
}
