#![allow(unused_imports)]

use super::evidence::{Palette, evidence_panel};
#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn draw(frame: &mut ratatui::Frame<'_>, app: &TuiApp) {
    let palette = Palette::for_theme(app.theme);
    let outer = Block::default().style(Style::default().bg(palette.ground).fg(palette.ink));
    frame.render_widget(outer, frame.area());

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "OSCALIFY OBSERVER",
            Style::default()
                .fg(palette.rust)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  read-only protocol client"),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(palette.tone)),
    )
    .style(Style::default().bg(palette.ground).fg(palette.ink));
    frame.render_widget(header, layout[0]);

    let titles = Screen::ALL
        .iter()
        .map(|screen| Line::from(screen.label()))
        .collect::<Vec<_>>();
    let selected = Screen::ALL
        .iter()
        .position(|screen| *screen == app.screen)
        .unwrap_or(0);
    let tabs = Tabs::new(titles)
        .select(selected)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(palette.tone)),
        )
        .highlight_style(
            Style::default()
                .fg(palette.rust)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(palette.tone));
    frame.render_widget(tabs, layout[1]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(34)])
        .split(layout[2]);
    frame.render_widget(main_panel(app, palette), columns[0]);
    frame.render_widget(evidence_panel(app, palette), columns[1]);

    let footer_text = if app.search_mode {
        format!("/{}   Enter search   Esc cancel", app.search_query)
    } else if app.filter_mode {
        format!("filter {}   Enter apply   Esc cancel", app.filter_query)
    } else if app.detail.is_some() {
        if app.detail_claim_id.is_some() {
            "j/k scroll   e events   c receipt   PgUp/PgDn   Esc back   q quit".to_owned()
        } else if app.detail_edge_id.is_some() {
            "j/k scroll   p projection timeline   PgUp/PgDn   Esc back   q quit".to_owned()
        } else {
            "j/k scroll   PgUp/PgDn   Esc back   q quit".to_owned()
        }
    } else {
        "1–5 navigate   j/k select   [ ] page   f filter   / search   ? help   r refresh   q quit"
            .to_owned()
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(palette.tone))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(palette.tone)),
        );
    frame.render_widget(footer, layout[3]);

    if app.help {
        draw_help(frame, palette);
    }
}

