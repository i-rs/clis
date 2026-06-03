use super::strings;
use crate::app::App;
use crate::tui::colors::*;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render_shortcuts_overlay(frame: &mut Frame, area: Rect) {
    let overlay = area.centered(
        Constraint::Length(56.min(area.width.saturating_sub(4))),
        Constraint::Length(22),
    );

    frame.render_widget(Clear, overlay);

    let mut items = Vec::with_capacity(strings::SHORTCUTS.len() + 3);
    items.push(Line::from(""));
    for (key, label) in strings::SHORTCUTS {
        items.push(Line::from(Span::styled(
            format!("  {:<14} {}", key, label),
            Style::new().fg(C_TEXT),
        )));
    }
    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        "     Press any key to close",
        Style::new().fg(C_MUTED),
    )));

    let block = Block::default()
        .title(format!(" {} ", strings::SHORTCUT_TITLE))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(C_ACCENT))
        .style(Style::new().bg(C_BG_SURFACE));

    let paragraph = Paragraph::new(Text::from(items)).block(block).alignment(Alignment::Center);
    frame.render_widget(paragraph, overlay);
}

pub fn render_debug_overlay(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let w = area.width.saturating_sub(4).min(80);
    let h = area.height.saturating_sub(4).min(30);
    let overlay = area.centered(Constraint::Length(w), Constraint::Length(h));

    frame.render_widget(Clear, overlay);

    let logs = crate::debug::get_log();
    let scroll = app.debug_scroll.min(logs.len().saturating_sub(1));
    let visible: Vec<Line> = logs
        .iter()
        .skip(scroll)
        .take((h as usize).saturating_sub(3))
        .map(|entry| {
            let status_style = match entry.response_status {
                200 => Style::new().fg(C_GREEN),
                s if s >= 400 => Style::new().fg(C_RED),
                _ => Style::new().fg(C_ORANGE),
            };
            Line::from(vec![
                Span::styled(format!("{} ", entry.time_short()), Style::new().fg(C_MUTED)),
                Span::styled(entry.status_label(), status_style),
                Span::raw(format!(" {} ({}ms)", entry.path(), entry.duration_ms)),
            ])
        })
        .collect();

    let block = Block::default()
        .title(" Debug Log (Ctrl+B close, Ctrl+L clear, ↑↓ scroll) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(C_ORANGE))
        .style(Style::new().bg(C_BG_SURFACE));

    let paragraph = Paragraph::new(visible)
        .block(block)
        .scroll((if scroll > 0 { scroll as u16 } else { 0 }, 0));
    frame.render_widget(paragraph, overlay);
}

pub fn render_theme_picker(frame: &mut Frame, area: Rect, app: &App) {
    use crate::tui::colors::THEMES;
    let total = THEMES.len();
    let selected = app.theme_picker_selected.min(total.saturating_sub(1));

    let w = 50u16.min(area.width.saturating_sub(4));
    let h = (total as u16 + 4).min(area.height.saturating_sub(4));
    let overlay = area.centered(Constraint::Length(w), Constraint::Length(h));

    frame.render_widget(Clear, overlay);

    let mut items: Vec<Line> = Vec::with_capacity(total + 2);
    items.push(Line::from(""));
    for (i, theme) in THEMES.iter().enumerate() {
        let is_selected = i == selected;
        let marker = if is_selected { "▌" } else { " " };
        let name_style = if is_selected {
            Style::new().fg(C_ACCENT).bold()
        } else {
            Style::new().fg(C_TEXT)
        };
        let check = if theme.id == crate::tui::colors::active().id {
            "✓"
        } else {
            " "
        };
        items.push(Line::from(vec![
            Span::styled(
                marker,
                if is_selected { Style::new().fg(C_ACCENT) } else { Style::new().fg(C_MUTED) },
            ),
            Span::styled(format!(" {}", check), Style::new().fg(C_GREEN)),
            Span::styled(format!("  {:<14}", theme.id), name_style),
            Span::styled(theme.display_name, Style::new().fg(C_DIM)),
        ]));
    }
    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        "   ↑↓ 选择   Enter 确认   Esc 取消",
        Style::new().fg(C_MUTED),
    )));

    let block = Block::default()
        .title(" Select Theme ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(C_ACCENT))
        .style(Style::new().bg(C_BG_SURFACE));

    let paragraph = Paragraph::new(Text::from(items)).block(block);
    frame.render_widget(paragraph, overlay);
}
