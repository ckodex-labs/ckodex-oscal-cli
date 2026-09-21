#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, SnapshotAction},
    error::Result,
    output,
    proto::oscal::services::v1::{CreateSnapshotRequest, GetSnapshotRequest, ListSnapshotsRequest},
    transport::{CrudClient, ReadOnlyClient},
};

pub(super) async fn run_snapshot(config: &AppConfig, action: SnapshotAction) -> Result<()> {
    match action {
        SnapshotAction::List {
            page_size,
            page_token,
        } => {
            validate_positive("page-size", page_size)?;
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = ListSnapshotsRequest {
                page_size,
                page_token,
            };
            let response: ListSnapshotsResponse = client.list_snapshots(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.ListSnapshots",
                &request,
                &response,
            )?;
            emit_names(
                config,
                "oscal.services.v1.ListSnapshotsResponse",
                &response,
                &response.names,
            )?;
        }
        SnapshotAction::Get { name } => {
            validate_non_empty("name", &name)?;
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = GetSnapshotRequest { name };
            let response: GetSnapshotResponse = client.get_snapshot(request.clone()).await?;
            capture_if_enabled(config, "GovernanceService.GetSnapshot", &request, &response)?;
            output::emit_message(
                config.output,
                "oscal.services.v1.GetSnapshotResponse",
                &response,
            )?;
        }
        SnapshotAction::Create { name } => {
            validate_non_empty("name", &name)?;
            let mut client = CrudClient::connect(config).await?;
            let request = CreateSnapshotRequest { name };
            let result = client.create_snapshot(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.CreateSnapshot",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.CreateSnapshotResponse",
                &result,
                config.valence,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_non_empty_rejects_blank_snapshot_name() {
        assert!(validate_non_empty("name", "").is_err());
        assert!(validate_non_empty("name", "   ").is_err());
        assert!(validate_non_empty("name", "snap-1").is_ok());
    }

    #[test]
    fn validate_positive_rejects_invalid_page_size() {
        assert!(validate_positive("page-size", 0).is_err());
        assert!(validate_positive("page-size", -5).is_err());
        assert!(validate_positive("page-size", 10).is_ok());
    }
}
