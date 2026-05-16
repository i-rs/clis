use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Message};

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

    if app.show_sidebar && !app.is_processing() {
        // Split chat area horizontally when sidebar is open
        let chat_side = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Percentage(35),
            ])
            .split(layout[1]);
        render_chat(f, chat_side[0], app);
        render_sidebar(f, chat_side[1], app);
    } else {
        render_chat(f, layout[1], app);
    }

    render_processing(f, layout[2], app);
    render_input(f, layout[3], app);
    render_status(f, layout[4], app);

    // Session list overlay (rendered on top of everything)
    if app.show_session_list {
        render_session_list(f, area, app);
    }

    // Request body overlay (rendered on top of everything)
    if let Some(idx) = app.sidebar_body_idx {
        if let Some(log) = app.http_logs.get(idx) {
            render_request_body(
                f,
                area,
                &log.request_body,
                idx,
                app.http_logs.len(),
                app.sidebar_body_scroll,
            );
        }
    }
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    // Full-width background
    f.render_widget(
        ratatui::widgets::Block::default()
            .style(Style::default().bg(Color::Rgb(25, 25, 42))),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    // App name — standout
    spans.push(Span::styled(
        " ✦ i-rs-claw",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));

    if app.is_processing() {
        // Processing indicator
        spans.push(Span::styled(
            format!("  ⏳ {} ", app.status_text),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    } else {
        // Tagline
        spans.push(Span::styled(
            "  个人数据智能助理",
            Style::default().fg(Color::Rgb(180, 180, 200)),
        ));
    }

    // Right-aligned model name (padded to fill width)
    let model_text = format!(" {} ", app.config.model);
    let model_text_ref: &str = &model_text;
    let model_width = unicode_width::UnicodeWidthStr::width(model_text_ref);
    let padding = (area.width as usize).saturating_sub(
        spans.iter().map(|s| {
            let content: &str = &*s.content;
            unicode_width::UnicodeWidthStr::width(content)
        }).sum::<usize>()
        + model_width
        + 2, // buffer
    );
    if padding > 0 {
        spans.push(Span::styled(
            " ".repeat(padding),
            Style::default(),
        ));
    }
    spans.push(Span::styled(
        model_text,
        Style::default()
            .fg(Color::Rgb(100, 100, 130))
            .add_modifier(Modifier::BOLD),
    ));

    let line = Line::from(spans);
    f.render_widget(line, area);
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
        .border_style(Style::default().fg(if at_bottom {
            Color::DarkGray
        } else {
            Color::Rgb(100, 120, 200)
        }));

    if !at_bottom && !items.is_empty() {
        let hidden = app.scroll_offset;
        block = block.title(format!(" ▲ {} 条历史消息 ", hidden));
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
        .border_style(Style::default().fg(if app.is_processing() {
            Color::DarkGray
        } else if app.input.is_empty() {
            Color::Rgb(80, 80, 100)
        } else {
            Color::Cyan
        }));

    let prefix = if app.is_processing() { "⏳ " } else { "❯ " };

    let (display_text, display_style) = if app.is_processing() {
        (format!("{}{}", prefix, app.input), Style::default().fg(Color::DarkGray))
    } else if app.input.is_empty() {
        (format!("{}输入消息...", prefix), Style::default().fg(Color::Rgb(80, 80, 100)))
    } else {
        (format!("{}{}", prefix, app.input), Style::default().fg(Color::White))
    };

    let input = Paragraph::new(Span::styled(display_text, display_style))
        .block(input_block);

    f.render_widget(input, area);

    // Set cursor position (only when not processing)
    if !app.is_processing() {
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix);
        let cursor_offset = if app.input.is_empty() {
            0
        } else {
            unicode_width::UnicodeWidthStr::width(&app.input[..app.input_cursor])
        };
        let cursor_x = area.x + 1 + prefix_width as u16 + cursor_offset as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let bg = if app.is_processing() {
        Color::Blue
    } else {
        Color::Rgb(30, 30, 46)
    };

    // Fill full-width background
    f.render_widget(
        ratatui::widgets::Block::default()
            .style(Style::default().bg(bg)),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    if app.is_processing() {
        // Processing state
        spans.push(Span::styled(
            format!(" ⏳ {} ", app.status_text),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
    } else {
        // Idle state — green dot + bold
        spans.push(Span::styled(
            " ● ",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "就绪 ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{} ", app.config.model),
            Style::default().fg(Color::Cyan),
        ));
    }

    // Separator
    spans.push(Span::styled(
        "│ ",
        Style::default().fg(Color::Rgb(80, 80, 100)),
    ));

    // Tool & message stats
    spans.push(Span::styled(
        format!("⚙ {} ", app.tool_call_count),
        Style::default().fg(Color::Cyan),
    ));
    spans.push(Span::styled(
        format!("💬 {} ", app.messages.len()),
        Style::default().fg(Color::Cyan),
    ));

    // Token usage
    if let Some(usage) = &app.token_usage {
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(Color::Rgb(80, 80, 100)),
        ));
        spans.push(Span::styled(
            format!("tok: {}p+{}c ", usage.prompt_tokens, usage.completion_tokens),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Keybindings (right side)
    spans.push(Span::styled(
        "│ ",
        Style::default().fg(Color::Rgb(80, 80, 100)),
    ));
    spans.push(Span::styled(
        "Ctrl+Q ",
        Style::default().fg(Color::Rgb(140, 140, 160)),
    ));
    spans.push(Span::styled(
        "Ctrl+N  ",
        Style::default().fg(Color::Rgb(140, 140, 160)),
    ));
    if !app.show_sidebar {
        spans.push(Span::styled(
            "Ctrl+R  ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
    }
    spans.push(Span::styled(
        "Ctrl+L",
        Style::default().fg(Color::Rgb(140, 140, 160)),
    ));

    let line = Line::from(spans);
    f.render_widget(line, area);
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

            items.push(ListItem::new(vec![Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    truncate_str(&session.title, (popup_width as usize).saturating_sub(8)),
                    style,
                ),
            ])]));
            
            // Session info: messages count and relative time
            items.push(ListItem::new(vec![Line::from(vec![
                Span::raw("      "),
                Span::styled(
                    format!("💬 {} · {}", session.message_count, relative_time(session.updated_at)),
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

/// Overlay showing the full request body JSON for a debug log entry.
fn render_request_body(
    f: &mut Frame,
    area: Rect,
    body_json: &str,
    idx: usize,
    total: usize,
    scroll: usize,
) {
    let popup_width = (area.width as f32 * 0.85) as u16;
    let popup_height = (area.height as f32 * 0.8) as u16;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Pretty-print the body JSON if possible
    let formatted = if let Ok(val) = serde_json::from_str::<serde_json::Value>(body_json) {
        serde_json::to_string_pretty(&val).unwrap_or_else(|_| body_json.to_string())
    } else {
        body_json.to_string()
    };

    let inner_w = (popup_width as usize).saturating_sub(4).max(20);
    // Visible content lines (popup height minus borders minus header)
    let visible_lines = (popup_height as usize).saturating_sub(4).max(1);

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        format!("  🔍 Request Body ({}/{})  [↑↓/scroll to browse | Esc to close]", idx + 1, total),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    )));

    // Build all JSON content lines first
    let mut content_lines: Vec<Line> = Vec::new();
    for line in formatted.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let wrapped = if unicode_width::UnicodeWidthStr::width(trimmed) > inner_w {
            wrap_text(trimmed, inner_w)
        } else {
            vec![trimmed.to_string()]
        };
        for w in wrapped {
            let color = if w.contains('"') && w.trim_start().starts_with('"') {
                // Key names
                Color::Green
            } else if w.contains('"') {
                // String values
                Color::Yellow
            } else if w.contains('{') || w.contains('}') {
                // Brackets
                Color::DarkGray
            } else {
                // Numbers, booleans, null
                Color::Cyan
            };
            content_lines.push(Line::from(Span::styled(
                format!("  {}", w),
                Style::default().fg(color),
            )));
        }
    }

    let total_content = content_lines.len();

    // Apply scroll offset
    let scroll = scroll.min(total_content.saturating_sub(visible_lines));
    let end = (scroll + visible_lines).min(total_content);
    if scroll > 0 {
        lines.push(Line::from(Span::styled(
            format!("  ↑ 还有 {} 行 ...", scroll),
            Style::default().fg(Color::Rgb(140, 140, 160)),
        )));
        // Adjust visible lines to account for this indicator
        let remaining = visible_lines.saturating_sub(1);
        let end2 = (scroll + remaining).min(total_content);
        for line in content_lines.iter().take(end2).skip(scroll) {
            lines.push(line.clone());
        }
    } else {
        for line in content_lines.iter().take(end).skip(scroll) {
            lines.push(line.clone());
        }
    }

    // Scroll indicator at bottom
    let more_below = end < total_content;
    if more_below {
        lines.push(Line::from(Span::styled(
            format!("  ↓ 还有 {} 行 ...", total_content - end),
            Style::default().fg(Color::Rgb(140, 140, 160)),
        )));
    }

    let list = List::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, popup_area);
}

fn render_sidebar(f: &mut Frame, area: Rect, app: &App) {
    // Sidebar block with border
    let block = Block::default()
        .borders(Borders::LEFT | Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 100)))
        .title(" 🔍 Debug ")
        .title_alignment(ratatui::layout::Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.http_logs.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            " (no requests)",
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(empty, inner);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();
    let max_lines = inner.height as usize;
    let side_width = inner.width as usize;

    for (i, log) in app.http_logs.iter().enumerate() {
        if items.len() >= max_lines {
            break;
        }

        let is_selected = i == app.sidebar_selected;
        let select_prefix = if is_selected { " ▶" } else { "  " };
        let select_fg = if is_selected {
            Color::Cyan
        } else {
            Color::White
        };

        let (status_icon, status_color) = if log.error.is_some() {
            ("✗", Color::Red)
        } else if log.status == 200 || log.status == 201 {
            ("✓", Color::Green)
        } else {
            ("!", Color::Yellow)
        };

        let duration_fmt = if log.duration_ms >= 1000 {
            format!("{:.1}s", log.duration_ms as f64 / 1000.0)
        } else {
            format!("{}ms", log.duration_ms)
        };

        // Count messages in request body
        let msg_count = if let Ok(v) =
            serde_json::from_str::<serde_json::Value>(&log.request_body)
        {
            v["messages"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0)
        } else {
            0
        };

        // Line 1: selection indicator + timestamp + status
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(select_prefix, Style::default().fg(select_fg).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" {} ", log.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} {}", status_icon, log.status),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 2: duration + model
            Line::from(vec![
                Span::styled(
                    format!(" {} ", duration_fmt),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    truncate_str(&log.model, side_width.saturating_sub(10)),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]));

        // Token stats line
        if log.prompt_tokens > 0 || log.completion_tokens > 0 {
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!(
                    "   {}p + {}c",
                    log.prompt_tokens, log.completion_tokens
                ),
                Style::default().fg(Color::Rgb(140, 140, 160)),
            ))]));
        }

        // Messages count & Enter hint
        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(
                format!("   📝 {} msgs", msg_count),
                Style::default().fg(Color::Rgb(140, 140, 160)),
            ),
            if is_selected {
                Span::styled(
                    "  <Enter>",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            },
        ])]));

        // Error detail line
        if let Some(err) = &log.error {
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!("   {}", truncate_str(err, side_width.saturating_sub(4))),
                Style::default().fg(Color::Red),
            ))]));
        }

        // Separator between entries
        items.push(ListItem::new(vec![Line::from(Span::styled(
            " ───",
            Style::default().fg(Color::Rgb(50, 50, 65)),
        ))]));
    }

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Convert a Unix timestamp to a localized relative time string.
fn relative_time(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = now.saturating_sub(ts);
    if diff < 60 {
        "刚刚".to_string()
    } else if diff < 3600 {
        format!("{}分钟前", diff / 60)
    } else if diff < 86400 {
        format!("{}小时前", diff / 3600)
    } else if diff < 2592000 {
        format!("{}天前", diff / 86400)
    } else {
        format!("{}月前", diff / 2592000)
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

/// Check if a string contains ANSI escape codes.
fn has_ansi(text: &str) -> bool {
    text.contains("\x1b[")
}

/// Parse ANSI-colored text into ratatui Lines with proper styling.
/// Strips ANSI codes and wraps text to fit max_width.
fn ansi_to_lines(text: &str, max_width: usize) -> Vec<Line<'static>> {
    let plain = strip_ansi(text);
    let wrapped = wrap_text(&plain, max_width.saturating_sub(3));

    // For each wrapped line, create a Line with colored Spans
    let mut lines = Vec::new();
    for w in &wrapped {
        let spans = parse_ansi_line(text, &plain, w, &wrapped);
        lines.push(Line::from(if spans.is_empty() {
            vec![Span::styled(
                format!("   {}", w),
                Style::default().fg(Color::White),
            )]
        } else {
            let mut result = vec![Span::raw("   ")];
            result.extend(spans);
            result
        }));
    }
    lines
}

/// Strip ANSI escape codes from a string.
fn strip_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.next() == Some('[') {
            // Skip until we find a letter (end of escape sequence)
            for esc_c in &mut chars {
                if esc_c.is_ascii_alphabetic() || esc_c == '~' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse ANSI codes from `raw` and produce Spans for the given `line` text.
/// `line` is a wrapped segment of the plain-text version.
fn parse_ansi_line(raw: &str, plain: &str, line: &str, _wrapped: &[String]) -> Vec<Span<'static>> {
    // Locate this line in the plain text
    let line_start = match plain.find(line) {
        Some(i) => i,
        None => return vec![],
    };

    // Build the segment: find the range in `raw` that corresponds to `line` in `plain`.
    // Map byte positions in `plain` to byte positions in `raw` (accounting for ANSI codes).
    let raw_bytes = raw.as_bytes();
    let plain_bytes = plain.as_bytes();

    // Find the byte in raw that corresponds to line_start in plain
    let mut raw_pos = 0usize;
    let mut plain_pos = 0usize;
    let mut raw_start = None;
    let mut raw_end = 0;

    while raw_pos < raw_bytes.len() && plain_pos < plain_bytes.len() {
        if raw_bytes[raw_pos] == b'\x1b' && raw_pos + 1 < raw_bytes.len() && raw_bytes[raw_pos + 1] == b'[' {
            // Skip ANSI sequence
            let mut esc_end = raw_pos + 2;
            while esc_end < raw_bytes.len() && !raw_bytes[esc_end].is_ascii_alphabetic() {
                esc_end += 1;
            }
            if esc_end < raw_bytes.len() {
                esc_end += 1; // skip the letter
            }
            raw_pos = esc_end;
            continue;
        }

        if raw_start.is_none() && plain_pos >= line_start {
            raw_start = Some(raw_pos);
        }

        if let Some(_start) = raw_start {
            let remaining = line.len() - (plain_pos - line_start);
            if plain_pos - line_start + remaining >= line.len() {
                raw_end = raw_pos + (plain_bytes[plain_pos..].len() - (plain_pos - line_start));
                // Approximate end: scan raw to find end of this line segment
                let target = plain_pos - line_start + line.len();
                if plain_pos >= target {
                    break;
                }
            }
        }

        raw_pos += 1;
        plain_pos += 1;
    }

    let raw_start = raw_start.unwrap_or(0);
    if raw_end <= raw_start {
        raw_end = raw.len();
    }

    // Now parse the ANSI slice and produce Spans
    let segment = &raw[raw_start..raw_end.min(raw.len())];
    let text_segment = &plain[line_start..line_start + line.len().min(plain.len() - line_start)];

    // If no ANSI in this segment, return plain white
    if !segment.contains("\x1b[") {
        return vec![Span::styled(
            text_segment.to_string(),
            Style::default().fg(Color::White),
        )];
    }

    // Parse ANSI SGR codes and build Spans
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut bold = false;
    let mut italic = false;
    let mut fg_color: Option<Color> = None;
    let mut current_text = String::new();
    let mut i = 0;
    let seg_bytes = segment.as_bytes();

    while i < seg_bytes.len() {
        if seg_bytes[i] == b'\x1b' && i + 1 < seg_bytes.len() && seg_bytes[i + 1] == b'[' {
            // Flush current text as a span
            if !current_text.is_empty() {
                let mut style = Style::default();
                if bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if italic {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if let Some(c) = fg_color {
                    style = style.fg(c);
                }
                spans.push(Span::styled(std::mem::take(&mut current_text), style));
            }

            // Parse the SGR code
            i += 2; // skip \x1b[
            let mut params = Vec::new();
            let mut num = 0i32;
            let mut has_num = false;

            while i < seg_bytes.len() {
                let c = seg_bytes[i] as char;
                if c == ';' {
                    params.push(num);
                    num = 0;
                    has_num = false;
                    i += 1;
                } else if c == 'm' {
                    if has_num || !params.is_empty() {
                        params.push(num);
                    }
                    if params.is_empty() {
                        params.push(0); // reset
                    }
                    // Apply SGR parameters
                    for p in &params {
                        match p {
                            0 => {
                                bold = false;
                                italic = false;
                                fg_color = None;
                            }
                            1 => bold = true,
                            3 => italic = true,
                            22 => bold = false,
                            23 => italic = false,
                            30 => fg_color = Some(Color::Black),
                            31 => fg_color = Some(Color::Red),
                            32 => fg_color = Some(Color::Green),
                            33 => fg_color = Some(Color::Yellow),
                            34 => fg_color = Some(Color::Blue),
                            35 => fg_color = Some(Color::Magenta),
                            36 => fg_color = Some(Color::Cyan),
                            37 => fg_color = Some(Color::White),
                            39 => fg_color = None,
                            90 => fg_color = Some(Color::Rgb(128, 128, 128)),
                            91 => fg_color = Some(Color::Rgb(255, 128, 128)),
                            92 => fg_color = Some(Color::Rgb(128, 255, 128)),
                            93 => fg_color = Some(Color::Rgb(255, 255, 128)),
                            94 => fg_color = Some(Color::Rgb(128, 128, 255)),
                            95 => fg_color = Some(Color::Rgb(255, 128, 255)),
                            96 => fg_color = Some(Color::Rgb(128, 255, 255)),
                            97 => fg_color = Some(Color::White),
                            _ => {}
                        }
                    }
                    i += 1;
                    break;
                } else if c.is_ascii_digit() {
                    num = num * 10 + (c as i32 - '0' as i32);
                    has_num = true;
                    i += 1;
                } else {
                    // Unknown code, skip to end
                    while i < seg_bytes.len() && seg_bytes[i] as char != 'm' {
                        i += 1;
                    }
                    if i < seg_bytes.len() {
                        i += 1;
                    }
                    break;
                }
            }
        } else {
            current_text.push(seg_bytes[i] as char);
            i += 1;
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        let mut style = Style::default();
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if let Some(c) = fg_color {
            style = style.fg(c);
        }
        spans.push(Span::styled(current_text, style));
    }

    spans
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
/// Strips ANSI codes for accurate width calculation.
fn wrapped_line_count(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count();
    }
    let clean = strip_ansi(text);
    clean.lines()
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
            ListItem::new(lines).style(Style::default().bg(Color::Rgb(35, 50, 45)))
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
                } else if has_ansi(result) {
                    for line in ansi_to_lines(result, text_width) {
                        lines.push(line);
                    }
                } else {
                    for wrapped in wrap_text(result, text_width.saturating_sub(3)) {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", wrapped),
                            Style::default().fg(Color::White),
                        )));
                    }
                }
            }

            ListItem::new(lines).style(Style::default().bg(Color::Rgb(28, 28, 35)))
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
            ListItem::new(lines).style(Style::default().bg(Color::Rgb(55, 30, 30)))
        }
    }
}
