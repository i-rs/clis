use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::utils;
use crate::app::App;

pub(super) fn render_sidebar(f: &mut Frame, area: Rect, app: &App) {
    // Sidebar block with border - refined styling
    let theme = &app.config.theme;
    let block = Block::default()
        .borders(Borders::LEFT | Borders::TOP)
        .border_style(Style::default().fg(theme.border()))
        .title(" ⬡ Debug ")
        .title_alignment(ratatui::layout::Alignment::Center)
        .border_type(ratatui::widgets::BorderType::Rounded);

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

        let is_selected = i == app.overlay.sidebar_selected;
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

        // Count messages in request body (pre-computed in HttpLog)
        let msg_count = log.msg_count;

        // Line 1: selection indicator + timestamp + status
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(
                    select_prefix,
                    Style::default().fg(select_fg).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", log.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} {}", status_icon, log.status),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Line 2: duration + model
            Line::from(vec![
                Span::styled(
                    format!(" {} ", duration_fmt),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    utils::truncate_str(&log.model, side_width.saturating_sub(10)),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]));

        // Token stats line
        if log.prompt_tokens > 0 || log.completion_tokens > 0 {
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!("   {}p + {}c", log.prompt_tokens, log.completion_tokens),
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
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            },
        ])]));

        // Error detail line
        if let Some(err) = &log.error {
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!(
                    "   {}",
                    utils::truncate_str(err, side_width.saturating_sub(4))
                ),
                Style::default().fg(Color::Red),
            ))]));
        }
    }

    let list = List::new(items);
    f.render_widget(list, inner);
}

/// Centered overlay showing the session list for switching conversations.
pub(super) fn render_session_list(f: &mut Frame, area: Rect, app: &App) {
    // Calculate popup dimensions
    let popup_width = (area.width as f32 * 0.7) as u16;
    let popup_height = (area.height as f32 * 0.6) as u16;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Filter sessions by search text
    let filtered = app.overlay.filtered_sessions();

    let empty = filtered.is_empty();
    let theme_primary = app.config.theme.primary();

    let mut items: Vec<ListItem> = Vec::new();

    // Header with optional search bar
    let search_display = if app.overlay.session_search_mode {
        let search_line = if app.overlay.session_search.is_empty() {
            " 🔍 输入搜索关键词…".to_string()
        } else {
            format!(" 🔍 {}", app.overlay.session_search)
        };
        items.push(ListItem::new(vec![Line::from(Span::styled(
            search_line,
            Style::default()
                .fg(theme_primary)
                .add_modifier(Modifier::BOLD),
        ))]));
        " 会话列表"
    } else {
        " 会话列表"
    };

    items.push(ListItem::new(vec![
        Line::from(Span::styled(
            search_display,
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
            if app.overlay.session_search.is_empty() {
                " 暂无会话"
            } else {
                " 未找到匹配会话"
            },
            Style::default().fg(Color::DarkGray),
        ))]));
    } else {
        for (i, session) in filtered.iter().enumerate() {
            let selected = i == app.overlay.session_list_index;
            let prefix = if selected { " ▶ " } else { "    " };
            let style = if selected {
                Style::default()
                    .fg(theme_primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            items.push(ListItem::new(vec![Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    utils::truncate_str(&session.title, (popup_width as usize).saturating_sub(8)),
                    style,
                ),
            ])]));

            // Session info: messages count and relative time
            items.push(ListItem::new(vec![Line::from(vec![
                Span::raw("      "),
                Span::styled(
                    format!(
                        "💬 {} · {}",
                        session.message_count,
                        utils::relative_time(session.updated_at)
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ])]));
        }
    }

    // Rename input field
    if !app.overlay.session_rename_buf.is_empty() {
        items.push(ListItem::new(vec![Line::from(Span::styled(
            " ────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        ))]));
        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(
                " ✏️ 重命名: ",
                Style::default()
                    .fg(app.config.theme.primary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.overlay.session_rename_buf.as_str(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ▌", Style::default().fg(Color::Yellow)),
        ])]));
    }

    // Delete confirmation
    if app.overlay.session_confirm_delete {
        items.push(ListItem::new(vec![Line::from(Span::styled(
            " ────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        ))]));
        items.push(ListItem::new(vec![Line::from(Span::styled(
            " ⚠ 确认删除此会话? (y = 确认, n = 取消)",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))]));
    }

    // Footer
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        if app.overlay.session_confirm_delete {
            " 确认删除? (y/n)"
        } else if !app.overlay.session_rename_buf.is_empty() {
            " 输入新名称  Enter 确认  Esc 取消"
        } else if empty && !app.overlay.session_search_mode {
            " Ctrl+N 新建会话  Ctrl+L 关闭"
        } else if app.overlay.session_search_mode {
            " 输入搜索  Esc 关闭搜索  Enter 切换"
        } else {
            " ↑↓ 选择  Enter 切换  / 搜索  Ctrl+R 重命名  Ctrl+D 删除  Ctrl+N 新建  Ctrl+L 关闭"
        },
        Style::default().fg(Color::DarkGray),
    ))]));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme_primary)),
    );

    f.render_widget(list, popup_area);
}

/// Centered overlay showing the agent picker.
pub(super) fn render_agent_picker(f: &mut Frame, area: Rect, app: &App) {
    let popup_width = 40u16.min(area.width.saturating_sub(4));
    let popup_height = (app.overlay.agent_list.len() as u16 + 3).min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let theme_primary = app.config.theme.primary();

    let mut items: Vec<ListItem> = Vec::new();

    for (i, agent_id) in app.overlay.agent_list.iter().enumerate() {
        let is_selected = i == app.overlay.agent_picker_index;
        let is_current = *agent_id == app.current_agent;
        let prefix = if is_selected { " ▶ " } else { "    " };
        let suffix = if is_current { " ◀ 当前" } else { "" };
        let style = if is_selected {
            Style::default()
                .fg(theme_primary)
                .add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{}{}", agent_id, suffix), style),
        ])]));
    }

    // Footer
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ──────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ↑↓ 选择  Enter 切换  Esc 取消",
        Style::default().fg(Color::DarkGray),
    ))]));

    let list = List::new(items).block(
        Block::default()
            .title(" Agent 切换 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme_primary)),
    );
    f.render_widget(list, popup_area);
}

/// Overlay showing the full request body JSON for a debug log entry.
pub(super) fn render_request_body(
    f: &mut Frame,
    area: Rect,
    body_json: &str,
    idx: usize,
    total: usize,
    scroll: usize,
    cached_json: &mut Option<String>,
) {
    let popup_width = (area.width as f32 * 0.85) as u16;
    let popup_height = (area.height as f32 * 0.8) as u16;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Pretty-print the body JSON if possible (cached)
    if cached_json.is_none() {
        *cached_json = Some(if let Ok(val) = serde_json::from_str::<serde_json::Value>(body_json) {
            serde_json::to_string_pretty(&val).unwrap_or_else(|_| body_json.to_string())
        } else {
            body_json.to_string()
        });
    }
    let formatted = cached_json.as_deref().unwrap_or(body_json);

    let inner_w = (popup_width as usize).saturating_sub(4).max(20);
    // Visible content lines (popup height minus borders minus header)
    let visible_lines = (popup_height as usize).saturating_sub(6).max(1);

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        format!(
            "  🔍 Request Body ({}/{})  [↑↓/scroll to browse | Esc to close]",
            idx + 1,
            total
        ),
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
            utils::wrap_text(trimmed, inner_w)
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
