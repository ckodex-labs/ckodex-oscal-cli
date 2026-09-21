#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, ImportAction},
    error::{AppError, Result},
    output,
    proto::oscal::services::v1::{ImportBatchRequest, PreflightImportRequest},
    transport::CrudClient,
};
use serde_json::Value;
use std::path::Path;

pub(super) async fn run_import(config: &AppConfig, action: ImportAction) -> Result<()> {
    let mut client = CrudClient::connect(config).await?;
    match action {
        ImportAction::Preflight { file } => {
            let value = read_document_value(&file)?;
            let request: PreflightImportRequest =
                super::crud::decode_from_json("oscal.services.v1.PreflightImportRequest", &value)?;
            let result = client.preflight_import(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.PreflightImport",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.PreflightImportResponse",
                &result,
                config.valence,
            )
        }
        ImportAction::Batch {
            file,
            all_or_nothing,
        } => {
            let value = read_document_value(&file)?;
            let value = inject_all_or_nothing(value, all_or_nothing);
            let request: ImportBatchRequest =
                super::crud::decode_from_json("oscal.services.v1.ImportBatchRequest", &value)?;
            let result = client.import_batch(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyExchangeService.ImportBatch",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.ImportBatchResponse",
                &result,
                config.valence,
            )
        }
    }
}

fn read_document_value<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path_ref = path.as_ref();
    let content =
        std::fs::read_to_string(path_ref).map_err(|e| crate::error::io_error(path_ref, e))?;
    if is_yaml(path_ref) {
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

fn inject_all_or_nothing(value: Value, all_or_nothing: bool) -> Value {
    match value {
        Value::Object(mut map) => {
            map.insert("all_or_nothing".to_owned(), Value::Bool(all_or_nothing));
            Value::Object(map)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    #[test]
    fn read_document_value_parses_json_and_yaml() {
        let dir = std::env::temp_dir().join(format!("mizan-import-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let json_path = dir.join("records.json");
        let yaml_path = dir.join("records.yaml");
        std::fs::write(&json_path, r#"{"records": []}"#).unwrap();
        std::fs::write(&yaml_path, "records:\n").unwrap();

        let json_val = read_document_value(&json_path).unwrap();
        assert!(json_val.get("records").is_some());

        let yaml_val = read_document_value(&yaml_path).unwrap();
        assert!(yaml_val.get("records").is_some());
    }

    #[test]
    fn inject_all_or_nothing_overrides_flag() {
        let value = json!({"records": []});
        let injected = inject_all_or_nothing(value, true);
        assert_eq!(injected["all_or_nothing"], true);
    }

    #[test]
    fn inject_all_or_nothing_preserves_non_object() {
        let value = Value::Array(vec![]);
        assert!(matches!(
            inject_all_or_nothing(value, true),
            Value::Array(_)
        ));
    }
}
