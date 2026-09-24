use crate::app::{App, AppMode};
use crate::tui::colors::*;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let prefix = "▎ ";
    let hint = Line::from(Span::styled(
        crate::tui::strings::STATUS_BAR,
        Style::new().fg(c_muted()),
    ));
    let lines: Vec<Line> = if matches!(app.mode, AppMode::Waiting) {
        vec![
            Line::from(vec![
                Span::styled("⏳ ", Style::new().fg(c_orange())),
                Span::styled(&app.input.content, Style::new().fg(c_dim())),
            ]),
            hint,
        ]
    } else if app.input.content.is_empty() {
        vec![
            Line::from(Span::styled(
                format!("{}{}", prefix, crate::tui::strings::INPUT_PLACEHOLDER),
                Style::new().fg(c_muted()),
            )),
            hint,
        ]
    } else {
        let mut result: Vec<Line> = app
            .input
            .content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let p = if i == 0 { prefix } else { "  " };
                Line::from(Span::styled(
                    format!("{}{}", p, line),
                    Style::new().fg(c_text()),
                ))
            })
            .collect();
        result.push(hint);
        result
    };

    let sep_color = c_border();
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(sep_color))
        .padding(ratatui::widgets::Padding::horizontal(1))
        .style(Style::new().bg(c_bg_input()));

    let input_widget = Paragraph::new(lines).block(input_block);
    frame.render_widget(input_widget, area);

    let prefix_width = unicode_width::UnicodeWidthStr::width(prefix) as u16;
    if matches!(app.mode, AppMode::Idle) && !app.input.content.is_empty() {
        let input_before = &app.input.content[..app.input.cursor_pos];
        let line_idx = input_before.matches('\n').count();
        let current_line_start = input_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let pos_in_line =
            unicode_width::UnicodeWidthStr::width(&input_before[current_line_start..]);
        let cursor_x = area.x + 1 + prefix_width + pos_in_line as u16;
        let cursor_y = area.y + 1 + line_idx as u16;
        frame.set_cursor_position((cursor_x, cursor_y));
    } else if matches!(app.mode, AppMode::Idle) {
        let cursor_x = area.x + 1 + prefix_width;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

pub fn filtered_slash_commands(app: &App) -> Vec<&'static crate::tui::slash_command::CmdHelp> {
    let partial = app
        .input
        .content
        .trim()
        .strip_prefix('/')
        .unwrap_or("")
        .to_lowercase();
    crate::tui::slash_command::COMMANDS
        .iter()
        .filter(|c| c.name.starts_with(&partial))
        .collect()
}

pub fn render_slash_picker(frame: &mut Frame, area: Rect, app: &App) {
    let commands = filtered_slash_commands(app);
    if commands.is_empty() {
        return;
    }

    let visible_rows = area.height.saturating_sub(2) as usize;
    let total = commands.len();

    let selected = app.slash_selected.min(total.saturating_sub(1));
    let (start, end) = if total <= visible_rows || visible_rows == 0 {
        (0usize, total.min(visible_rows))
    } else {
        let half = visible_rows / 2;
        let s = if selected <= half {
            0
        } else if selected + half >= total {
            total - visible_rows
        } else {
            selected - half
        };
        (s, s + visible_rows)
    };

    let mut items: Vec<Line> = Vec::with_capacity(end - start + 1);
    if start > 0 {
        items.push(Line::from(Span::styled(
            format!("  ⋮  ({} more above)", start),
            Style::new().fg(c_muted()),
        )));
    }
    for (i, cmd) in commands.iter().enumerate().take(end).skip(start) {
        let is_selected = i == selected;
        let marker = if is_selected { "▌" } else { " " };
        let name_style = if is_selected {
            Style::new().fg(c_accent()).bold()
        } else {
            Style::new().fg(c_text())
        };
        let args = if cmd.args.is_empty() {
            String::new()
        } else {
            format!(" {}", cmd.args)
        };

        items.push(Line::from(vec![
            Span::styled(
                marker,
                if is_selected {
                    Style::new().fg(c_accent())
                } else {
                    Style::new().fg(c_muted())
                },
            ),
            Span::styled(format!("/{}{}", cmd.name, args), name_style),
            Span::raw("  "),
            Span::styled(cmd.desc, Style::new().fg(c_dim())),
        ]));
    }
    if end < total {
        items.push(Line::from(Span::styled(
            format!("  ⋮  ({} more below)", total - end),
            Style::new().fg(c_muted()),
        )));
    }

    frame.render_widget(ratatui::widgets::Clear, area);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(c_border_active()))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0))
        .style(Style::new().bg(c_bg_surface()));

    let paragraph = Paragraph::new(ratatui::text::Text::from(items)).block(block);
    frame.render_widget(paragraph, area);
}
