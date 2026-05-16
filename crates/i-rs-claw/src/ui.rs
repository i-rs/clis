use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Message};
use crate::utils;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(1),    // Chat area
            Constraint::Length(1), // Processing indicator
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    render_title(f, layout[0], app);
    render_chat(f, layout[1], app);
    render_processing(f, layout[2], app);
    render_input(f, layout[3], app);
    render_status(f, layout[4], app);

    // Session list overlay (rendered on top of everything)
    if app.show_session_list {
        render_session_list(f, area, app);
    }
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title_text = if app.is_processing() {
        format!(
            " ✦ i-rs-claw  ⏳ {}  |  {}",
            app.status_text, app.config.model
        )
    } else {
        format!(
            " ✦ i-rs-claw  个人数据智能助理  |  {}",
            app.config.model
        )
    };

    let title = Line::from(Span::styled(
        title_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(title, area);
}

fn render_chat(f: &mut Frame, area: Rect, app: &App) {
    // Auto-scroll: show latest messages that fit in the chat area
    // Estimate ~3 lines per message (header + content + blank)
    let max_visible = (area.height as usize).max(1);
    let start = app.messages.len().saturating_sub(max_visible);

    let items: Vec<ListItem> = app
        .messages[start..]
        .iter()
        .map(|msg| match msg {
            Message::User { text } => {
                let lines = vec![
                    Line::from(Span::styled(
                        " ◆ You:",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("   {}", text),
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::raw("")),
                ];
                ListItem::new(lines)
            }
            Message::Assistant { text } => {
                let mut lines = vec![Line::from(Span::styled(
                    " ◇ Claw:",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))];
                if text.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "   ...",
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("   {}", text),
                        Style::default().fg(Color::White),
                    )));
                }
                lines.push(Line::from(Span::raw("")));
                ListItem::new(lines)
            }
            Message::ToolCall { name, args, result } => {
                let call_str = format!("{} {}", name, args);
                let line = if call_str.len() > 60 {
                    format!("  {}...", utils::truncate(&call_str, 57))
                } else {
                    format!("  {}", call_str)
                };
                ListItem::new(vec![
                    Line::from(Span::styled(
                        format!(" ⚡ {}", line),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::DIM),
                    )),
                    Line::from(Span::styled(
                        format!("   ↳ {}", result),
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
            }
            Message::Error { text } => {
                let lines = vec![
                    Line::from(Span::styled(
                        " ✗ Error:",
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("   {}", text),
                        Style::default().fg(Color::Red),
                    )),
                    Line::from(Span::raw("")),
                ];
                ListItem::new(lines)
            }
        })
        .collect();

    let chat_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let list = List::new(items).block(chat_block);
    f.render_widget(list, area);
}

/// Show a processing/thinking indicator between chat and input
fn render_processing(f: &mut Frame, area: Rect, app: &App) {
    if app.is_processing() && !app.status_text.is_empty() {
        // Render a subtle progress bar/indicator
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::SLOW_BLINK),
            )
            .label(app.status_text.clone())
            .use_unicode(true)
            .percent(50);
        f.render_widget(gauge, area);
    }
}

fn render_input(f: &mut Frame, area: Rect, app: &App) {
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if app.is_processing() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan)
        });

    let prefix = if app.is_processing() {
        "⏳ "
    } else {
        "❯ "
    };

    let input_style = if app.is_processing() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(Span::styled(
        format!("{}{}", prefix, app.input),
        input_style,
    ))
    .block(input_block);

    f.render_widget(input, area);

    // Set cursor position (only when not processing)
    if !app.is_processing() {
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
        let cursor_x = area.x + 1 + prefix_width as u16 + app.input_cursor as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    // Format token usage display
    let token_str = match &app.token_usage {
        Some(usage) => format!("tok: {}p+{}c", usage.prompt_tokens, usage.completion_tokens),
        None => String::new(),
    };

    let status_info = if app.is_processing() {
        format!(
            " ⏳ {} | tools: {} | msgs: {} | {} | Ctrl+Q quit | Ctrl+N new | Ctrl+L sessions",
            app.status_text,
            app.tool_call_count,
            app.messages.len(),
            token_str,
        )
    } else {
        format!(
            " ● 就绪 | {} | tools: {} | msgs: {} | {} | Ctrl+Q quit | Ctrl+N new | Ctrl+L sessions",
            app.config.model,
            app.tool_call_count,
            app.messages.len(),
            token_str,
        )
    };

    let bg = if app.is_processing() {
        Color::Blue
    } else {
        Color::DarkGray
    };

    let status = Line::from(Span::styled(
        status_info,
        Style::default()
            .fg(Color::White)
            .bg(bg)
            .add_modifier(Modifier::DIM),
    ));
    f.render_widget(status, area);
}

/// Centered overlay showing the session list for switching conversations.
fn render_session_list(f: &mut Frame, area: Rect, app: &App) {
    // Calculate popup dimensions
    let popup_width = (area.width as f32 * 0.7) as u16;
    let popup_height = (area.height as f32 * 0.6) as u16;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Items: title + each session
    let empty = app.session_list.is_empty();

    let mut items: Vec<ListItem> = Vec::new();

    // Header
    items.push(ListItem::new(vec![
        Line::from(Span::styled(
            " 会话列表",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " ────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )),
    ]));

    if empty {
        items.push(ListItem::new(vec![Line::from(Span::styled(
            " 暂无会话",
            Style::default().fg(Color::DarkGray),
        ))]));
    } else {
        for (i, session) in app.session_list.iter().enumerate() {
            let selected = i == app.session_list_index;
            let prefix = if selected { " ▶ " } else { "    " };
            let style = if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Format date from timestamp
            let time = chrono::DateTime::from_timestamp(session.created_at, 0)
                .map(|dt| dt.format("%m-%d %H:%M").to_string())
                .unwrap_or_default();

            items.push(ListItem::new(vec![Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    truncate_str(&session.title, (popup_width as usize).saturating_sub(8)),
                    style,
                ),
            ])]));

            // Session info: messages, time, and ID
            let id_short = if session.id.len() > 8 {
                format!("{}…", &session.id[..8])
            } else {
                session.id.clone()
            };
            items.push(ListItem::new(vec![Line::from(vec![
                Span::raw("      "),
                Span::styled(
                    format!("{} msgs | {} | id: {}", session.message_count, time, id_short),
                    Style::default().fg(Color::DarkGray),
                ),
            ])]));
        }
    }

    // Footer
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        if empty {
            " Ctrl+N 新建会话  Ctrl+L 关闭"
        } else {
            " ↑↓ 选择  Enter 切换  Ctrl+N 新建  Ctrl+L 关闭"
        },
        Style::default().fg(Color::DarkGray),
    ))]));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, popup_area);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
