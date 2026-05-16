use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
    // Available text width (minus indentation)
    let text_width = (area.width as usize).saturating_sub(4).max(20);
    // Subtract 1 line for the top border
    let area_lines = (area.height as usize).saturating_sub(1).max(1);

    // Build items from the end, skipping scroll_offset messages
    let mut items: Vec<ListItem> = Vec::new();
    let mut skipped = 0usize;
    let mut lines_used = 0usize;

    for msg in app.messages.iter().rev() {
        if skipped < app.scroll_offset {
            skipped += 1;
            continue;
        }
        let h = message_line_count(msg, text_width);
        if lines_used + h > area_lines && !items.is_empty() {
            break;
        }
        lines_used += h;
        items.push(build_message_item(msg, text_width));
    }
    items.reverse();

    // Show an indicator when scrolled up
    let at_bottom = app.scroll_offset == 0;

    let mut block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    if !at_bottom && !items.is_empty() {
        block = block.title(" ↑ 滚动浏览历史 ↑ ");
        block = block.title_alignment(ratatui::layout::Alignment::Center);
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// Show a processing/thinking indicator between chat and input
fn render_processing(f: &mut Frame, area: Rect, app: &App) {
    if !app.is_processing() || app.status_text.is_empty() {
        return;
    }
    let dots = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let frame = (app.messages.len() + app.tool_call_count) % dots.len();
    let spinner = dots[frame];

    let label = Line::from(Span::styled(
        format!(" {}  {}", spinner, app.status_text),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, area);
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
        let visible_cursor = unicode_width::UnicodeWidthStr::width(&app.input[..app.input_cursor]);
        let cursor_x = area.x + 1 + prefix_width as u16 + visible_cursor as u16;
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

/// Format a JSON CLI result into display lines.
/// Returns (lines, was_json) — empty lines + false means it wasn't JSON.
fn format_json_result(result: &str, max_width: usize) -> (Vec<Line<'static>>, bool) {
    let val = match serde_json::from_str::<serde_json::Value>(result) {
        Ok(v) => v,
        Err(_) => return (vec![], false),
    };

    let mut lines: Vec<Line<'static>> = Vec::new();

    // List response: { data: [...], meta: { count: N } }
    if let Some(data) = val.get("data").and_then(|d| d.as_array()) {
        // Show count from meta
        if let Some(count) = val
            .get("meta")
            .and_then(|m| m.get("count"))
            .and_then(|c| c.as_u64())
        {
            lines.push(Line::from(Span::styled(
                format!("   ─── {} records ───", count),
                Style::default().fg(Color::DarkGray),
            )));
        }

        if data.is_empty() {
            lines.push(Line::from(Span::styled(
                "   (empty)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for item in data {
                if let Some(obj) = item.as_object() {
                    // Build compact key: value line from object fields
                    let parts: Vec<String> = obj
                        .iter()
                        .filter(|(k, _)| {
                            !k.contains("created_at")
                                && !k.contains("updated_at")
                                && *k != "unit"
                        })
                        .map(|(k, v)| {
                            let v_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                _ => format!("{}", v),
                            };
                            format!("{}: {}", k, v_str)
                        })
                        .collect();
                    let text = parts.join("  ·  ");
                    for wrapped in wrap_text(&text, max_width.saturating_sub(4)) {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", wrapped),
                            Style::default().fg(Color::White),
                        )));
                    }
                } else {
                    let text = format!("{}", item);
                    for wrapped in wrap_text(&text, max_width.saturating_sub(4)) {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", wrapped),
                            Style::default().fg(Color::White),
                        )));
                    }
                }
            }
        }
        return (lines, true);
    }

    // Single item response: { success: true, data: { ... } }
    if val.get("data").and_then(|d| d.as_object()).is_some() {
        if let Some(obj) = val.get("data").and_then(|d| d.as_object()) {
            let parts: Vec<String> = obj
                .iter()
                .filter(|(k, _)| {
                    !k.contains("created_at") && !k.contains("updated_at") && *k != "unit"
                })
                .map(|(k, v)| {
                    let v_str = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => format!("{}", v),
                    };
                    format!("{}: {}", k, v_str)
                })
                .collect();
            let text = parts.join("  ·  ");
            for wrapped in wrap_text(&text, max_width.saturating_sub(4)) {
                lines.push(Line::from(Span::styled(
                    format!("   {}", wrapped),
                    Style::default().fg(Color::White),
                )));
            }
        }
        return (lines, true);
    }

    // Simple success response: { success: true } (no data field)
    if val.get("success").and_then(|s| s.as_bool()) == Some(true) {
        lines.push(Line::from(Span::styled(
            "   ✓ success",
            Style::default().fg(Color::Green),
        )));
        return (lines, true);
    }

    // Fallback: show compact JSON
    let text = format!("{}", val);
    for wrapped in wrap_text(&text, max_width.saturating_sub(3)) {
        lines.push(Line::from(Span::styled(
            format!("   {}", wrapped),
            Style::default().fg(Color::DarkGray),
        )));
    }
    (lines, true)
}

