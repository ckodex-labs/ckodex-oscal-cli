#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::{
    AppConfig, CrosstermBackend, EnterAlternateScreen, LeaveAlternateScreen, ListCatalogsRequest,
    ListClaimsRequest, ListEdgesRequest, ListFrameworksRequest, ListNodesRequest, ReadOnlyClient,
    Result, Terminal, Theme, Uuid, disable_raw_mode, enable_raw_mode, execute, io, output,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Screen {
    Overview,
    Explorer,
    Search,
    Graph,
    Evidence,
}

impl Screen {
    pub(super) const ALL: [Self; 5] = [
        Self::Overview,
        Self::Explorer,
        Self::Search,
        Self::Graph,
        Self::Evidence,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Explorer => "Explorer",
            Self::Search => "Search",
            Self::Graph => "Graph",
            Self::Evidence => "Evidence",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Receipt {
    pub(super) state: &'static str,
    pub(super) detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ItemKind {
    Catalog,
    Framework,
    Claim,
    Node,
    Edge,
    Search,
}

#[derive(Clone, Debug)]
pub(super) struct BrowseItem {
    pub(super) kind: ItemKind,
    pub(super) id: String,
    pub(super) summary: String,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PageCursor {
    tokens: Vec<String>,
    next_tokens: Vec<String>,
    index: usize,
}

impl PageCursor {
    pub(super) fn current_token(&self) -> String {
        self.tokens.get(self.index).cloned().unwrap_or_default()
    }

    pub(super) fn page(&self) -> usize {
        self.index + 1
    }

    pub(super) fn reset(&mut self) {
        self.tokens = vec![String::new()];
        self.next_tokens = vec![String::new()];
        self.index = 0;
    }

    pub(super) fn update_next(&mut self, token: String) {
        if self.next_tokens.len() <= self.index {
            self.next_tokens.resize(self.index + 1, String::new());
        }
        self.next_tokens[self.index] = token;
    }

    pub(super) fn can_next(&self) -> bool {
        self.next_tokens
            .get(self.index)
            .is_some_and(|token| !token.is_empty())
    }

    pub(super) fn can_previous(&self) -> bool {
        self.index > 0
    }

    pub(super) fn next(&mut self) -> bool {
        if !self.can_next() {
            return false;
        }
        if self.tokens.is_empty() {
            self.tokens.push(String::new());
        }
        let next_token = self.next_tokens[self.index].clone();
        self.index += 1;
        if self.tokens.len() <= self.index {
            self.tokens.push(next_token);
        }
        true
    }

    pub(super) fn previous(&mut self) -> bool {
        if !self.can_previous() {
            return false;
        }
        self.index -= 1;
        true
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct PageState {
    pub(super) items: Vec<BrowseItem>,
    pub(super) cursor: PageCursor,
}

impl PageState {
    pub(super) fn reset(&mut self) {
        self.items.clear();
        self.cursor.reset();
    }
}

#[derive(Clone, Debug)]
pub(super) struct DetailView {
    pub(super) title: String,
    pub(super) lines: Vec<String>,
    pub(super) scroll: u16,
}

pub(super) struct TuiApp {
    pub(super) config: AppConfig,
    pub(super) client: Option<ReadOnlyClient>,
    pub(super) screen: Screen,
    pub(super) connection: String,
    pub(super) catalogs: PageState,
    pub(super) frameworks: PageState,
    pub(super) claims: PageState,
    pub(super) nodes: PageState,
    pub(super) edges: PageState,
    pub(super) search_page: PageState,
    pub(super) selection: usize,
    pub(super) search_query: String,
    pub(super) search_mode: bool,
    pub(super) filter_query: String,
    pub(super) filter_mode: bool,
    pub(super) help: bool,
    pub(super) detail: Option<DetailView>,
    pub(super) detail_claim_id: Option<String>,
    pub(super) detail_edge_id: Option<String>,
    pub(super) receipts: Vec<Receipt>,
    pub(super) theme: Theme,
}

pub(super) struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    pub(super) fn enter() -> Result<Self> {
        enable_raw_mode().map_err(|error| crate::error::io_error("terminal", error))?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(crate::error::io_error("terminal", error));
        }
        let backend = CrosstermBackend::new(stdout);
        match Terminal::new(backend) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let mut stdout = io::stdout();
                let _ = execute!(stdout, LeaveAlternateScreen);
                let _ = disable_raw_mode();
                Err(crate::error::io_error("terminal", error))
            }
        }
    }

    pub(super) fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

impl TuiApp {
    pub(super) fn new(config: AppConfig) -> Self {
        let theme = config.theme;
        Self {
            config,
            client: None,
            screen: Screen::Overview,
            connection: "connection evaluating".to_owned(),
            catalogs: PageState::default(),
            frameworks: PageState::default(),
            claims: PageState::default(),
            nodes: PageState::default(),
            edges: PageState::default(),
            search_page: PageState::default(),
            selection: 0,
            search_query: String::new(),
            search_mode: false,
            filter_query: String::new(),
            filter_mode: false,
            help: false,
            detail: None,
            detail_claim_id: None,
            detail_edge_id: None,
            receipts: Vec::new(),
            theme,
        }
    }

    pub(super) async fn bootstrap(config: AppConfig) -> Self {
        let mut app = Self::new(config);
        match ReadOnlyClient::connect(&app.config).await {
            Ok(client) => {
                app.client = Some(client);
                app.connection = "connection evaluated".to_owned();
                app.refresh_data().await;
            }
            Err(error) => app.record("rejected", error.to_string()),
        }
        app
    }

    pub(super) fn adopt_bootstrap(&mut self, mut loaded: Self) {
        self.client = loaded.client.take();
        self.connection = loaded.connection;
        self.catalogs = loaded.catalogs;
        self.frameworks = loaded.frameworks;
        self.claims = loaded.claims;
        self.nodes = loaded.nodes;
        self.edges = loaded.edges;
        self.search_page = loaded.search_page;
        self.receipts = loaded.receipts;
    }

    pub(super) async fn refresh(&mut self) {
        self.client = None;
        self.reset_pages();
        self.receipts.clear();
        self.connection = "connection evaluating".to_owned();
        match ReadOnlyClient::connect(&self.config).await {
            Ok(client) => {
                self.client = Some(client);
                self.connection = "connection evaluated".to_owned();
            }
            Err(error) => {
                self.connection = "connection rejected".to_owned();
                self.record("rejected", error.to_string());
                return;
            }
        }
        self.refresh_data().await;
    }

    pub(super) fn reset_pages(&mut self) {
        self.catalogs.reset();
        self.frameworks.reset();
        self.claims.reset();
        self.nodes.reset();
        self.edges.reset();
        self.search_page.reset();
        self.selection = 0;
        self.detail = None;
        self.detail_claim_id = None;
        self.detail_edge_id = None;
    }

    pub(super) async fn refresh_data(&mut self) {
        self.receipts.clear();
        self.load_catalogs().await;
        self.load_frameworks().await;
        self.load_claims().await;
        self.load_nodes().await;
        self.load_edges().await;
    }

    pub(super) async fn load_catalogs(&mut self) {
        let request = ListCatalogsRequest {
            page_size: 10,
            page_token: self.catalogs.cursor.current_token(),
            filter: self.filter_query.clone(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_catalogs(request.clone()).await
        } else {
            return;
        };
        match result {
            Ok(response) => {
                self.catalogs.items = response
                    .catalogs
                    .iter()
                    .map(|catalog| {
                        let id = catalog
                            .uuid
                            .as_ref()
                            .map_or_else(|| "unidentified".to_owned(), |uuid| uuid.value.clone());
                        let title = catalog.metadata.as_ref().map_or_else(
                            || "untitled".to_owned(),
                            |metadata| metadata.title.clone(),
                        );
                        BrowseItem {
                            kind: ItemKind::Catalog,
                            id: id.clone(),
                            summary: format!(
                                "{}  {}",
                                output::terminal_text(&id),
                                output::terminal_text(&title)
                            ),
                        }
                    })
                    .collect();
                self.catalogs
                    .cursor
                    .update_next(response.next_page_token.clone());
                self.capture("OscalService.ListCatalogs", &request, &response);
                self.record(
                    "evaluated",
                    format!("catalogs page {}", self.catalogs.cursor.page()),
                );
            }
            Err(error) => self.record("rejected", format!("catalogs: {error}")),
        }
    }

    pub(super) async fn load_frameworks(&mut self) {
        let request = ListFrameworksRequest {
            jurisdiction_filter: self.filter_query.clone(),
            page_size: 10,
            page_token: self.frameworks.cursor.current_token(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_frameworks(request.clone()).await
        } else {
            return;
        };
        match result {
            Ok(response) => {
                self.frameworks.items = response
                    .frameworks
                    .iter()
                    .map(|framework| BrowseItem {
                        kind: ItemKind::Framework,
                        id: framework.ref_id.clone(),
                        summary: format!(
                            "{}  {}",
                            output::terminal_text(&framework.ref_id),
                            output::terminal_text(&framework.name)
                        ),
                    })
                    .collect();
                self.frameworks
                    .cursor
                    .update_next(response.next_page_token.clone());
                self.capture("GovernanceService.ListFrameworks", &request, &response);
                self.record(
                    "evaluated",
                    format!("frameworks page {}", self.frameworks.cursor.page()),
                );
            }
            Err(error) => self.record("rejected", format!("frameworks: {error}")),
        }
    }

    pub(super) async fn load_claims(&mut self) {
        let request = ListClaimsRequest {
            subject_digest: String::new(),
            bom_kind: String::new(),
            relation: self.filter_query.clone(),
            trust_state: String::new(),
            valid_after: None,
            page_size: 10,
            page_token: self.claims.cursor.current_token(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_claims(request.clone()).await
        } else {
            return;
        };
        match result {
            Ok(response) => {
                self.claims.items = response
                    .claims
                    .iter()
                    .map(|claim| {
                        let relation = claim.predicate.as_ref().map_or_else(
                            || "unqualified".to_owned(),
                            |predicate| predicate.relation.clone(),
                        );
                        BrowseItem {
                            kind: ItemKind::Claim,
                            id: claim.id.clone(),
                            summary: format!(
                                "{}  {}",
                                output::terminal_text(&claim.id),
                                output::terminal_text(&relation)
                            ),
                        }
                    })
                    .collect();
                self.claims
                    .cursor
                    .update_next(response.next_page_token.clone());
                self.capture(
                    "TransparencyExchangeService.ListClaims",
                    &request,
                    &response,
                );
                self.record(
                    "evaluated",
                    format!("claims page {}", self.claims.cursor.page()),
                );
            }
            Err(error) => self.record("rejected", format!("claims: {error}")),
        }
    }

    pub(super) async fn load_nodes(&mut self) {
        let request = ListNodesRequest {
            kind: self.filter_query.clone(),
            label_filter: String::new(),
            created_after: None,
            page_size: 10,
            page_token: self.nodes.cursor.current_token(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_nodes(request.clone()).await
        } else {
            return;
        };
        match result {
            Ok(response) => {
                self.nodes.items = response
                    .nodes
                    .iter()
                    .map(|node| BrowseItem {
                        kind: ItemKind::Node,
                        id: node.id.clone(),
                        summary: format!(
                            "{}  {}",
                            output::terminal_text(&node.id),
                            output::terminal_text(&node.kind)
                        ),
                    })
                    .collect();
                self.nodes
                    .cursor
                    .update_next(response.next_page_token.clone());
                self.capture("TransparencyGraphService.ListNodes", &request, &response);
                self.record(
                    "evaluated",
                    format!("nodes page {}", self.nodes.cursor.page()),
                );
            }
            Err(error) => self.record("rejected", format!("nodes: {error}")),
        }
    }

    pub(super) async fn load_edges(&mut self) {
        let request = ListEdgesRequest {
            from_node: String::new(),
            to_node: String::new(),
            relation: self.filter_query.clone(),
            trust_state: String::new(),
            valid_after: None,
            page_size: 10,
            page_token: self.edges.cursor.current_token(),
        };
        let result = if let Some(client) = self.client.as_mut() {
            client.list_edges(request.clone()).await
        } else {
            return;
        };
        match result {
            Ok(response) => {
                self.edges.items = response
                    .edges
                    .iter()
                    .map(|edge| BrowseItem {
                        kind: ItemKind::Edge,
                        id: edge.id.clone(),
                        summary: format!(
                            "{}  {} → {}",
                            output::terminal_text(&edge.relation),
                            output::terminal_text(&edge.from_node),
                            output::terminal_text(&edge.to_node)
                        ),
                    })
                    .collect();
                self.edges
                    .cursor
                    .update_next(response.next_page_token.clone());
                self.capture("TransparencyGraphService.ListEdges", &request, &response);
                self.record(
                    "evaluated",
                    format!("edges page {}", self.edges.cursor.page()),
                );
            }
            Err(error) => self.record("rejected", format!("edges: {error}")),
        }
    }
}
