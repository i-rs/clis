use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::{App, AppMode};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    render_title_bar(frame, chunks[0], app);
    render_main_area(frame, chunks[1], app);
    render_input_bar(frame, chunks[2], app);
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let dir = if app.current_dir.len() > 48 {
        format!("...{}", &app.current_dir[app.current_dir.len() - 45..])
    } else {
        app.current_dir.clone()
    };

    let text = Line::from(vec![
        Span::styled(" i-rs-code ", Style::default().fg(Color::White).bg(Color::Blue)),
        Span::styled(
            format!(" v{} ", app.version),
            Style::default().fg(Color::Cyan).bg(Color::Blue),
        ),
        Span::raw("  "),
        Span::styled(
            dir,
            Style::default().fg(Color::White).bg(Color::Blue),
        ),
        Span::raw(" "),
    ]);

    let bar = Paragraph::new(text).style(Style::default().bg(Color::Blue));
    frame.render_widget(bar, area);
}

fn render_main_area(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_chat(frame, chunks[0], app);
    render_status(frame, chunks[1], app);
}

fn render_chat(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);

    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.messages {
        match msg.role.as_str() {
            "user" => {
                lines.push(Line::from(vec![
                    Span::styled(" You ", Style::default().fg(Color::White).bg(Color::Blue)),
                ]));
                for line in msg.content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
            "assistant" => {
                lines.push(Line::from(vec![
                    Span::styled(" AI ", Style::default().fg(Color::White).bg(Color::Green)),
                ]));
                for line in msg.content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
            "tool" => {
                lines.push(Line::from(vec![
                    Span::styled(
                        " Tool ",
                        Style::default().fg(Color::Black).bg(Color::Yellow),
                    ),
                ]));
                let preview: String = msg.content.chars().take(200).collect();
                if preview.len() < msg.content.len() {
                    lines.push(Line::from(Span::raw(format!(" {}...", preview))));
                } else {
                    lines.push(Line::from(Span::raw(format!(" {}", preview))));
                }
            }
            _ => {
                for line in msg.content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
        }
        lines.push(Line::from(""));
    }

    if matches!(app.mode, AppMode::Waiting) {
        lines.push(Line::from(vec![
            Span::styled(" AI ", Style::default().fg(Color::White).bg(Color::Green)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                " thinking...",
                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let scroll = app.scroll_offset.min(max_scroll);

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .scroll((scroll as u16, 0));

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);

    frame.render_widget(block, area);

    let mut items = Vec::new();

    items.push(Line::from(vec![
        Span::styled("Dir ", Style::default().fg(Color::Gray)),
    ]));
    let dir = if app.current_dir.len() > inner.width as usize - 2 {
        format!(
            "..{}",
            &app.current_dir
                [app.current_dir.len().saturating_sub(inner.width as usize - 4)..]
        )
    } else {
        app.current_dir.clone()
    };
    items.push(Line::from(Span::raw(format!(" {}", dir))));
    items.push(Line::from(""));

    items.push(Line::from(vec![
        Span::styled("Version ", Style::default().fg(Color::Gray)),
        Span::raw(app.version.clone()),
    ]));
    items.push(Line::from(""));

    items.push(Line::from(vec![
        Span::styled("Tokens ", Style::default().fg(Color::Gray)),
        Span::raw(format!(
            "I: {}  O: {}",
            app.token_usage.input, app.token_usage.output
        )),
    ]));
    items.push(Line::from(""));

    let change_count = app.file_changes.len();
    let change_text = if change_count == 0 {
        "none".to_string()
    } else {
        format!("{} file(s)", change_count)
    };
    items.push(Line::from(vec![
        Span::styled("Changes ", Style::default().fg(Color::Gray)),
        Span::styled(
            change_text,
            if change_count > 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ]));

    if change_count > 0 {
        let mut changes: Vec<&String> = app.file_changes.iter().collect();
        changes.sort();
        for path in changes.iter().take(5) {
            let display = if path.len() > inner.width as usize - 4 {
                format!(
                    " ..{}",
                    &path[path.len().saturating_sub(inner.width as usize - 6)..]
                )
            } else {
                format!(" {}", path)
            };
            items.push(Line::from(Span::styled(
                display,
                Style::default().fg(Color::Yellow),
            )));
        }
        if change_count > 5 {
            items.push(Line::from(Span::styled(
                format!(" ... and {} more", change_count - 5),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
    items.push(Line::from(""));

    let mode_text = match app.mode {
        AppMode::Idle => "Idle",
        AppMode::Waiting => "Waiting...",
    };
    let mode_style = match app.mode {
        AppMode::Idle => Style::default().fg(Color::Green),
        AppMode::Waiting => Style::default().fg(Color::Yellow),
    };
    items.push(Line::from(vec![
        Span::styled("Mode ", Style::default().fg(Color::Gray)),
        Span::styled(mode_text, mode_style),
    ]));
    items.push(Line::from(""));

    items.push(Line::from(vec![
        Span::styled("Messages ", Style::default().fg(Color::Gray)),
        Span::raw(app.messages.len().to_string()),
    ]));

    let paragraph = Paragraph::new(Text::from(items));
    frame.render_widget(paragraph, inner);
}

fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let prompt_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let cursor_style = Style::default().fg(Color::Cyan);

    let text_before_cursor = &app.input[..app.cursor_pos];
    let text_after_cursor = &app.input[app.cursor_pos..];

    let left_line = Line::from(vec![
        Span::styled("> ", prompt_style),
        Span::raw(text_before_cursor),
        if matches!(app.mode, AppMode::Idle) {
            Span::styled("█", cursor_style)
        } else {
            Span::raw("")
        },
        Span::raw(text_after_cursor),
    ]);

    let right_label = match app.mode {
        AppMode::Idle => {
            if app.input.is_empty() {
                Span::styled(" ⚡ idle ", Style::default().fg(Color::DarkGray))
            } else {
                Span::styled(" ↵ send ", Style::default().fg(Color::Green))
            }
        }
        AppMode::Waiting => Span::styled(" ⏳ waiting...", Style::default().fg(Color::Yellow)),
    };

    let label_len = 14usize;
    let left_width = inner.width.saturating_sub(label_len as u16) as usize;

    let line = Line::from(vec![
        Span::raw(format!(
            "{:width$}",
            left_line.to_string(),
            width = left_width
        )),
        right_label.clone(),
    ]);

    let paragraph = Paragraph::new(Text::from(vec![line]));
    frame.render_widget(paragraph, inner);

    if matches!(app.mode, AppMode::Idle) {
        let cursor_x = (2 + app.cursor_pos) as u16;
        let cursor_x = cursor_x.min(inner.width.saturating_sub(1));
        frame.set_cursor_position((inner.x + cursor_x, inner.y));
    }
}
