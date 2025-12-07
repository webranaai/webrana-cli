// ============================================
// TUI Rendering - FORGE (Team Alpha)
// ============================================

use ratatui::{prelude::*, widgets::*};

use super::app::{App, AppState, FocusedPanel, MessageRole};

pub fn draw(f: &mut Frame, app: &App) {
    // Main layout: 3 columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Files
            Constraint::Percentage(55), // Chat
            Constraint::Percentage(25), // Output
        ])
        .split(f.size());

    // Draw file panel
    draw_files_panel(f, app, main_chunks[0]);

    // Chat area: split into messages and input
    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Messages
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status bar
        ])
        .split(main_chunks[1]);

    // Draw chat messages
    draw_chat_panel(f, app, chat_chunks[0]);

    // Draw input
    draw_input_panel(f, app, chat_chunks[1]);

    // Draw status bar
    draw_status_bar(f, app, chat_chunks[2]);

    // Draw output panel
    draw_output_panel(f, app, main_chunks[2]);

    // Draw help overlay if in help mode
    if app.state == AppState::Help {
        draw_help_overlay(f);
    }
}

fn draw_files_panel(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app.selected_file {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(file.as_str()).style(style)
        })
        .collect();

    let border_style = if app.focused_panel == FocusedPanel::Files {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let files_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Files ")
                .title_style(Style::default().fg(Color::Cyan).bold()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(files_list, area);
}

fn draw_chat_panel(f: &mut Frame, app: &App, area: Rect) {
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let (prefix, style) = match msg.role {
                MessageRole::User => ("▶ You", Style::default().fg(Color::Green)),
                MessageRole::Assistant => ("◀ AI", Style::default().fg(Color::Cyan)),
                MessageRole::System => ("● Sys", Style::default().fg(Color::Yellow)),
            };

            let content = format!("[{}] {}: {}", msg.timestamp, prefix, msg.content);
            ListItem::new(content).style(style)
        })
        .collect();

    let border_style = if app.focused_panel == FocusedPanel::Chat {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let chat = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Chat ")
            .title_style(Style::default().fg(Color::Cyan).bold()),
    );

    f.render_widget(chat, area);
}

fn draw_input_panel(f: &mut Frame, app: &App, area: Rect) {
    let input_style = if app.state == AppState::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(input_style)
                .title(if app.state == AppState::Input {
                    " Input (Esc to exit) "
                } else {
                    " Input (i to type) "
                })
                .title_style(input_style),
        );

    f.render_widget(input, area);

    // Show cursor in input mode
    if app.state == AppState::Input {
        f.set_cursor(area.x + app.cursor_position as u16 + 1, area.y + 1);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_style = match app.state {
        AppState::Input => Style::default().bg(Color::DarkGray).fg(Color::Yellow),
        AppState::Processing => Style::default().bg(Color::Blue).fg(Color::White),
        _ => Style::default().bg(Color::DarkGray).fg(Color::White),
    };

    let status = Paragraph::new(format!(
        " {} │ Tab: switch panel │ ?: help │ q: quit",
        app.status
    ))
    .style(status_style);

    f.render_widget(status, area);
}

fn draw_output_panel(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focused_panel == FocusedPanel::Output {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let output_text = if app.output.is_empty() {
        "Tool output will appear here..."
    } else {
        &app.output
    };

    let output = Paragraph::new(output_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Output ")
                .title_style(Style::default().fg(Color::Cyan).bold()),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(output, area);
}

fn draw_help_overlay(f: &mut Frame) {
    let area = centered_rect(60, 60, f.size());

    f.render_widget(Clear, area);

    let help_text = vec![
        "",
        "  WEBRANA AI - KEYBOARD SHORTCUTS",
        "  ═══════════════════════════════",
        "",
        "  NORMAL MODE",
        "  ───────────",
        "  i        Enter input mode",
        "  q        Quit application",
        "  ?        Show this help",
        "  Tab      Switch panel focus",
        "  j/↓      Scroll down / Next item",
        "  k/↑      Scroll up / Previous item",
        "",
        "  INPUT MODE",
        "  ──────────",
        "  Enter    Send message",
        "  Esc      Exit to normal mode",
        "  Ctrl+C   Quit application",
        "",
        "  Press q or Esc to close this help",
        "",
    ];

    let help = Paragraph::new(help_text.join("\n"))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help ")
                .title_style(Style::default().fg(Color::Cyan).bold())
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