/// Wrap text to fit within max_width columns (using Unicode-aware width).
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if unicode_width::UnicodeWidthStr::width(line) <= max_width {
            lines.push(line.to_string());
            continue;
        }
        let mut current = String::new();
        let mut current_w = 0;
        for word in line.split(' ') {
            let word_w = unicode_width::UnicodeWidthStr::width(word);
            let separator = if current.is_empty() { 0 } else { 1 };
            if current_w + separator + word_w > max_width && !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_w = 0;
            }
            if !current.is_empty() {
                current.push(' ');
                current_w += 1;
            }
            current.push_str(word);
            current_w += word_w;
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

/// Estimate the number of rendered lines a message occupies.
fn message_line_count(msg: &Message, text_width: usize) -> usize {
    match msg {
        Message::User { text } => {
            // header + wrapped lines + trailing blank
            1 + wrapped_line_count(text, text_width) + 1
        }
        Message::Assistant { text } if text.is_empty() => {
            // header + "..." + trailing blank
            1 + 1 + 1
        }
        Message::Assistant { text } => {
            // header + wrapped lines + trailing blank
            1 + wrapped_line_count(text, text_width) + 1
        }
        Message::ToolCall {
            name,
            args,
            result,
        } => {
            let mut lines = 1; // header
            // optional explanation line
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
                if name == "i_rs" && val.get("explanation").and_then(|v| v.as_str()).is_some() {
                    lines += 1;
                }
            }
            // result lines (at least 1 if non-empty)
            if !result.is_empty() {
                // Most results are JSON → each data item is roughly 1-3 lines
                lines += wrapped_line_count(result, text_width.saturating_sub(3)).max(1);
            }
            lines
        }
        Message::Error { text } => {
            // header + wrapped lines + trailing blank
            1 + wrapped_line_count(text, text_width) + 1
        }
    }
}

/// Estimate how many lines a block of text wraps to.
fn wrapped_line_count(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count();
    }
    text.lines()
        .map(|line| {
            let w = unicode_width::UnicodeWidthStr::width(line);
            if w == 0 {
                1
            } else {
                (w + max_width - 1) / max_width
            }
        })
        .sum()
}

/// Build a ListItem widget from a Message.
fn build_message_item(msg: &Message, text_width: usize) -> ListItem<'static> {
    match msg {
        Message::User { text } => {
            let mut lines = vec![
                Line::from(Span::styled(
                    "  You:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
            ];
            for wrapped in wrap_text(text, text_width) {
                lines.push(Line::from(Span::styled(
                    format!("   {}", wrapped),
                    Style::default().fg(Color::White),
                )));
            }
            lines.push(Line::from(Span::raw("")));
            ListItem::new(lines)
        }
        Message::Assistant { text } => {
            let mut lines = vec![Line::from(Span::styled(
                "  Claw:",
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
                for wrapped in wrap_text(text, text_width) {
                    lines.push(Line::from(Span::styled(
                        format!("   {}", wrapped),
                        Style::default().fg(Color::White),
                    )));
                }
            }
            lines.push(Line::from(Span::raw("")));
            ListItem::new(lines)
        }
        Message::ToolCall {
            name,
            args,
            result,
        } => {
            let mut lines = Vec::new();

            let (header, detail) =
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
                    if name == "i_rs" {
                        let tool = val.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
                        let cmd = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
                        let explanation = val.get("explanation").and_then(|v| v.as_str());
                        (
                            format!(" ⚡ i-rs-{} {}", tool, cmd),
                            explanation.map(|s| s.to_string()),
                        )
                    } else if name == "search_tools" {
                        let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                        (format!(" 🔍 search: {}", q), None)
                    } else if name == "update_user_memory" {
                        (" 💾 记住用户信息".to_string(), None)
                    } else {
                        (format!(" ⚡ {}", name), None)
                    }
                } else {
                    (format!(" ⚡ {} {}", name, args), None)
                };

            lines.push(Line::from(Span::styled(
                header,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(exp) = &detail {
                lines.push(Line::from(Span::styled(
                    format!("   └─ {}", exp),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            if !result.is_empty() {
                let (json_lines, _) = format_json_result(result, text_width);
                if !json_lines.is_empty() {
                    lines.extend(json_lines);
                } else {
                    let result_display = if result.len() > 200 {
                        format!("{}…", utils::truncate(result, 200))
                    } else {
                        result.to_string()
                    };
                    for wrapped in wrap_text(&result_display, text_width.saturating_sub(3)) {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", wrapped),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
            }

            ListItem::new(lines)
        }
        Message::Error { text } => {
            let mut lines = vec![
                Line::from(Span::styled(
                    " ✗ Error:",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )),
            ];
            for wrapped in wrap_text(text, text_width) {
                lines.push(Line::from(Span::styled(
                    format!("   {}", wrapped),
                    Style::default().fg(Color::Red),
                )));
            }
            lines.push(Line::from(Span::raw("")));
            ListItem::new(lines)
        }
    }
}
