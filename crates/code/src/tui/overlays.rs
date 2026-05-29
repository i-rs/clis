use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::tui::colors::*;
use super::strings;

pub fn render_shortcuts_overlay(frame: &mut Frame, area: Rect) {
    let overlay = area.centered(
        Constraint::Length(56.min(area.width.saturating_sub(4))),
        Constraint::Length(22),
    );

    frame.render_widget(Clear, overlay);

    let items: Vec<Line> = std::iter::once(Line::from(""))
        .chain(strings::SHORTCUTS.iter().map(|(key, label)| {
            Line::from(Span::styled(
                format!("  {:<14} {}", key, label),
                Style::new().fg(Color::White),
            ))
        }))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(Span::styled(
            "     Press any key to close", Style::new().fg(C_DIM),
        ))))
        .collect();

    let block = Block::default()
        .title(format!(" {} ", strings::SHORTCUT_TITLE))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan));

    let paragraph = Paragraph::new(Text::from(items))
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, overlay);
}

pub fn render_debug_overlay(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let w = area.width.saturating_sub(4).min(80);
    let h = area.height.saturating_sub(4).min(30);
    let overlay = area.centered(Constraint::Length(w), Constraint::Length(h));

    frame.render_widget(Clear, overlay);

    let logs = crate::debug::get_log();
    let scroll = app.debug_scroll.min(logs.len().saturating_sub(1));
    let visible: Vec<Line> = logs.iter().skip(scroll).take((h as usize).saturating_sub(3)).map(|entry| {
        let status_style = match entry.response_status {
            200 => Style::new().fg(Color::Green),
            s if s >= 400 => Style::new().fg(Color::Red),
            _ => Style::new().fg(Color::Yellow),
        };
        Line::from(vec![
            Span::styled(format!("{} ", entry.time_short()), Style::new().fg(C_DIM)),
            Span::styled(entry.status_label(), status_style),
            Span::raw(format!(" {} ({}ms)", entry.path(), entry.duration_ms)),
        ])
    }).collect();

    let block = Block::default()
        .title(" Debug Log (Ctrl+B close, Ctrl+L clear, ↑↓ scroll) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Yellow));

    let paragraph = Paragraph::new(Text::from(visible))
        .block(block)
        .scroll((if scroll > 0 { scroll as u16 } else { 0 }, 0));
    frame.render_widget(paragraph, overlay);
}
