use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Calculate required height for input area based on content line count.
pub(super) fn input_height(input: &str) -> u16 {
    let content_lines = input.lines().count().max(1);
    // content lines + hint line + top/bottom borders
    (content_lines + 1 + 2).clamp(3, 20) as u16
}

/// Return the input hint line showing available shortcuts.
/// Voice input hint is only shown on macOS where Fn key is available.
pub(super) fn input_hint_text() -> &'static str {
    if cfg!(target_os = "macos") {
        "  [Fn] 语音输入  [Alt+Enter] 换行"
    } else {
        "  [Alt+Enter] 换行"
    }
}

pub(super) fn render_input(f: &mut Frame, area: Rect, app: &App) {
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.is_processing() {
            Color::DarkGray
        } else if app.input.is_empty() {
            Color::Rgb(80, 80, 100)
        } else {
            Color::Cyan
        }));

    let prefix = if app.is_processing() { "⏳ " } else { "❯ " };

    let lines: Vec<Line> = if app.is_processing() {
        vec![Line::from(Span::styled(
            format!("{}{}", prefix, app.input),
            Style::default().fg(Color::DarkGray),
        ))]
    } else if app.input.is_empty() {
        vec![
            Line::from(Span::styled(
                format!("{}输入消息...", prefix),
                Style::default().fg(Color::Rgb(80, 80, 100)),
            )),
            Line::from(Span::styled(
                input_hint_text(),
                Style::default().fg(Color::Rgb(60, 60, 80)),
            )),
        ]
    } else {
        let mut result: Vec<Line> = app.input.lines().enumerate().map(|(i, line)| {
            let p = if i == 0 { prefix } else { "  " };
            Line::from(Span::styled(
                format!("{}{}", p, line),
                Style::default().fg(Color::White),
            ))
        }).collect();
        result.push(Line::from(Span::styled(
            input_hint_text(),
            Style::default().fg(Color::Rgb(60, 60, 80)),
        )));
        result
    };

    let input_widget = Paragraph::new(lines).block(input_block);

    f.render_widget(input_widget, area);

    // Set cursor position (only when not processing)
    if !app.is_processing() && !app.input.is_empty() {
        let input_before = &app.input[..app.input_cursor];
        let line_idx = input_before.matches('\n').count();
        let current_line_start = input_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let pos_in_line = unicode_width::UnicodeWidthStr::width(&input_before[current_line_start..]);
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
        let cursor_x = area.x + 1 + prefix_width as u16 + pos_in_line as u16;
        let cursor_y = area.y + 1 + line_idx as u16;
        f.set_cursor_position((cursor_x, cursor_y));
    } else if !app.is_processing() {
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
        let cursor_x = area.x + 1 + prefix_width as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
