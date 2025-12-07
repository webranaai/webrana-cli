// ============================================
// WEBRANA CLI - Terminal User Interface
// Created by: FORGE (Team Alpha)
// ============================================
//
// TUI is an optional feature. Enable with:
// cargo build --features tui

#[cfg(feature = "tui")]
mod app;
#[cfg(feature = "tui")]
mod event;
#[cfg(feature = "tui")]
mod ui;

#[cfg(feature = "tui")]
pub use app::{App, AppState};
#[cfg(feature = "tui")]
pub use event::{Event, EventHandler};
#[cfg(feature = "tui")]
pub use ui::draw;

use anyhow::Result;

/// Run the TUI application
#[cfg(feature = "tui")]
pub async fn run_tui() -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::prelude::*;
    use std::io;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    let event_handler = EventHandler::new(250);

    // Run the main loop
    let result = run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

#[cfg(feature = "tui")]
async fn run_app<B: ratatui::prelude::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        match event_handler.next().await? {
            Event::Tick => {
                app.tick();
            }
            Event::Key(key_event) => {
                if app.handle_key(key_event) {
                    break;
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    Ok(())
}

/// Stub when TUI feature is not enabled
#[cfg(not(feature = "tui"))]
pub async fn run_tui() -> Result<()> {
    Err(anyhow::anyhow!(
        "TUI feature not enabled. Rebuild with: cargo build --features tui"
    ))
}
