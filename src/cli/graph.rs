#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) async fn run_graph(config: &AppConfig, action: GraphAction) -> Result<()> {
    if let GraphAction::Traverse { max_depth, .. } | GraphAction::Impact { max_depth, .. } = &action
    {
        validate_positive("max-depth", *max_depth)?;
    }
    if let GraphAction::ProjectionEvents { claim_id, edge_id } = &action {
        if claim_id.is_none() && edge_id.is_none() {
            return Err(AppError::InvalidArgument(
                "one of --claim-id or --edge-id is required".to_owned(),
            ));
        }
        if let Some(claim_id) = claim_id {
            validate_non_empty("claim-id", claim_id)?;
        }
        if let Some(edge_id) = edge_id {
            validate_non_empty("edge-id", edge_id)?;
        }
    }
    let mut client = ReadOnlyClient::connect(config).await?;
    match action {
        GraphAction::Node { action } => match action {
            GraphNodeAction::List {
                kind,
                label_filter,
                page_size,
                page_token,
            } => {
                validate_positive("page-size", page_size)?;
                let request = ListNodesRequest {
                    kind,
                    label_filter,
                    created_after: None,
                    page_size,
                    page_token,
                };
                let response: ListNodesResponse = client.list_nodes(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "TransparencyGraphService.ListNodes",
                    &request,
                    &response,
                )?;
                if config.output == OutputFormat::Table {
                    let rows = response
                        .nodes
                        .iter()
                        .map(|node| vec![node.id.clone(), node.kind.clone(), node.urn.clone()])
                        .collect::<Vec<_>>();
                    output::table(&["ID", "KIND", "URN"], &rows);
                    output::page_token(&response.next_page_token);
                } else {
                    output::emit_message(
                        config.output,
                        "oscal.services.v1.ListNodesResponse",
                        &response,
                    )?;
                }
            }
            GraphNodeAction::Get { node_id } => {
                validate_non_empty("node-id", &node_id)?;
                let request = GetNodeRequest { node_id };
                let response: GetNodeResponse = client.get_node(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "TransparencyGraphService.GetNode",
                    &request,
                    &response,
                )?;
                output::emit_message(
                    config.output,
                    "oscal.services.v1.GetNodeResponse",
                    &response,
                )?;
            }
        },
        GraphAction::Edge { action } => match action {
            GraphEdgeAction::List {
                from_node,
                to_node,
                relation,
                trust_state,
                page_size,
                page_token,
            } => {
                validate_positive("page-size", page_size)?;
                let request = ListEdgesRequest {
                    from_node,
                    to_node,
                    relation,
                    trust_state,
                    valid_after: None,
                    page_size,
                    page_token,
                };
                let response: ListEdgesResponse = client.list_edges(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "TransparencyGraphService.ListEdges",
                    &request,
                    &response,
                )?;
                if config.output == OutputFormat::Table {
                    let rows = response
                        .edges
                        .iter()
                        .map(|edge| {
                            vec![
                                edge.id.clone(),
                                edge.from_node.clone(),
                                edge.to_node.clone(),
                                edge.relation.clone(),
                                edge.trust_state.clone(),
                            ]
                        })
                        .collect::<Vec<_>>();
                    output::table(&["ID", "FROM", "TO", "RELATION", "TRUST"], &rows);
                    output::page_token(&response.next_page_token);
                } else {
                    output::emit_message(
                        config.output,
                        "oscal.services.v1.ListEdgesResponse",
                        &response,
                    )?;
                }
            }
            GraphEdgeAction::Get { edge_id } => {
                validate_non_empty("edge-id", &edge_id)?;
                let request = GetEdgeRequest { edge_id };
                let response: GetEdgeResponse = client.get_edge(request.clone()).await?;
                capture_if_enabled(
                    config,
                    "TransparencyGraphService.GetEdge",
                    &request,
                    &response,
                )?;
                output::emit_message(
                    config.output,
                    "oscal.services.v1.GetEdgeResponse",
                    &response,
                )?;
            }
        },
        GraphAction::ProjectionEvents { claim_id, edge_id } => {
            let request = ListProjectionEventsRequest {
                claim_id: claim_id.unwrap_or_default(),
                edge_id: edge_id.unwrap_or_default(),
            };
            let response: ListProjectionEventsResponse =
                client.list_projection_events(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.ListProjectionEvents",
                &request,
                &response,
            )?;
            if config.output == OutputFormat::Table {
                let chain_state = if response.chain_valid {
                    "valid"
                } else {
                    "invalid"
                };
                println!("chain_valid  {chain_state}");
                let rows = response
                    .events
                    .iter()
                    .map(|event| {
                        vec![
                            event.sequence.to_string(),
                            event.event_id.clone(),
                            event.edge_id.clone(),
                            event.claim_id.clone(),
                            event.relation.clone(),
                            event.trust_state.clone(),
                            event.event_hash.clone(),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(
                    &[
                        "SEQ",
                        "EVENT",
                        "EDGE",
                        "CLAIM",
                        "RELATION",
                        "TRUST",
                        "EVENT HASH",
                    ],
                    &rows,
                );
            } else {
                output::emit_message(
                    config.output,
                    "oscal.services.v1.ListProjectionEventsResponse",
                    &response,
                )?;
            }
        }
        GraphAction::Traverse {
            start_node,
            relations,
            max_depth,
            min_trust_state,
            traversal_mode,
        } => {
            let request = TraverseRequest {
                start_node,
                relations,
                max_depth,
                min_trust_state,
                traversal_mode,
            };
            let response: TraverseResponse = client.traverse(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.Traverse",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.TraverseResponse",
                &response,
            )?;
        }
        GraphAction::Path {
            from_node,
            to_node,
            min_trust_state,
            allowed_relations,
        } => {
            let request = ShortestPathRequest {
                from_node,
                to_node,
                min_trust_state,
                allowed_relations,
            };
            let response: ShortestPathResponse = client.shortest_path(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.ShortestPath",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.ShortestPathResponse",
                &response,
            )?;
        }
        GraphAction::Impact {
            node,
            max_depth,
            min_trust_state,
            relations,
        } => {
            let request = ImpactRadiusRequest {
                node,
                max_depth,
                min_trust_state,
                relations,
            };
            let response: ImpactRadiusResponse = client.impact_radius(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.ImpactRadius",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.ImpactRadiusResponse",
                &response,
            )?;
        }
        GraphAction::Explain { claim_id, edge_id } => {
            let request = ExplainClaimRequest {
                claim_id,
                edge_id: edge_id.unwrap_or_default(),
            };
            let response: ExplainClaimResponse = client.explain_claim(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.ExplainClaim",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.ExplainClaimResponse",
                &response,
            )?;
        }
        GraphAction::Trust { claim_id, edge_id } => {
            let request = ComputeTrustStateRequest {
                claim_id,
                edge_id: edge_id.unwrap_or_default(),
            };
            let response: ComputeTrustStateResponse =
                client.compute_trust_state(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.ComputeTrustState",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.ComputeTrustStateResponse",
                &response,
            )?;
        }
        GraphAction::Closure {
            subject_node,
            purpose,
        } => {
            let request = VerifyClosureRequest {
                subject_node,
                purpose,
                as_of: None,
            };
            let response: VerifyClosureResponse = client.verify_closure(request.clone()).await?;
            capture_if_enabled(
                config,
                "TransparencyGraphService.VerifyClosure",
                &request,
                &response,
            )?;
            output::emit_message(
                config.output,
                "oscal.services.v1.VerifyClosureResponse",
                &response,
            )?;
        }
        GraphAction::ProjectEdge { .. } | GraphAction::DeleteEdge { .. } => {
            return Err(AppError::Configuration(
                "graph edge writes require network configuration".to_owned(),
            ));
        }
    }
    Ok(())
}