pub(super) fn draw_help(frame: &mut ratatui::Frame<'_>, palette: Palette) {
    let bounds = frame.area();
    let width = bounds.width.saturating_sub(4).min(68);
    let height = bounds.height.saturating_sub(4).min(18);
    if width < 24 || height < 8 {
        return;
    }
    let area = Rect::new(
        bounds.x + (bounds.width - width) / 2,
        bounds.y + (bounds.height - height) / 2,
        width,
        height,
    );
    let lines = vec![
        Line::from("1–5 or Tab     change surface"),
        Line::from("j/k or arrows   move selection"),
        Line::from("Enter           inspect read-only detail"),
        Line::from("e / c in claim   verification events / receipt"),
        Line::from("p in edge        projection audit events"),
        Line::from("[ ] / PgUp/Dn   observed page navigation"),
        Line::from("/               server search"),
        Line::from("f               filter Explorer or Graph"),
        Line::from("r               refresh observed pages"),
        Line::from("Esc             close detail/help or quit"),
        Line::from("q               quit"),
        Line::from("? / Esc         close this help"),
        Line::from(""),
        Line::from("No key invokes a mutating OSCALify RPC."),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Help — read-only observer")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(palette.rust)),
            )
            .style(Style::default().bg(palette.ground).fg(palette.ink))
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub(super) fn main_panel(app: &TuiApp, palette: Palette) -> Paragraph<'static> {
    if let Some(detail) = &app.detail {
        return Paragraph::new(
            detail
                .lines
                .iter()
                .cloned()
                .map(Line::from)
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .title(detail.title.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.tone)),
        )
        .style(Style::default().fg(palette.ink))
        .scroll((detail.scroll, 0))
        .wrap(Wrap { trim: false });
    }

    let mut lines = Vec::new();
    let title = match app.screen {
        Screen::Overview => {
            lines.extend([
                "Mizan Compliance Engine — Real-time Observer & Policy Engine".to_owned(),
                "─────────────────────────────────────────────────────────────".to_owned(),
                format!("• Catalog page entries:    {}", app.catalogs.items.len()),
                format!("• Framework page entries:  {}", app.frameworks.items.len()),
                format!("• Claim page entries:      {}", app.claims.items.len()),
                String::new(),
                "Tri-Jurisdictional Baselines:".to_owned(),
                "  [US] NIST SP 800-53 Rev 5 / FedRAMP High".to_owned(),
                "  [CA] CCCS ITSG-33 Protected B / PBMM".to_owned(),
                "  [EU] EUCS & ISO/IEC 27001:2022".to_owned(),
                "  [Enterprise] Custom Inherited Overlays".to_owned(),
                String::new(),
                "Supply Chain & CI/CD Security:".to_owned(),
                "  ✓ SLSA v1.2 & v1.0 Provenance Attestation".to_owned(),
                "  ✓ CycloneDX & SPDX SBOM OSCAL Correlation".to_owned(),
                "  ✓ OASIS SARIF v2.1.0 & GitLab Security Exporters".to_owned(),
                "  ✓ Regorus Compliance Policy Rulepack Engine".to_owned(),
            ]);
            "Mizan Unified Compliance & Governance"
        }
        Screen::Explorer => {
            lines.push(format!(
                "filter  {}",
                output::terminal_text(filter_label(&app.filter_query))
            ));
            lines.push("Enter inspect   [ ] page".to_owned());
            let mut offset = 0;
            offset = append_page(&mut lines, "Catalogs", &app.catalogs, offset, app.selection);
            offset = append_page(
                &mut lines,
                "Frameworks",
                &app.frameworks,
                offset,
                app.selection,
            );
            append_page(&mut lines, "Claims", &app.claims, offset, app.selection);
            if app.visible_items().is_empty() {
                lines.push("No observed entries on the current pages.".to_owned());
            }
            "Explorer"
        }
        Screen::Search => {
            lines.push(if app.search_query.is_empty() {
                "Slash starts a server search.".to_owned()
            } else {
                format!("query  {}", output::terminal_text(&app.search_query))
            });
            lines.push("Enter inspect   [ ] page".to_owned());
            append_page(&mut lines, "Results", &app.search_page, 0, app.selection);
            if app.search_page.items.is_empty() && !app.search_mode {
                lines.push("No observed results.".to_owned());
            }
            "Search"
        }
        Screen::Graph => {
            lines.push(format!("filter  {}", filter_label(&app.filter_query)));
            lines.push("Enter inspect   [ ] page".to_owned());
            let mut offset = 0;
            offset = append_page(&mut lines, "Nodes", &app.nodes, offset, app.selection);
            append_page(&mut lines, "Edges", &app.edges, offset, app.selection);
            lines.push(String::new());
            lines.push("Analysis remains server-bound:".to_owned());
            lines.push("oscal-cli graph traverse NODE --max-depth 3".to_owned());
            lines.push("oscal-cli graph closure NODE \"release review\"".to_owned());
            lines.push("p in edge detail opens projection audit events".to_owned());
            "Graph analysis"
        }
        Screen::Evidence => {
            lines.extend([
                "Mizan Cryptographic & Epistemic Trust States:".to_owned(),
                "─────────────────────────────────────────────────────────────".to_owned(),
                "⊢ observed     server returned an empirical value".to_owned(),
                "⇝ inferred     client-derived navigation & blast radius".to_owned(),
                "○ claimed      server claim without cryptographic proof".to_owned(),
                "◆ attested     SLSA v1.2 / in-toto statement verified".to_owned(),
                "⊭ contradicted server contradiction / policy breach".to_owned(),
                "⊘ quarantined  containment & boundary isolation state".to_owned(),
                String::new(),
                "CAS & Verification Storage:".to_owned(),
                "  • Content-Addressable Storage (SHA-256 CAS Store)".to_owned(),
                "  • Merkle Proof Root Generation & Ledger".to_owned(),
                "  • Continuous Watch Daemon Invariant Checking".to_owned(),
                String::new(),
                "--capture persists request/response protobufs locally.".to_owned(),
            ]);
            "Evidence & Cryptographic Trust"
        }
    };
    Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.tone)),
        )
        .style(Style::default().fg(palette.ink))
        .scroll((list_scroll(app), 0))
        .wrap(Wrap { trim: false })
}

pub(super) fn list_scroll(app: &TuiApp) -> u16 {
    if matches!(
        app.screen,
        Screen::Explorer | Screen::Graph | Screen::Search
    ) {
        app.selection.saturating_sub(6) as u16
    } else {
        0
    }
}

pub(super) fn append_page(
    lines: &mut Vec<String>,
    title: &str,
    page: &PageState,
    offset: usize,
    selection: usize,
) -> usize {
    lines.push(title.to_owned());
    lines.push(format!(
        "page {}   next {}",
        page.cursor.page(),
        if page.cursor.can_next() {
            "available"
        } else {
            "none"
        }
    ));
    if page.items.is_empty() {
        lines.push("No observed entries on this page.".to_owned());
    } else {
        for (index, item) in page.items.iter().enumerate() {
            let marker = if offset + index == selection {
                "⊲"
            } else {
                " "
            };
            lines.push(format!("{marker} {}", item.summary));
        }
    }
    lines.push(String::new());
    offset + page.items.len()
}

pub(super) fn filter_label(filter: &str) -> &str {
    if filter.is_empty() { "none" } else { filter }
}

pub(super) fn json_lines(value: &Value) -> Vec<String> {
    serde_json::to_string_pretty(value)
        .map(|json| json.lines().map(str::to_owned).collect())
        .unwrap_or_else(|error| vec![format!("detail serialization unavailable: {error}")])
}
