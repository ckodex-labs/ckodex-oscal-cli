#![allow(unused_imports, clippy::module_inception)]

use super::evidence::{projection_timeline_lines, short_endpoint};
use super::views::{draw, filter_label};
#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::{Terminal, backend::TestBackend};

    use crate::proto::oscal::services::v1::GraphProjectionEvent;

    use super::*;

    pub(super) fn test_app(help: bool) -> TuiApp {
        TuiApp {
            config: AppConfig {
                endpoint: "http://observer.example:50051".to_owned(),
                token: None,
                timeout: Duration::from_secs(1),
                tls_domain: None,
                ca_cert: None,
                client_cert: None,
                client_key: None,
                output: crate::config::OutputFormat::Table,
                theme: Theme::Ledger,
                capture_root: PathBuf::from("/tmp/oscal-cli-test-captures"),
                capture_enabled: false,
                read_only: false,
                valence: false,
            },
            client: None,
            screen: Screen::Overview,
            connection: "connection unavailable".to_owned(),
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
            help,
            detail: None,
            detail_claim_id: None,
            detail_edge_id: None,
            receipts: Vec::new(),
            theme: Theme::Ledger,
        }
    }

    #[test]
    pub(super) fn page_cursor_tracks_forward_and_backward_navigation() {
        let mut cursor = PageCursor::default();
        cursor.reset();
        cursor.update_next("page-2".to_owned());

        assert!(cursor.next());
        assert_eq!(cursor.page(), 2);
        assert_eq!(cursor.current_token(), "page-2");

        cursor.update_next("page-3".to_owned());
        assert!(cursor.next());
        assert_eq!(cursor.page(), 3);
        assert_eq!(cursor.current_token(), "page-3");

        assert!(cursor.previous());
        assert_eq!(cursor.page(), 2);
        assert!(cursor.previous());
        assert_eq!(cursor.page(), 1);
        assert!(!cursor.previous());
    }

    #[test]
    pub(super) fn page_cursor_denies_unavailable_navigation() {
        let mut cursor = PageCursor::default();
        cursor.reset();

        assert!(!cursor.next());
        assert!(!cursor.previous());
        assert_eq!(cursor.page(), 1);
    }

    #[test]
    pub(super) fn page_cursor_preserves_forward_navigation_after_terminal_page() {
        let mut cursor = PageCursor::default();
        cursor.reset();
        cursor.update_next("page-2".to_owned());
        assert!(cursor.next());

        cursor.update_next(String::new());
        assert!(!cursor.next());
        assert!(cursor.previous());
        assert!(cursor.can_next());
        assert!(cursor.next());
        assert_eq!(cursor.current_token(), "page-2");
    }

    #[test]
    pub(super) fn filter_label_keeps_empty_filter_explicit() {
        assert_eq!(filter_label(""), "none");
        assert_eq!(filter_label("iso-42001"), "iso-42001");
    }

    #[test]
    pub(super) fn help_overlay_and_evidence_margin_render_in_terminal_backend() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let app = test_app(true);

        terminal
            .draw(|frame| draw(frame, &app))
            .expect("TUI should render to a test backend");

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("Help — read-only observer"));
        assert!(rendered.contains("Evidence margin"));
        assert!(rendered.contains("No key invokes a mutating OSCALify RPC."));
    }

    #[test]
    pub(super) fn endpoint_display_redacts_userinfo() {
        assert_eq!(
            short_endpoint("https://operator:secret@example.test:443/rpc"),
            "example.test:443"
        );
    }

    #[test]
    pub(super) fn projection_timeline_exposes_chain_state_and_hash_lineage() {
        let response = ListProjectionEventsResponse {
            events: vec![GraphProjectionEvent {
                sequence: 7,
                event_id: "projection-event-fixture".to_owned(),
                edge_id: "edge-fixture".to_owned(),
                claim_id: "claim-fixture".to_owned(),
                from_node: "node-a".to_owned(),
                to_node: "node-b".to_owned(),
                relation: "depends_on".to_owned(),
                evidence_digest: "sha256:evidence".to_owned(),
                trust_state: "observed".to_owned(),
                previous_hash: "sha256:previous".to_owned(),
                event_hash: "sha256:event".to_owned(),
                projected_at: Some(prost_types::Timestamp {
                    seconds: 1_700_000_000,
                    nanos: 123,
                }),
            }],
            chain_valid: true,
        };

        let lines = projection_timeline_lines(&response).join("\n");
        assert!(lines.contains("⊢ chain valid"));
        assert!(lines.contains("projection-event-fixture"));
        assert!(lines.contains("sha256:event"));
        assert!(lines.contains("sha256:previous"));
        assert!(lines.contains("node-a -> node-b"));
    }
}
