use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::{App, AppMode};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .split(area);

    render_title_bar(frame, chunks[0], app);
    render_main_area(frame, chunks[1], app);
    render_input_bar(frame, chunks[2], app);

    if app.show_shortcuts {
        render_shortcuts_overlay(frame, area);
    }
}

fn render_shortcuts_overlay(frame: &mut Frame, area: Rect) {
    let w = 50.min(area.width.saturating_sub(4));
    let h = 16;
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    let overlay = Rect { x, y, width: w, height: h };

    frame.render_widget(Clear, overlay);

    let items = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ? / Esc       关闭此面板",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  Ctrl+D        打开 HTTP 调试面板",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  Enter         发送消息",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  Alt+Enter     换行",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  Esc / q       退出",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  PgUp / PgDn   滚动聊天",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  ↑ / ↓         逐行滚动",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  Tab           插入缩进 (2 spaces)",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "     Press any key to close",
            Style::default().fg(Color::Rgb(113, 113, 122)),
        )),
    ];

    let block = Block::default()
        .title(" ⌨ Keyboard Shortcuts ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(Text::from(items))
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, overlay);
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
        Span::styled(dir, Style::default().fg(Color::White).bg(Color::Blue)),
        Span::raw(" "),
    ]);

    let bar = Paragraph::new(text).style(Style::default().bg(Color::Blue));
    frame.render_widget(bar, area);
}

