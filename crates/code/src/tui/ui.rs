#[cfg(feature = "tui")]
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
#[cfg(feature = "tui")]
use crate::app::App;

#[cfg(feature = "tui")]
pub fn render(frame: &mut Frame, _app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title bar
    let title = Paragraph::new(" i-rs-code — Code Editor AI Agent ")
        .style(Style::default().fg(Color::White).on_blue());
    frame.render_widget(title, chunks[0]);

    // Main content area
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // File browser panel
    let file_panel = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let file_content = Paragraph::new(Text::from(""))
        .block(file_panel);
    frame.render_widget(file_content, main[0]);

    // Conversation panel
    let chat_panel = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let chat_content = Paragraph::new(Text::from(
        Line::from(vec![
            Span::styled("Welcome to i-rs-code", Style::default().bold()),
        ])
    ))
    .block(chat_panel)
    .wrap(Wrap { trim: false });
    frame.render_widget(chat_content, main[1]);

    // Input bar
    let input = Paragraph::new(" > Type your prompt... (q to quit) ")
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(input, chunks[2]);
}

#[cfg(not(feature = "tui"))]
pub fn render() {}
