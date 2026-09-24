#![allow(unused_imports)]

use super::keymap::next_screen;
use super::views::draw;
#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) async fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    bootstrap: &mut Option<tokio::task::JoinHandle<TuiApp>>,
) -> Result<()> {
    loop {
        if bootstrap
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            let task = bootstrap
                .take()
                .expect("finished bootstrap task should be present");
            match task.await {
                Ok(loaded) => app.adopt_bootstrap(loaded),
                Err(error) => {
                    app.connection = "connection rejected".to_owned();
                    app.record("rejected", format!("startup observation: {error}"));
                }
            }
        }
        terminal
            .draw(|frame| draw(frame, app))
            .map_err(|error| crate::error::io_error("terminal", error))?;
        if event::poll(Duration::from_millis(100))
            .map_err(|error| crate::error::io_error("terminal", error))?
            && let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read().map_err(|error| crate::error::io_error("terminal", error))?
        {
            if modifiers.contains(KeyModifiers::CONTROL)
                && matches!(code, KeyCode::Char('c') | KeyCode::Char('C'))
            {
                break;
            }
            if app.help {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('?') | KeyCode::Esc => app.help = false,
                    _ => {}
                }
                continue;
            }
            if app.detail.is_some() {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => {
                        app.detail = None;
                        app.detail_claim_id = None;
                        app.detail_edge_id = None;
                    }
                    KeyCode::Char('e') if app.detail_claim_id.is_some() => {
                        app.open_claim_events().await
                    }
                    KeyCode::Char('c') if app.detail_claim_id.is_some() => {
                        app.open_claim_receipt().await
                    }
                    KeyCode::Char('p') if app.detail_edge_id.is_some() => {
                        app.open_projection_events().await
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(detail) = app.detail.as_mut() {
                            detail.scroll = detail.scroll.saturating_add(1);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(detail) = app.detail.as_mut() {
                            detail.scroll = detail.scroll.saturating_sub(1);
                        }
                    }
                    KeyCode::PageDown => {
                        if let Some(detail) = app.detail.as_mut() {
                            detail.scroll = detail.scroll.saturating_add(8);
                        }
                    }
                    KeyCode::PageUp => {
                        if let Some(detail) = app.detail.as_mut() {
                            detail.scroll = detail.scroll.saturating_sub(8);
                        }
                    }
                    _ => {}
                }
                continue;
            }
            if app.search_mode {
                match code {
                    KeyCode::Char(character) => app.search_query.push(character),
                    KeyCode::Backspace => {
                        app.search_query.pop();
                    }
                    KeyCode::Enter => {
                        app.search_mode = false;
                        app.search_page.reset();
                        app.search().await;
                    }
                    KeyCode::Esc => app.search_mode = false,
                    _ => {}
                }
                continue;
            }
            if app.filter_mode {
                match code {
                    KeyCode::Char(character) => app.filter_query.push(character),
                    KeyCode::Backspace => {
                        app.filter_query.pop();
                    }
                    KeyCode::Enter => {
                        app.filter_mode = false;
                        app.apply_filter().await;
                    }
                    KeyCode::Esc => app.filter_mode = false,
                    _ => {}
                }
                continue;
            }
            match code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('?') => app.help = true,
                KeyCode::Char('1') => {
                    app.screen = Screen::Overview;
                    app.selection = 0;
                }
                KeyCode::Char('2') => {
                    app.screen = Screen::Explorer;
                    app.selection = 0;
                }
                KeyCode::Char('3') => {
                    app.screen = Screen::Search;
                    app.selection = 0;
                }
                KeyCode::Char('4') => {
                    app.screen = Screen::Graph;
                    app.selection = 0;
                }
                KeyCode::Char('5') => {
                    app.screen = Screen::Evidence;
                    app.selection = 0;
                }
                KeyCode::Tab => {
                    app.screen = next_screen(app.screen);
                    app.selection = 0;
                }
                KeyCode::Char('r') if bootstrap.is_some() => app.record(
                    "rejected",
                    "initial observation is still evaluating".to_owned(),
                ),
                KeyCode::Char('r') => app.refresh().await,
                KeyCode::Char('/') => {
                    app.screen = Screen::Search;
                    app.search_mode = true;
                    app.search_query.clear();
                    app.search_page.reset();
                    app.selection = 0;
                }
                KeyCode::Char('f') if matches!(app.screen, Screen::Explorer | Screen::Graph) => {
                    app.filter_mode = true;
                    app.filter_query.clear();
                }
                KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
                KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
                KeyCode::Char(']') | KeyCode::PageDown => app.next_page().await,
                KeyCode::Char('[') | KeyCode::PageUp => app.previous_page().await,
                KeyCode::Enter
                    if matches!(
                        app.screen,
                        Screen::Explorer | Screen::Graph | Screen::Search
                    ) =>
                {
                    app.open_selected().await;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
