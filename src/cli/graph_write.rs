#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

use crate::{
    config::{AppConfig, GraphAction},
    error::{AppError, Result},
    output,
    proto::oscal::services::v1::{
        DeleteEdgeRequest, DeleteEdgeResponse, ProjectEdgeRequest, ProjectEdgeResponse,
    },
    transport::CrudClient,
};

pub async fn run_graph(config: &AppConfig, action: GraphAction) -> Result<()> {
    let mut client = CrudClient::connect(config).await?;
    match action {
        GraphAction::ProjectEdge { claim_id, edge_id } => {
            validate_non_empty("claim_id", &claim_id)?;
            if let Some(ref id) = edge_id {
                validate_non_empty("edge_id", id)?;
            }
            let request = ProjectEdgeRequest {
                claim_id,
                edge_id: edge_id.unwrap_or_default(),
            };
            let result: crate::valence::CrudResult<ProjectEdgeResponse> =
                client.project_edge(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.ProjectEdge",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.ProjectEdgeResponse",
                &result,
                config.valence,
            )
        }
        GraphAction::DeleteEdge { edge_id } => {
            validate_non_empty("edge_id", &edge_id)?;
            let request = DeleteEdgeRequest { edge_id };
            let result: crate::valence::CrudResult<DeleteEdgeResponse> =
                client.delete_edge(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.DeleteEdge",
                &request,
                &result.data,
            )?;
            output::emit_crud_result(
                config.output,
                "oscal.services.v1.DeleteEdgeResponse",
                &result,
                config.valence,
            )
        }
        _ => Err(AppError::Configuration(
            "graph read actions require a read subcommand".to_owned(),
        )),
    }
}
