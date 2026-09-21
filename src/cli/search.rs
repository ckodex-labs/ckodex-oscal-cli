#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) async fn run_search(config: &AppConfig, args: SearchArgs) -> Result<()> {
    validate_non_empty("query", &args.query)?;
    validate_positive("top-k", args.top_k)?;
    if args.semantic && !args.model_types.is_empty() {
        return Err(AppError::InvalidArgument(
            "--model is only valid for standard search".to_owned(),
        ));
    }
    if !args.semantic && args.framework.is_some() {
        return Err(AppError::InvalidArgument(
            "--framework requires --semantic".to_owned(),
        ));
    }
    if args.semantic && !args.page_token.is_empty() {
        return Err(AppError::InvalidArgument(
            "--page-token is only valid for standard search".to_owned(),
        ));
    }
    let mut client = ReadOnlyClient::connect(config).await?;
    if args.semantic {
        let request = SemanticSearchRequest {
            query: args.query,
            framework: args.framework.unwrap_or_default(),
            top_k: args.top_k,
        };
        let response = client.semantic_search(request.clone()).await?;
        capture_if_enabled(
            config,
            "GovernanceService.SemanticSearch",
            &request,
            &response,
        )?;
        if config.output == OutputFormat::Table {
            let rows: Vec<Vec<String>> = response
                .results
                .iter()
                .map(|result| {
                    vec![
                        result.entity_type.clone(),
                        result.entity_urn.clone(),
                        format!("{:.4}", result.score),
                    ]
                })
                .collect();
            output::table(&["TYPE", "ENTITY", "SCORE"], &rows);
        } else {
            output::emit_message(
                config.output,
                "oscal.services.v1.SemanticSearchResponse",
                &response,
            )?;
        }
    } else {
        let request = SearchRequest {
            query: args.query,
            model_types: args.model_types,
            page_size: args.top_k,
            page_token: args.page_token,
        };
        let response = client.search(request.clone()).await?;
        capture_if_enabled(config, "OscalService.Search", &request, &response)?;
        if config.output == OutputFormat::Table {
            let rows: Vec<Vec<String>> = response
                .results
                .iter()
                .map(|result| {
                    vec![
                        result.model_type.clone(),
                        result
                            .uuid
                            .as_ref()
                            .map_or_else(String::new, |uuid| uuid.value.clone()),
                        result.title.clone(),
                        format!("{:.4}", result.score),
                    ]
                })
                .collect();
            output::table(&["MODEL", "UUID", "TITLE", "SCORE"], &rows);
            output::page_token(&response.next_page_token);
        } else {
            output::emit_message(config.output, "oscal.services.v1.SearchResponse", &response)?;
        }
    }
    Ok(())
}