fn render_main_area(frame: &mut Frame, area: Rect, app: &App) {
    if app.show_debug {
        render_debug_panel(frame, area, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_chat(frame, chunks[0], app);
    render_status(frame, chunks[1], app);
}

fn render_debug_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(format!(" HTTP Debug ({} logs) ", crate::debug::log_count()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let logs = crate::debug::get_log();
    if logs.is_empty() {
        let text = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "  No HTTP requests yet.",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  [Ctrl+L] clear  [Esc/Ctrl+D] close",
                Style::default().fg(Color::Rgb(80, 80, 90)),
            )),
        ]));
        frame.render_widget(text, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    let max_scroll = logs.len().saturating_sub(inner.height as usize / 5);
    let scroll = app.debug_scroll.min(max_scroll);

    for entry in logs.iter().rev().skip(scroll) {
        let status_color = if entry.response_status == 200 {
            Color::Green
        } else if entry.response_status >= 400 {
            Color::Red
        } else {
            Color::Yellow
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", entry.time_short()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!(" POST {}", entry.path()),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),
            Span::styled(
                entry.status_label(),
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({}.{:03}s)", entry.duration_ms / 1000, entry.duration_ms % 1000),
                Style::default().fg(Color::Gray),
            ),
        ]));

        // Request body preview
        let req_preview: String = entry.request_body.chars().take(inner.width.saturating_sub(4) as usize).collect();
        for line in req_preview.lines().take(2) {
            lines.push(Line::from(Span::styled(
                format!("  >> {}", line),
                Style::default().fg(Color::Rgb(100, 180, 100)),
            )));
        }

        // Response body preview
        let res_preview: String = entry.response_body_preview.chars().take(inner.width.saturating_sub(4) as usize).collect();
        for line in res_preview.lines().take(3) {
            lines.push(Line::from(Span::styled(
                format!("  << {}", line),
                Style::default().fg(Color::Rgb(180, 150, 100)),
            )));
        }

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .scroll((0, 0));
    frame.render_widget(paragraph, inner);
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
                    Span::styled(" Tool ", Style::default().fg(Color::Black).bg(Color::Yellow)),
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

    // Render streaming state
    if let Some(ref s) = app.streaming {
        lines.push(Line::from(vec![
            Span::styled(" AI ", Style::default().fg(Color::White).bg(Color::Green)),
        ]));

        if !s.content.is_empty() {
            for line in s.content.lines() {
                lines.push(Line::from(Span::raw(format!(" {}", line))));
            }
        }

        if let Some(ref tool) = s.current_tool {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" [Tool] {} running...", tool.name),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]));
            let preview: String = tool.args.chars().take(inner.width.saturating_sub(4) as usize).collect();
            for line in preview.lines() {
                lines.push(Line::from(Span::styled(
                    format!("   {}", line),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        for tool in &s.tool_calls {
            lines.push(Line::from(""));
            let status = if tool.result.is_some() { "done" } else { "running..." };
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" [Tool] {} {}", tool.name, status),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]));
            if let Some(ref result) = tool.result {
                let preview: String = result.chars().take(300).collect();
                for line in preview.lines().take(6) {
                    lines.push(Line::from(Span::styled(
                        format!("   {}", line),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                if preview.len() < result.len() || result.lines().count() > 6 {
                    lines.push(Line::from(Span::styled(
                        "   ... (truncated)",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }

        if s.current_tool.is_none() && !s.content.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(" ▊", Style::default().fg(Color::Green)),
            ]));
        } else if s.current_tool.is_none() && s.content.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(
                    " thinking...",
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                ),
            ]));
        }
    }

    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let scroll = app.scroll_offset.min(max_scroll);

    // Show hidden message count
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let hidden_msgs = app.messages.len().saturating_sub(1);
    if scroll > 0 || (hidden_msgs > 0 && lines.len() > inner.height as usize) {
        block = block.title(format!(" ↑ {} 条历史消息 ", hidden_msgs));
        block = block.title_alignment(Alignment::Center);
    }

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
    items.push(Line::from(""));

    // Debug log count
    let debug_count = crate::debug::log_count();
    items.push(Line::from(vec![
        Span::styled("HTTP Log ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} entries", debug_count),
            Style::default().fg(if debug_count > 0 { Color::Cyan } else { Color::DarkGray }),
        ),
    ]));
    items.push(Line::from(Span::styled(
        "  [Ctrl+D] open  [Ctrl+L] clear",
        Style::default().fg(Color::Rgb(80, 80, 90)),
    )));
    items.push(Line::from(""));

    // Active streaming info
    if let Some(ref s) = app.streaming {
        let token_count = s.content.len();
        let tool_count = s.tool_calls.len();
        items.push(Line::from(vec![
            Span::styled("Streaming ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} tokens, {} tools", token_count, tool_count),
                Style::default().fg(Color::Cyan),
            ),
        ]));
        if s.current_tool.is_some() {
            items.push(Line::from(vec![
                Span::styled("▸ ", Style::default().fg(Color::Yellow)),
                Span::styled("tool executing...", Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    let paragraph = Paragraph::new(Text::from(items));
    frame.render_widget(paragraph, inner);
}

fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let border_color = if matches!(app.mode, AppMode::Waiting) {
        Color::DarkGray
    } else if app.input.is_empty() {
        Color::Rgb(80, 80, 90)
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let prefix = "❯ ";

    let lines: Vec<Line> = if matches!(app.mode, AppMode::Waiting) {
        vec![Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(Color::Rgb(113, 113, 122))),
            Span::styled(&app.input, Style::default().fg(Color::Rgb(113, 113, 122))),
        ])]
    } else if app.input.is_empty() {
        vec![
            Line::from(Span::styled(
                format!("{}输入消息...", prefix),
                Style::default().fg(Color::Rgb(113, 113, 122)),
            )),
            Line::from(Span::styled(
                "  [Enter] 发送  [Esc] 退出  [?] 帮助",
                Style::default().fg(Color::Rgb(80, 80, 90)),
            )),
        ]
    } else {
        let mut result: Vec<Line> = app.input.lines().enumerate().map(|(i, line)| {
            let p = if i == 0 { prefix } else { "  " };
            Line::from(Span::styled(
                format!("{}{}", p, line),
                Style::default().fg(Color::Rgb(250, 250, 250)),
            ))
        }).collect();
        result.push(Line::from(Span::styled(
            "  [Enter] 发送  [?] 帮助",
            Style::default().fg(Color::Rgb(80, 80, 90)),
        )));
        result
    };

    let input_widget = Paragraph::new(lines).block(Block::default());
    frame.render_widget(input_widget, inner);

    if matches!(app.mode, AppMode::Idle) && !app.input.is_empty() {
        let input_before = &app.input[..app.cursor_pos];
        let line_idx = input_before.matches('\n').count();
        let current_line_start = input_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let pos_in_line = unicode_width::UnicodeWidthStr::width(&input_before[current_line_start..]);
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
        let cursor_x = inner.x + 1 + prefix_width as u16 + pos_in_line as u16;
        let cursor_y = inner.y + line_idx as u16;
        frame.set_cursor_position((cursor_x, cursor_y));
    } else if matches!(app.mode, AppMode::Idle) {
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
        let cursor_x = inner.x + 1 + prefix_width as u16;
        let cursor_y = inner.y;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
