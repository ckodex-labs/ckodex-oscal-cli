#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, FrameworkAction},
    error::{AppError, Result},
    output,
    proto::oscal::services::v1::{
        GetFrameworkRequest, IngestRequirementsRequest, ListFrameworksRequest,
    },
    transport::{CrudClient, ReadOnlyClient},
};
use std::path::Path;

pub(super) async fn run_framework(config: &AppConfig, action: FrameworkAction) -> Result<()> {
    if let FrameworkAction::List { page_size, .. } = &action {
        validate_positive("page-size", *page_size)?;
    }
    match action {
        FrameworkAction::List {
            jurisdiction,
            page_size,
            page_token,
        } => {
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = ListFrameworksRequest {
                jurisdiction_filter: jurisdiction.unwrap_or_default(),
                page_size,
                page_token,
            };
            let response: ListFrameworksResponse = client.list_frameworks(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.ListFrameworks",
                &request,
                &response,
            )?;
            let rows = response
                .frameworks
                .iter()
                .map(|item| FrameworkRow {
                    ref_id: item.ref_id.clone(),
                    name: item.name.clone(),
                    provider: item.provider.clone(),
                    jurisdiction: item.jurisdiction.clone(),
                    version: item.version.clone(),
                    nodes: item.node_count,
                    assessable: item.assessable_count,
                })
                .collect::<Vec<_>>();
            if config.output == OutputFormat::Table {
                let table_rows = rows
                    .iter()
                    .map(|row| {
                        vec![
                            row.ref_id.clone(),
                            row.name.clone(),
                            row.jurisdiction.clone(),
                            row.version.clone(),
                            row.nodes.to_string(),
                            row.assessable.to_string(),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(
                    &[
                        "REF",
                        "NAME",
                        "JURISDICTION",
                        "VERSION",
                        "NODES",
                        "ASSESSABLE",
                    ],
                    &table_rows,
                );
                output::page_token(&response.next_page_token);
            } else if config.output == OutputFormat::Proto {
                output::emit_message(
                    config.output,
                    "oscal.services.v1.ListFrameworksResponse",
                    &response,
                )?;
            } else {
                output::emit_json(
                    config.output,
                    &serde_json::json!({
                        "items": rows,
                        "nextPageToken": response.next_page_token,
                    }),
                )?;
            }
        }
        FrameworkAction::Get { ref_id } => {
            validate_non_empty("ref_id", &ref_id)?;
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = GetFrameworkRequest { ref_id };
            let response: GetFrameworkResponse = client.get_framework(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.GetFramework",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.GetFrameworkResponse",
                &response,
            )?;
        }
        FrameworkAction::Ingest {
            file,
            format,
            framework,
        } => {
            validate_non_empty("format", &format)?;
            validate_non_empty("framework", &framework)?;
            let raw_data = read_file_bytes(&file)?;
            let mut client = CrudClient::connect(config).await?;
            let request = IngestRequirementsRequest {
                raw_data,
                format,
                framework,
            };
            let result = client.ingest_requirements(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.IngestRequirements",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.IngestRequirementsResponse",
                &result,
                config.valence,
            )?;
        }
    }
    Ok(())
}

fn read_file_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let path_ref = path.as_ref();
    std::fs::read(path_ref).map_err(|e| crate::error::io_error(path_ref, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(content: &[u8]) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("mizan-framework-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("requirements.json");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    #[test]
    fn read_file_bytes_loads_content() {
        let (_, path) = write_file(b"{\"controls\": []}");
        assert_eq!(read_file_bytes(&path).unwrap(), b"{\"controls\": []}");
    }

    #[test]
    fn read_file_bytes_fails_for_missing_file() {
        let path = std::env::temp_dir().join("missing-framework-file.json");
        assert!(read_file_bytes(&path).is_err());
    }

    #[test]
    fn validate_ingest_args_reject_empty_values() {
        assert!(validate_non_empty("format", "").is_err());
        assert!(validate_non_empty("framework", "").is_err());
        assert!(validate_non_empty("format", "json").is_ok());
        assert!(validate_non_empty("framework", "nist-800-53").is_ok());
    }
}
