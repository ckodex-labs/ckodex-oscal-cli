#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn evidence_panel(app: &TuiApp, palette: Palette) -> List<'static> {
    let mut items = vec![
        ListItem::new(Line::from(app.connection.clone())),
        ListItem::new(Line::from(format!(
            "endpoint  {}",
            output::terminal_text(&short_endpoint(&app.config.endpoint))
        ))),
        ListItem::new(Line::from("mode      read-only")),
        ListItem::new(Line::from("source    OSCALify proto")),
        ListItem::new(Line::from(
            "proto.lock  ".to_owned() + &digest_preview(&proto_digest()),
        )),
        ListItem::new(Line::from("")),
    ];
    items.extend(app.receipts.iter().map(|receipt| {
        ListItem::new(Line::from(vec![
            Span::styled(receipt.state, Style::default().fg(palette.tone)),
            Span::raw("  "),
            Span::raw(receipt.detail.clone()),
        ]))
    }));
    List::new(items)
        .block(
            Block::default()
                .title("Evidence margin")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.tone)),
        )
        .style(Style::default().fg(palette.ink))
}

#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub(super) ground: Color,
    pub(super) ink: Color,
    pub(super) tone: Color,
    pub(super) rust: Color,
}

impl Palette {
    pub(super) fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Ledger => Self {
                ground: Color::Rgb(246, 241, 232),
                ink: Color::Rgb(33, 27, 20),
                tone: Color::Rgb(110, 100, 87),
                rust: Color::Rgb(180, 83, 42),
            },
            Theme::Vault => Self {
                ground: Color::Rgb(10, 19, 34),
                ink: Color::Rgb(234, 229, 218),
                tone: Color::Rgb(154, 146, 132),
                rust: Color::Rgb(210, 105, 58),
            },
            Theme::Hc => Self {
                ground: Color::Black,
                ink: Color::White,
                tone: Color::Gray,
                rust: Color::Yellow,
            },
        }
    }
}

pub(super) fn proto_digest() -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(include_bytes!("../../proto.lock"));
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

pub(super) fn short_endpoint(endpoint: &str) -> String {
    let redacted = redact_endpoint(endpoint);
    redacted
        .strip_prefix("http://")
        .or_else(|| redacted.strip_prefix("https://"))
        .unwrap_or(redacted.as_str())
        .to_owned()
}

pub(super) fn digest_preview(digest: &str) -> String {
    if digest.len() <= 22 {
        return digest.to_owned();
    }
    format!("{}…{}", &digest[..9], &digest[digest.len() - 4..])
}

pub(super) fn projection_timeline_lines(response: &ListProjectionEventsResponse) -> Vec<String> {
    let chain_state = if response.chain_valid {
        "⊢ chain valid — observed append-only history"
    } else {
        "⊭ chain invalid — do not treat this history as attested"
    };
    let mut lines = vec![
        chain_state.to_owned(),
        format!("events  {}", response.events.len()),
        String::new(),
    ];
    if response.events.is_empty() {
        lines.push("No observed projection events.".to_owned());
        return lines;
    }

    for (index, event) in response.events.iter().enumerate() {
        let branch = if index + 1 == response.events.len() {
            "+--"
        } else {
            "|--"
        };
        lines.push(format!(
            "{branch} seq {}  {}  {}",
            event.sequence,
            output::terminal_text(&event.event_id),
            projection_timestamp(event.projected_at.as_ref())
        ));
        lines.push(format!(
            "     {} -> {}  {}  trust {}",
            output::terminal_text(&event.from_node),
            output::terminal_text(&event.to_node),
            output::terminal_text(&event.relation),
            output::terminal_text(&event.trust_state)
        ));
        lines.push(format!(
            "     claim {}  edge {}",
            output::terminal_text(&event.claim_id),
            output::terminal_text(&event.edge_id)
        ));
        lines.push(format!(
            "     event hash  {}",
            output::terminal_text(&digest_preview(&event.event_hash))
        ));
        let previous_hash = if event.previous_hash.is_empty() {
            "none".to_owned()
        } else {
            digest_preview(&event.previous_hash)
        };
        lines.push(format!(
            "     previous    {}",
            output::terminal_text(&previous_hash)
        ));
    }
    lines
}

pub(super) fn projection_timestamp(timestamp: Option<&prost_types::Timestamp>) -> String {
    timestamp
        .map(|value| format!("{}.{:09} UTC", value.seconds, value.nanos))
        .unwrap_or_else(|| "timestamp unavailable".to_owned())
}
