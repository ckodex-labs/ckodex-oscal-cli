#![allow(unused_imports)]

use super::evidence::{digest_preview, projection_timeline_lines};
use super::views::json_lines;
#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

impl TuiApp {
    pub(super) async fn search(&mut self) {
        let query = self.search_query.trim().to_owned();
        if query.is_empty() {
            self.record("rejected", "search query is empty".to_owned());
            return;
        }
        let request = SearchRequest {
            query,
            model_types: Vec::new(),
            page_size: 10,
            page_token: self.search_page.cursor.current_token(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.search(request.clone()).await
        } else {
            self.record("rejected", "search requires a connection".to_owned());
            return;
        };
        match result {
            Ok(response) => {
                self.search_page.items = response
                    .results
                    .iter()
                    .map(|result| {
                        let id = result
                            .uuid
                            .as_ref()
                            .map_or_else(|| "unidentified".to_owned(), |uuid| uuid.value.clone());
                        BrowseItem {
                            kind: ItemKind::Search,
                            id,
                            summary: format!(
                                "{}  {}  {}  {:.4}",
                                output::terminal_text(&result.model_type),
                                output::terminal_text(
                                    result
                                        .uuid
                                        .as_ref()
                                        .map_or("unidentified", |uuid| uuid.value.as_str()),
                                ),
                                output::terminal_text(&result.title),
                                result.score
                            ),
                        }
                    })
                    .collect();
                self.search_page
                    .cursor
                    .update_next(response.next_page_token.clone());
                self.capture("OscalService.Search", &request, &response);
                self.record(
                    "evaluated",
                    format!("search page {}", self.search_page.cursor.page()),
                );
            }
            Err(error) => self.record("rejected", format!("search: {error}")),
        }
    }

    pub(super) async fn next_page(&mut self) {
        let screen = self.screen;
        let moved = match screen {
            Screen::Explorer => {
                let catalogs = self.catalogs.cursor.next();
                let frameworks = self.frameworks.cursor.next();
                let claims = self.claims.cursor.next();
                catalogs || frameworks || claims
            }
            Screen::Graph => {
                let nodes = self.nodes.cursor.next();
                let edges = self.edges.cursor.next();
                nodes || edges
            }
            Screen::Search => self.search_page.cursor.next(),
            _ => false,
        };
        if !moved {
            self.record("rejected", "no next page".to_owned());
            return;
        }
        self.selection = 0;
        match screen {
            Screen::Explorer => {
                self.load_catalogs().await;
                self.load_frameworks().await;
                self.load_claims().await;
            }
            Screen::Graph => {
                self.load_nodes().await;
                self.load_edges().await;
            }
            Screen::Search => self.search().await,
            _ => {}
        }
    }

    pub(super) async fn previous_page(&mut self) {
        let screen = self.screen;
        let moved = match screen {
            Screen::Explorer => {
                let catalogs = self.catalogs.cursor.previous();
                let frameworks = self.frameworks.cursor.previous();
                let claims = self.claims.cursor.previous();
                catalogs || frameworks || claims
            }
            Screen::Graph => {
                let nodes = self.nodes.cursor.previous();
                let edges = self.edges.cursor.previous();
                nodes || edges
            }
            Screen::Search => self.search_page.cursor.previous(),
            _ => false,
        };
        if !moved {
            self.record("rejected", "first page".to_owned());
            return;
        }
        self.selection = 0;
        match screen {
            Screen::Explorer => {
                self.load_catalogs().await;
                self.load_frameworks().await;
                self.load_claims().await;
            }
            Screen::Graph => {
                self.load_nodes().await;
                self.load_edges().await;
            }
            Screen::Search => self.search().await,
            _ => {}
        }
    }

    pub(super) async fn apply_filter(&mut self) {
        if !matches!(self.screen, Screen::Explorer | Screen::Graph) {
            return;
        }
        self.reset_pages();
        self.refresh_data().await;
        self.record("transformed", "filter applied".to_owned());
    }

    pub(super) fn visible_items(&self) -> Vec<&BrowseItem> {
        match self.screen {
            Screen::Explorer => self
                .catalogs
                .items
                .iter()
                .chain(self.frameworks.items.iter())
                .chain(self.claims.items.iter())
                .collect(),
            Screen::Graph => self
                .nodes
                .items
                .iter()
                .chain(self.edges.items.iter())
                .collect(),
            Screen::Search => self.search_page.items.iter().collect(),
            _ => Vec::new(),
        }
    }

    pub(super) fn selected_item(&self) -> Option<BrowseItem> {
        self.visible_items()
            .get(self.selection)
            .map(|item| (*item).clone())
    }

    pub(super) fn move_selection(&mut self, delta: isize) {
        let item_count = self.visible_items().len();
        if item_count == 0 {
            self.selection = 0;
            return;
        }
        let next = self.selection as isize + delta;
        self.selection = next.clamp(0, item_count as isize - 1) as usize;
    }

    pub(super) async fn open_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            self.record("rejected", "no selected item".to_owned());
            return;
        };
        if item.id == "unidentified" {
            self.record(
                "rejected",
                "selected item has no stable identifier".to_owned(),
            );
            return;
        }
        self.detail_claim_id = None;
        self.detail_edge_id = None;
        match item.kind {
            ItemKind::Catalog => {
                let request = GetCatalogRequest {
                    uuid: Some(Uuid {
                        value: item.id.clone(),
                    }),
                };
                let result = if let Some(client) = self.client.as_mut() {
                    client.get_catalog(request.clone()).await
                } else {
                    Err(crate::error::AppError::InvalidArgument(
                        "detail requires a connection".to_owned(),
                    ))
                };
                match result {
                    Ok(response) => self.set_detail(
                        format!("Catalog {}", output::terminal_text(&item.id)),
                        "OscalService.GetCatalog",
                        "oscal.services.v1.GetCatalogResponse",
                        &request,
                        &response,
                        &item.id,
                    ),
                    Err(error) => self.record("rejected", format!("catalog detail: {error}")),
                }
            }
            ItemKind::Framework => {
                let request = GetFrameworkRequest {
                    ref_id: item.id.clone(),
                };
                let result = if let Some(client) = self.client.as_mut() {
                    client.get_framework(request.clone()).await
                } else {
                    Err(crate::error::AppError::InvalidArgument(
                        "detail requires a connection".to_owned(),
                    ))
                };
                match result {
                    Ok(response) => self.set_detail(
                        format!("Framework {}", output::terminal_text(&item.id)),
                        "GovernanceService.GetFramework",
                        "oscal.services.v1.GetFrameworkResponse",
                        &request,
                        &response,
                        &item.id,
                    ),
                    Err(error) => self.record("rejected", format!("framework detail: {error}")),
                }
            }
            ItemKind::Claim => {
                self.detail_claim_id = Some(item.id.clone());
                let request = GetClaimRequest {
                    claim_id: item.id.clone(),
                };
                let result = if let Some(client) = self.client.as_mut() {
                    client.get_claim(request.clone()).await
                } else {
                    Err(crate::error::AppError::InvalidArgument(
                        "detail requires a connection".to_owned(),
                    ))
                };
                match result {
                    Ok(response) => self.set_detail(
                        format!("Claim {}", output::terminal_text(&item.id)),
                        "TransparencyExchangeService.GetClaim",
                        "oscal.services.v1.GetClaimResponse",
                        &request,
                        &response,
                        &item.id,
                    ),
                    Err(error) => self.record("rejected", format!("claim detail: {error}")),
                }
            }
            ItemKind::Node => {
                let request = GetNodeRequest {
                    node_id: item.id.clone(),
                };
                let result = if let Some(client) = self.client.as_mut() {
                    client.get_node(request.clone()).await
                } else {
                    Err(crate::error::AppError::InvalidArgument(
                        "detail requires a connection".to_owned(),
                    ))
                };
                match result {
                    Ok(response) => self.set_detail(
                        format!("Node {}", output::terminal_text(&item.id)),
                        "TransparencyGraphService.GetNode",
                        "oscal.services.v1.GetNodeResponse",
                        &request,
                        &response,
                        &item.id,
                    ),
                    Err(error) => self.record("rejected", format!("node detail: {error}")),
                }
            }
            ItemKind::Edge => {
                self.detail_edge_id = Some(item.id.clone());
                let request = GetEdgeRequest {
                    edge_id: item.id.clone(),
                };
                let result = if let Some(client) = self.client.as_mut() {
                    client.get_edge(request.clone()).await
                } else {
                    Err(crate::error::AppError::InvalidArgument(
                        "detail requires a connection".to_owned(),
                    ))
                };
                match result {
                    Ok(response) => self.set_detail(
                        format!("Edge {}", output::terminal_text(&item.id)),
                        "TransparencyGraphService.GetEdge",
                        "oscal.services.v1.GetEdgeResponse",
                        &request,
                        &response,
                        &item.id,
                    ),
                    Err(error) => self.record("rejected", format!("edge detail: {error}")),
                }
            }
            ItemKind::Search => {
                self.detail = Some(DetailView {
                    title: "Search result".to_owned(),
                    lines: vec![
                        format!("query  {}", output::terminal_text(&self.search_query)),
                        String::new(),
                        item.summary,
                    ],
                    scroll: 0,
                });
                self.record("evaluated", "search result detail".to_owned());
            }
        }
    }

    pub(super) async fn open_claim_events(&mut self) {
        let Some(claim_id) = self.detail_claim_id.clone() else {
            self.record("rejected", "claim events require a claim detail".to_owned());
            return;
        };
        let request = ListVerificationEventsRequest {
            claim_id: claim_id.clone(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_verification_events(request.clone()).await
        } else {
            Err(crate::error::AppError::InvalidArgument(
                "claim events require a connection".to_owned(),
            ))
        };
        match result {
            Ok(response) => self.set_detail(
                format!("Claim events {claim_id}"),
                "TransparencyExchangeService.ListVerificationEvents",
                "oscal.services.v1.ListVerificationEventsResponse",
                &request,
                &response,
                &claim_id,
            ),
            Err(error) => self.record("rejected", format!("claim events: {error}")),
        }
    }

    pub(super) async fn open_claim_receipt(&mut self) {
        let Some(claim_id) = self.detail_claim_id.clone() else {
            self.record(
                "rejected",
                "claim receipt requires a claim detail".to_owned(),
            );
            return;
        };
        let request = ExportClaimReceiptRequest {
            claim_id: claim_id.clone(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.export_claim_receipt(request.clone()).await
        } else {
            Err(crate::error::AppError::InvalidArgument(
                "claim receipt requires a connection".to_owned(),
            ))
        };
        match result {
            Ok(response) => self.set_detail(
                format!("Claim receipt {claim_id}"),
                "TransparencyExchangeService.ExportClaimReceipt",
                "oscal.services.v1.ExportClaimReceiptResponse",
                &request,
                &response,
                &claim_id,
            ),
            Err(error) => self.record("rejected", format!("claim receipt: {error}")),
        }
    }

    pub(super) async fn open_projection_events(&mut self) {
        let Some(edge_id) = self.detail_edge_id.clone() else {
            self.record(
                "rejected",
                "projection events require an edge detail".to_owned(),
            );
            return;
        };
        let request = ListProjectionEventsRequest {
            claim_id: String::new(),
            edge_id: edge_id.clone(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_projection_events(request.clone()).await
        } else {
            Err(crate::error::AppError::InvalidArgument(
                "projection events require a connection".to_owned(),
            ))
        };
        match result {
            Ok(response) => self.set_projection_detail(edge_id, &request, &response),
            Err(error) => self.record("rejected", format!("projection events: {error}")),
        }
    }

    pub(super) fn set_projection_detail(
        &mut self,
        edge_id: String,
        request: &ListProjectionEventsRequest,
        response: &ListProjectionEventsResponse,
    ) {
        self.capture(
            "TransparencyGraphService.ListProjectionEvents",
            request,
            response,
        );
        self.detail = Some(DetailView {
            title: format!(
                "Projection events timeline {}",
                output::terminal_text(&edge_id)
            ),
            lines: projection_timeline_lines(response),
            scroll: 0,
        });
        self.record("evaluated", format!("projection timeline {edge_id}"));
    }

    pub(super) fn set_detail<M1: Message, M2: Message>(
        &mut self,
        title: String,
        method: &str,
        full_name: &str,
        request: &M1,
        response: &M2,
        id: &str,
    ) {
        self.capture(method, request, response);
        match output::message_json(full_name, response) {
            Ok(value) => {
                self.detail = Some(DetailView {
                    title,
                    lines: json_lines(&value),
                    scroll: 0,
                });
                self.record("evaluated", format!("detail {id}"));
            }
            Err(error) => self.record("rejected", format!("detail: {error}")),
        }
    }

    pub(super) fn capture<M1: Message, M2: Message>(
        &mut self,
        method: &str,
        request: &M1,
        response: &M2,
    ) {
        if !self.config.capture_enabled {
            return;
        }
        match capture::write(&self.config, method, request, response) {
            Ok(manifest) => self.record(
                "generated",
                format!("capture {}", digest_preview(&manifest.response_sha256)),
            ),
            Err(error) => self.record("rejected", format!("capture: {error}")),
        }
    }

    pub(super) fn record(&mut self, state: &'static str, detail: String) {
        self.receipts.push(Receipt {
            state,
            detail: output::terminal_text(&detail),
        });
        if self.receipts.len() > 8 {
            self.receipts.remove(0);
        }
    }
}
