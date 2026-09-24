#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, EntityAction},
    error::{AppError, Result},
    output,
    proto::oscal::services::v1::{
        CreateEntityRequest, Entity, GetEntityRequest, ListEntitiesRequest, UpdateEntityRequest,
    },
    transport::{CrudClient, ReadOnlyClient},
};
use serde_json::Value;
use std::path::Path;

pub(super) async fn run_entity(config: &AppConfig, action: EntityAction) -> Result<()> {
    match action {
        EntityAction::List {
            type_filter,
            status_filter,
            page_size,
            page_token,
        } => {
            validate_positive("page-size", page_size)?;
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = ListEntitiesRequest {
                type_filter,
                status_filter,
                page_size,
                page_token,
            };
            let response: ListEntitiesResponse = client.list_entities(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.ListEntities",
                &request,
                &response,
            )?;
            if config.output == OutputFormat::Table {
                let rows = response
                    .entities
                    .iter()
                    .map(|entity| {
                        vec![
                            entity.urn.clone(),
                            entity.r#type.clone(),
                            entity.version.clone(),
                            entity.status.clone(),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(&["URN", "TYPE", "VERSION", "STATUS"], &rows);
                output::page_token(&response.next_page_token);
            } else {
                output::emit_message(
                    config.output,
                    "oscal.services.v1.ListEntitiesResponse",
                    &response,
                )?;
            }
        }
        EntityAction::Get { urn } => {
            validate_non_empty("urn", &urn)?;
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = GetEntityRequest { urn };
            let response: GetEntityResponse = client.get_entity(request.clone()).await?;
            capture_if_enabled(config, "GovernanceService.GetEntity", &request, &response)?;
            output::emit_message(
                config.output,
                "oscal.services.v1.GetEntityResponse",
                &response,
            )?;
        }
        EntityAction::Create { file } => {
            let entity = read_entity(&file)?;
            let mut client = CrudClient::connect(config).await?;
            let request = CreateEntityRequest {
                entity: Some(entity),
            };
            let result = client.create_entity(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.CreateEntity",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.CreateEntityResponse",
                &result,
                config.valence,
            )?;
        }
        EntityAction::Update { file } => {
            let entity = read_entity(&file)?;
            let mut client = CrudClient::connect(config).await?;
            let request = UpdateEntityRequest {
                entity: Some(entity),
            };
            let result = client.update_entity(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.UpdateEntity",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.UpdateEntityResponse",
                &result,
                config.valence,
            )?;
        }
    }
    Ok(())
}

fn read_entity<P: AsRef<Path>>(path: P) -> Result<Entity> {
    let value = read_document_value(path.as_ref())?;
    let entity_value = extract_message_value(value, "entity");
    super::crud::decode_from_json("oscal.services.v1.Entity", &entity_value)
}

fn read_document_value(path: &Path) -> Result<Value> {
    let content = std::fs::read_to_string(path).map_err(|e| crate::error::io_error(path, e))?;
    if is_yaml(path) {
        serde_yaml::from_str(&content)
            .map_err(|e| AppError::Configuration(format!("failed to parse YAML: {e}")))
    } else {
        serde_json::from_str(&content).map_err(AppError::Serialization)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ENTITY_TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn write_entity_file(content: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let counter = ENTITY_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "mizan-entity-test-{}-{}",
            std::process::id(),
            counter
        ));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("entity.json");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    #[test]
    fn read_entity_parses_wrapped_and_bare_documents() {
        let (_, wrapped) = write_entity_file(
            r#"{"entity": {"urn": "urn:test:1", "type": "control", "version": "1.0", "status": "active", "payload": "{}"}}"#,
        );
        let entity = read_entity(&wrapped).unwrap();
        assert_eq!(entity.urn, "urn:test:1");

        let (_, bare) = write_entity_file(
            r#"{"urn": "urn:test:2", "type": "catalog", "version": "2.0", "status": "draft", "payload": "{}"}"#,
        );
        let entity = read_entity(&bare).unwrap();
        assert_eq!(entity.urn, "urn:test:2");
    }

    #[test]
    fn read_entity_rejects_invalid_json() {
        let (_, path) = write_entity_file("not json");
        assert!(read_entity(&path).is_err());
    }

    #[test]
    fn read_entity_rejects_missing_file() {
        let path = std::env::temp_dir().join("missing-entity-file.json");
        assert!(read_entity(&path).is_err());
    }

    #[test]
    fn validate_positive_rejects_zero_and_negative() {
        assert!(validate_positive("page-size", 0).is_err());
        assert!(validate_positive("page-size", -1).is_err());
        assert!(validate_positive("page-size", 1).is_ok());
    }
}
