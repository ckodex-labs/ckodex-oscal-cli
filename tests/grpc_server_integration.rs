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
