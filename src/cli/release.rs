#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, ReleaseAction},
    error::Result,
    output,
    proto::oscal::services::v1::{CreateReleaseRequest, ListReleasesRequest},
    transport::{CrudClient, ReadOnlyClient},
};

pub(super) async fn run_release(config: &AppConfig, action: ReleaseAction) -> Result<()> {
    match action {
        ReleaseAction::List {
            page_size,
            page_token,
        } => {
            validate_positive("page-size", page_size)?;
            let mut client = ReadOnlyClient::connect(config).await?;
            let request = ListReleasesRequest {
                page_size,
                page_token,
            };
            let response: ListReleasesResponse = client.list_releases(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.ListReleases",
                &request,
                &response,
            )?;
            emit_names(
                config,
                "oscal.services.v1.ListReleasesResponse",
                &response,
                &response.names,
            )?;
        }
        ReleaseAction::Create {
            name,
            snapshot_name,
        } => {
            validate_non_empty("name", &name)?;
            validate_non_empty("snapshot_name", &snapshot_name)?;
            let mut client = CrudClient::connect(config).await?;
            let request = CreateReleaseRequest {
                name,
                snapshot_name,
            };
            let result = client.create_release(request.clone()).await?;
            capture_if_enabled(
                config,
                "GovernanceService.CreateRelease",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.CreateReleaseResponse",
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
    fn validate_release_args_reject_empty_values() {
        assert!(validate_non_empty("name", "").is_err());
        assert!(validate_non_empty("snapshot_name", "").is_err());
        assert!(validate_non_empty("name", "v1.0").is_ok());
        assert!(validate_non_empty("snapshot_name", "snap-1").is_ok());
    }

    #[test]
    fn validate_positive_rejects_invalid_page_size() {
        assert!(validate_positive("page-size", 0).is_err());
        assert!(validate_positive("page-size", -1).is_err());
        assert!(validate_positive("page-size", 5).is_ok());
    }
}
