use crate::{
    capture,
    config::{AppConfig, Theme},
    error::{redact_endpoint, Result},
    output,
    proto::oscal::{
        common::v1::Uuid,
        services::v1::{
            ExportClaimReceiptRequest, GetCatalogRequest, GetClaimRequest, GetEdgeRequest,
            GetFrameworkRequest, GetNodeRequest, ListCatalogsRequest, ListClaimsRequest,
            ListEdgesRequest, ListFrameworksRequest, ListNodesRequest, ListProjectionEventsRequest,
            ListProjectionEventsResponse, ListVerificationEventsRequest, SearchRequest,
        },
    },
    transport::ReadOnlyClient,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use prost::Message;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Terminal,
};
use serde_json::Value;
use sha2::Digest;
use std::{io, time::Duration};

pub async fn run(config: AppConfig) -> Result<()> {
    let mut app = TuiApp::new(config.clone());
    let mut session = TerminalSession::enter()?;
    let mut bootstrap = Some(tokio::spawn(TuiApp::bootstrap(config)));
    let result = events::event_loop(session.terminal_mut(), &mut app, &mut bootstrap).await;
    if let Some(task) = bootstrap.take() {
        task.abort();
    }
    drop(session);
    result
}

pub mod browse;
pub mod events;
pub mod evidence;
pub mod keymap;
pub mod state;
pub mod views;

use self::state::*;

#[cfg(test)]
mod tests;
