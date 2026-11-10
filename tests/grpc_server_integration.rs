use std::{net::TcpListener, time::Duration};

use mizan_oscal::{
    config::AppConfig,
    proto::oscal::{
        catalog::v1::Catalog,
        common::v1::{Metadata, Uuid},
        services::v1::{CreateCatalogRequest, GetCatalogRequest, ListCatalogsRequest},
    },
    transport::{CrudClient, start_embedded_server},
};

#[tokio::test]
async fn test_embedded_grpc_server_lifecycle_and_crud() {
    // 1. Allocate an ephemeral local port
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);

    // 2. Spawn the embedded server
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server_task = tokio::spawn(async move {
        start_embedded_server(addr, shutdown_rx)
            .await
            .expect("embedded gRPC server run cleanly");
    });

    // 3. Connect CrudClient
    let config = AppConfig {
        endpoint: format!("http://127.0.0.1:{}", addr.port()),
        ..Default::default()
    };

    let mut client = None;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if let Ok(c) = CrudClient::connect(&config).await {
            client = Some(c);
            break;
        }
    }
    let mut client = client.expect("CrudClient connected to embedded server");

    // 4. Verify initial catalog list is empty
    let initial_list = client
        .list_catalogs(ListCatalogsRequest::default())
        .await
        .expect("list catalogs");
    assert_eq!(initial_list.data.catalogs.len(), 0);

    // 5. Create a new catalog
    let catalog_uuid = "cat-nist-800-53-r5".to_string();
    let new_catalog = Catalog {
        uuid: Some(Uuid {
            value: catalog_uuid.clone(),
        }),
        metadata: Some(Metadata {
            title: "NIST SP 800-53 Rev 5 Core Catalog".to_string(),
            version: "5.1.0".to_string(),
            published: None,
            last_modified: None,
            remarks: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let create_resp = client
        .create_catalog(CreateCatalogRequest {
            catalog: Some(new_catalog),
        })
        .await
        .expect("create catalog");
    assert!(create_resp.data.catalog.is_some());

    // 6. Get the created catalog
    let get_resp = client
        .get_catalog(GetCatalogRequest {
            uuid: Some(Uuid {
                value: catalog_uuid.clone(),
            }),
        })
        .await
        .expect("get catalog");
    let fetched = get_resp.data.catalog.expect("catalog returned");
    assert_eq!(
        fetched.uuid.as_ref().map(|u| u.value.as_str()),
        Some("cat-nist-800-53-r5")
    );
    assert_eq!(
        fetched.metadata.as_ref().map(|m| m.title.as_str()),
        Some("NIST SP 800-53 Rev 5 Core Catalog")
    );

    // 7. Verify list now contains the catalog
    let post_list = client
        .list_catalogs(ListCatalogsRequest::default())
        .await
        .expect("list catalogs after create");
    assert_eq!(post_list.data.catalogs.len(), 1);

    // 8. Gracefully shut down embedded server
    let _ = shutdown_tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(2), server_task)
        .await
        .expect("server task terminated cleanly");
}

#[tokio::test]
async fn test_multi_tenant_isolation_across_catalogs_and_resources() {
    use mizan_oscal::{
        proto::oscal::{
            poam::v1::PlanOfActionAndMilestones,
            profile::v1::Profile,
            services::v1::{
                Claim, CreateClaimRequest, CreatePoamRequest, CreateProfileRequest,
                CreateSspRequest, DeleteCatalogRequest, DeletePoamRequest, DeleteProfileRequest,
                DeleteSspRequest, GetClaimRequest, GetPoamRequest, GetProfileRequest,
                GetSspRequest, ListClaimsRequest, ListPoamsRequest, ListProfilesRequest,
                ListSspsRequest, Predicate, ProjectEdgeRequest, Reference, TraverseRequest,
                UpdateCatalogRequest, UpdatePoamRequest, UpdateProfileRequest, UpdateSspRequest,
                claim, oscal_service_client::OscalServiceClient,
                transparency_exchange_service_client::TransparencyExchangeServiceClient,
                transparency_graph_service_client::TransparencyGraphServiceClient,
            },
            ssp::v1::SystemSecurityPlan,
        },
        transport::{MizanServerState, start_embedded_server_with_incoming},
    };
    use tonic::{Request, metadata::MetadataValue, transport::Endpoint};

    let std_listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    std_listener.set_nonblocking(true).expect("nonblocking");
    let addr = std_listener.local_addr().expect("local addr");
    let incoming = tonic::transport::server::TcpIncoming::from(
        tokio::net::TcpListener::from_std(std_listener).expect("tokio listener"),
    );

    let state = MizanServerState::new();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server_state = state.clone();
    let server_task = tokio::spawn(async move {
        start_embedded_server_with_incoming(incoming, server_state, shutdown_rx)
            .await
            .expect("embedded server");
    });

    let channel = Endpoint::from_shared(format!("http://127.0.0.1:{}", addr.port()))
        .expect("valid endpoint")
        .connect()
        .await
        .expect("connect channel");

    let mut oscal_client = OscalServiceClient::new(channel.clone());
    let mut exchange_client = TransparencyExchangeServiceClient::new(channel.clone());
    let mut graph_client = TransparencyGraphServiceClient::new(channel);

    fn as_tenant_a<T>(mut req: Request<T>) -> Request<T> {
        req.metadata_mut()
            .insert("x-mizan-tenant", MetadataValue::from_static("tenant-a"));
        req
    }
    fn as_tenant_b<T>(mut req: Request<T>) -> Request<T> {
        req.metadata_mut()
            .insert("x-mizan-tenant", MetadataValue::from_static("tenant-b"));
        req
    }

    // 1. Tenant A creates a catalog, profile, SSP, POA&M, claim, and graph edge
    let cat_id = "cat-shared-id".to_string();
    oscal_client
        .create_catalog(as_tenant_a(Request::new(CreateCatalogRequest {
            catalog: Some(Catalog {
                uuid: Some(Uuid {
                    value: cat_id.clone(),
                }),
                metadata: Some(Metadata {
                    title: "Tenant A Secret Catalog".to_string(),
                    version: "1.0.0".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        })))
        .await
        .expect("Tenant A create catalog");

    let prof_id = "prof-a-1".to_string();
    oscal_client
        .create_profile(as_tenant_a(Request::new(CreateProfileRequest {
            profile: Some(Profile {
                uuid: Some(Uuid {
                    value: prof_id.clone(),
                }),
                ..Default::default()
            }),
        })))
        .await
        .expect("Tenant A create profile");

    let ssp_id = "ssp-a-1".to_string();
    oscal_client
        .create_ssp(as_tenant_a(Request::new(CreateSspRequest {
            ssp: Some(SystemSecurityPlan {
                uuid: Some(Uuid {
                    value: ssp_id.clone(),
                }),
                ..Default::default()
            }),
        })))
        .await
        .expect("Tenant A create ssp");

    let poam_id = "poam-a-1".to_string();
    oscal_client
        .create_poam(as_tenant_a(Request::new(CreatePoamRequest {
            poam: Some(PlanOfActionAndMilestones {
                uuid: Some(Uuid {
                    value: poam_id.clone(),
                }),
                ..Default::default()
            }),
        })))
        .await
        .expect("Tenant A create poam");

    exchange_client
        .create_claim(as_tenant_a(Request::new(CreateClaimRequest {
            claim: Some(Claim {
                id: "claim-a-1".to_string(),
                subject: Some(Reference {
                    kind: "component".to_string(),
                    id: "node-a-1".to_string(),
                    ..Default::default()
                }),
                predicate: Some(Predicate {
                    relation: "depends_on".to_string(),
                    ..Default::default()
                }),
                object_opt: Some(claim::ObjectOpt::Object(Reference {
                    kind: "component".to_string(),
                    id: "node-a-2".to_string(),
                    ..Default::default()
                })),
                trust_state: "verified".to_string(),
                ..Default::default()
            }),
        })))
        .await
        .expect("Tenant A create claim");

    graph_client
        .project_edge(as_tenant_a(Request::new(ProjectEdgeRequest {
            claim_id: "claim-a-1".to_string(),
            edge_id: "edge-a-1".to_string(),
        })))
        .await
        .expect("Tenant A project edge");

    // 2. Tenant B cannot read, list, update, traverse, or delete Tenant A's resources
    assert_eq!(
        oscal_client
            .get_catalog(as_tenant_b(Request::new(GetCatalogRequest {
                uuid: Some(Uuid {
                    value: cat_id.clone(),
                }),
            })))
            .await
            .expect_err("Tenant B must not read Tenant A catalog")
            .code(),
        tonic::Code::NotFound
    );
    assert!(
        oscal_client
            .list_catalogs(as_tenant_b(Request::new(ListCatalogsRequest::default())))
            .await
            .expect("Tenant B list catalogs")
            .into_inner()
            .catalogs
            .is_empty()
    );
    assert_eq!(
        oscal_client
            .update_catalog(as_tenant_b(Request::new(UpdateCatalogRequest {
                uuid: Some(Uuid {
                    value: cat_id.clone(),
                }),
                catalog: Some(Catalog {
                    uuid: Some(Uuid {
                        value: cat_id.clone(),
                    }),
                    ..Default::default()
                }),
            })))
            .await
            .expect_err("Tenant B must not update Tenant A catalog")
            .code(),
        tonic::Code::NotFound
    );
    assert_eq!(
        oscal_client
            .delete_catalog(as_tenant_b(Request::new(DeleteCatalogRequest {
                uuid: Some(Uuid {
                    value: cat_id.clone(),
                }),
            })))
            .await
            .expect_err("Tenant B must not delete Tenant A catalog")
            .code(),
        tonic::Code::NotFound
    );

    // Profile, SSP, POA&M, Claim, Graph isolation for Tenant B
    assert_eq!(
        oscal_client
            .get_profile(as_tenant_b(Request::new(GetProfileRequest {
                uuid: Some(Uuid {
                    value: prof_id.clone(),
                }),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert!(
        oscal_client
            .list_profiles(as_tenant_b(Request::new(ListProfilesRequest::default())))
            .await
            .unwrap()
            .into_inner()
            .profiles
            .is_empty()
    );
    assert_eq!(
        oscal_client
            .update_profile(as_tenant_b(Request::new(UpdateProfileRequest {
                uuid: Some(Uuid {
                    value: prof_id.clone(),
                }),
                profile: Some(Profile::default()),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert_eq!(
        oscal_client
            .delete_profile(as_tenant_b(Request::new(DeleteProfileRequest {
                uuid: Some(Uuid {
                    value: prof_id.clone(),
                }),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );

    assert_eq!(
        oscal_client
            .get_ssp(as_tenant_b(Request::new(GetSspRequest {
                uuid: Some(Uuid {
                    value: ssp_id.clone(),
                }),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert!(
        oscal_client
            .list_ssps(as_tenant_b(Request::new(ListSspsRequest::default())))
            .await
            .unwrap()
            .into_inner()
            .ssps
            .is_empty()
    );
    assert_eq!(
        oscal_client
            .update_ssp(as_tenant_b(Request::new(UpdateSspRequest {
                uuid: Some(Uuid {
                    value: ssp_id.clone(),
                }),
                ssp: Some(SystemSecurityPlan::default()),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert_eq!(
        oscal_client
            .delete_ssp(as_tenant_b(Request::new(DeleteSspRequest {
                uuid: Some(Uuid {
                    value: ssp_id.clone(),
                }),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );

    assert_eq!(
        oscal_client
            .get_poam(as_tenant_b(Request::new(GetPoamRequest {
                uuid: Some(Uuid {
                    value: poam_id.clone(),
                }),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert!(
        oscal_client
            .list_poams(as_tenant_b(Request::new(ListPoamsRequest::default())))
            .await
            .unwrap()
            .into_inner()
            .poams
            .is_empty()
    );
    assert_eq!(
        oscal_client
            .update_poam(as_tenant_b(Request::new(UpdatePoamRequest {
                uuid: Some(Uuid {
                    value: poam_id.clone(),
                }),
                poam: Some(PlanOfActionAndMilestones::default()),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert_eq!(
        oscal_client
            .delete_poam(as_tenant_b(Request::new(DeletePoamRequest {
                uuid: Some(Uuid {
                    value: poam_id.clone(),
                }),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );

    assert_eq!(
        exchange_client
            .get_claim(as_tenant_b(Request::new(GetClaimRequest {
                claim_id: "claim-a-1".to_string(),
            })))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert!(
        exchange_client
            .list_claims(as_tenant_b(Request::new(ListClaimsRequest::default())))
            .await
            .unwrap()
            .into_inner()
            .claims
            .is_empty()
    );
    assert!(
        graph_client
            .traverse(as_tenant_b(Request::new(TraverseRequest {
                start_node: "node-a-1".to_string(),
                max_depth: 3,
                ..Default::default()
            })))
            .await
            .unwrap()
            .into_inner()
            .path
            .is_empty()
    );

    // 3. Tenant A can still read, update, and delete its own resources, and datastore stays synced
    let ctx_a = mizan_oscal::fabric::tenant::TenantContext::new(
        mizan_oscal::fabric::tenant::TenantId::new("tenant-a").unwrap(),
        mizan_oscal::fabric::tenant::UserId::new("system"),
    );
    let ctx_b = mizan_oscal::fabric::tenant::TenantContext::new(
        mizan_oscal::fabric::tenant::TenantId::new("tenant-b").unwrap(),
        mizan_oscal::fabric::tenant::UserId::new("system"),
    );
    let stored_cat_v1 = state
        .datastore
        .get_document(&ctx_a, "catalogs", &cat_id)
        .expect("datastore should contain Tenant A catalog v1");
    assert_eq!(stored_cat_v1.version, 1);
    assert!(
        state
            .datastore
            .get_document(&ctx_b, "catalogs", &cat_id)
            .is_err(),
        "Tenant B must not read Tenant A catalog from datastore"
    );
    assert!(
        state
            .datastore
            .get_document(&ctx_a, "profiles", &prof_id)
            .is_ok()
    );
    assert!(
        state
            .datastore
            .get_document(&ctx_a, "ssps", &ssp_id)
            .is_ok()
    );
    assert!(
        state
            .datastore
            .get_document(&ctx_a, "poams", &poam_id)
            .is_ok()
    );

    // Verify warm-load rehydration into a fresh MizanServerState from the populated datastore
    let rehydrated = MizanServerState::with_datastore(state.datastore.clone());
    assert!(
        rehydrated
            .catalogs
            .read()
            .unwrap()
            .contains_key(&format!("tenant-a/{cat_id}")),
        "MizanServerState::with_datastore must warm-load tenant catalogs"
    );

    // Verify create_catalog with empty/omitted UUID assigns and persists a non-empty UUID
    let auto_cat = oscal_client
        .create_catalog(as_tenant_a(Request::new(CreateCatalogRequest {
            catalog: Some(Catalog {
                uuid: None,
                ..Default::default()
            }),
        })))
        .await
        .expect("create catalog without uuid")
        .into_inner()
        .catalog
        .unwrap();
    let auto_cat_id = auto_cat
        .uuid
        .as_ref()
        .map(|u| u.value.as_str())
        .unwrap_or("");
    assert!(
        !auto_cat_id.is_empty(),
        "server must assign non-empty UUID when client omits uuid"
    );
    assert!(
        state
            .datastore
            .get_document(&ctx_a, "catalogs", auto_cat_id)
            .is_ok()
    );

    oscal_client
        .update_catalog(as_tenant_a(Request::new(UpdateCatalogRequest {
            uuid: Some(Uuid {
                value: cat_id.clone(),
            }),
            catalog: Some(Catalog {
                uuid: Some(Uuid {
                    value: cat_id.clone(),
                }),
                metadata: Some(Metadata {
                    title: "Tenant A Updated Catalog".to_string(),
                    version: "1.1.0".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        })))
        .await
        .expect("Tenant A update catalog");

    let stored_cat_v2 = state
        .datastore
        .get_document(&ctx_a, "catalogs", &cat_id)
        .expect("datastore should contain Tenant A catalog v2 after update");
    assert_eq!(stored_cat_v2.version, 2);
    assert!(
        stored_cat_v2
            .content_json
            .contains("Tenant A Updated Catalog")
    );

    let updated = oscal_client
        .get_catalog(as_tenant_a(Request::new(GetCatalogRequest {
            uuid: Some(Uuid {
                value: cat_id.clone(),
            }),
        })))
        .await
        .expect("Tenant A get updated catalog")
        .into_inner()
        .catalog
        .unwrap();
    assert_eq!(
        updated.metadata.as_ref().map(|m| m.title.as_str()),
        Some("Tenant A Updated Catalog")
    );

    oscal_client
        .delete_catalog(as_tenant_a(Request::new(DeleteCatalogRequest {
            uuid: Some(Uuid {
                value: cat_id.clone(),
            }),
        })))
        .await
        .expect("Tenant A delete catalog");
    assert!(
        state
            .datastore
            .get_document(&ctx_a, "catalogs", &cat_id)
            .is_err(),
        "datastore must remove catalog after delete_catalog"
    );

    let _ = shutdown_tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(2), server_task)
        .await
        .expect("server task terminated cleanly");
}
