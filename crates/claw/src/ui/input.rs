use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::app::App;

pub(super) fn input_height(input: &str, terminal_width: u16) -> u16 {
    let max_visual_width = (terminal_width as usize).saturating_sub(4).max(20);
    let content_lines = if input.is_empty() {
        1
    } else {
        input
            .lines()
            .map(|line| {
                let w = UnicodeWidthStr::width(line);
                if w == 0 {
                    1
                } else {
                    w.div_ceil(max_visual_width)
                }
            })
            .sum::<usize>()
            .max(1)
    };
    (content_lines + 1 + 2).clamp(3, 20) as u16
}

/// Return the input hint line showing available shortcuts.
/// Voice input hint is only shown on macOS where Fn key is available.
pub(super) fn input_hint_text() -> &'static str {
    if cfg!(target_os = "macos") {
        "  [Fn] 语音  [Alt+Enter] 换行"
    } else {
        "  [Alt+Enter] 换行"
    }
}

pub(super) fn render_input(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.config.theme;

    // Determine border color based on state
    let border_color = if app.is_processing() || app.input.text.is_empty() {
        theme.border()
    } else {
        theme.primary()
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    // Prefix character - styled based on state
    let prefix = if app.is_processing() {
        ("⏳ ", Color::Rgb(113, 113, 122))
    } else {
        ("❯ ", theme.primary())
    };

    let lines: Vec<Line> = if app.is_processing() {
        vec![Line::from(Span::styled(
            format!("{}{}", prefix.0, app.input.text),
            Style::default().fg(prefix.1),
        ))]
    } else if app.input.text.is_empty() {
        vec![
            Line::from(Span::styled(
                format!("{}输入消息...", prefix.0),
                Style::default().fg(Color::Rgb(113, 113, 122)),
            )),
            Line::from(Span::styled(
                input_hint_text(),
                Style::default().fg(Color::Rgb(80, 80, 90)),
            )),
        ]
    } else {
        let mut result: Vec<Line> = app
            .input
            .text
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let p = if i == 0 { prefix.0 } else { "  " };
                Line::from(Span::styled(
                    format!("{}{}", p, line),
                    Style::default().fg(Color::Rgb(250, 250, 250)),
                ))
            })
            .collect();
        result.push(Line::from(Span::styled(
            input_hint_text(),
            Style::default().fg(Color::Rgb(80, 80, 90)),
        )));
        result
    };

    let input_widget = Paragraph::new(lines).block(input_block);

    f.render_widget(input_widget, area);

    // Set cursor position (only when not processing)
    if !app.is_processing() && !app.input.text.is_empty() {
        let input_before = &app.input.text[..app.input.cursor];
        let line_idx = input_before.matches('\n').count();
        let current_line_start = input_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let pos_in_line =
            unicode_width::UnicodeWidthStr::width(&input_before[current_line_start..]);
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix.0);
        let max_x = area.x + area.width.saturating_sub(2);
        let cursor_x = (area.x + 1 + prefix_width as u16 + pos_in_line as u16).min(max_x);
        let cursor_y = area.y + 1 + line_idx as u16;
        if cursor_y < area.y + area.height.saturating_sub(1) {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    } else if !app.is_processing() {
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix.0);
        let cursor_x = area.x + 1 + prefix_width as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
