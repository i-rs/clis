use pulldown_cmark::{Event, Parser, Tag, TagEnd};
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

    let plan_height: u16 = if !app.plan_steps.is_empty() && app.is_processing() {
        (app.plan_steps.len() as u16).min(5)
    } else {
        0
    };

    // Dynamic processing area height: spinner + optional reasoning lines
    let processing_height: u16 = if app.is_processing() && !app.status_text.is_empty() {
        let reason_lines = if !app.current_reasoning.is_empty() {
            (app.current_reasoning.lines().count() as u16).min(3)
        } else {
            0
        };
        1 + reason_lines
    } else {
        1
    };

    let mut constraints = vec![
        Constraint::Length(1), // Title bar
        Constraint::Min(1),    // Chat area
    ];

    if plan_height > 0 {
        constraints.push(Constraint::Length(1)); // Plan indicator
    }

    constraints.extend(vec![
        Constraint::Length(processing_height),             // Processing indicator
        Constraint::Length(input_height(&app.input)),      // Input
        Constraint::Length(1),                             // Status bar
    ]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;
    render_title(f, layout[idx], app);
    idx += 1;
    // Chat area with optional sidebar
    if app.show_sidebar && !app.is_processing() {
        let chat_side = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Percentage(35),
            ])
            .split(layout[idx]);
        render_chat(f, chat_side[0], app);
        render_sidebar(f, chat_side[1], app);
    } else {
        render_chat(f, layout[idx], app);
    }
    idx += 1;
    if plan_height > 0 {
        render_plan(f, layout[idx], app);
        idx += 1;
    }
    render_processing(f, layout[idx], app);
    idx += 1;
    render_input(f, layout[idx], app);
    idx += 1;
    render_status(f, layout[idx], app);

    // Session list overlay (rendered on top of everything)
    if app.show_session_list {
        render_backdrop(f, area);
        render_session_list(f, area, app);
    }

    // Agent picker overlay
    if app.show_agent_picker {
        render_backdrop(f, area);
        render_agent_picker(f, area, app);
    }

    // Help panel overlay
    if app.show_help {
        render_backdrop(f, area);
        render_help_panel(f, area);
    }

    // Completion popup overlay
    if !app.tab_completions.is_empty() {
        render_completions(f, area, app);
    }

    // Request body overlay (rendered on top of everything)
    if let Some(idx) = app.sidebar_body_idx
        && let Some(log) = app.http_logs.get(idx) {
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

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    // Full-width background
    f.render_widget(
        ratatui::widgets::Block::default()
            .style(Style::default().bg(app.config.theme.background())),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    // App name — standout
    spans.push(Span::styled(
        " ✦ i-rs-claw",
        Style::default()
            .fg(app.config.theme.primary())
            .add_modifier(Modifier::BOLD),
    ));

    if app.is_processing() {
        // Processing indicator
        spans.push(Span::styled(
            format!("  ⏳ {} ", app.status_text),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    } else {
        // Tagline + agent name
        spans.push(Span::styled(
            format!("  个人数据智能助理  [{}]", app.current_agent),
            Style::default().fg(Color::Rgb(180, 180, 200)),
        ));
    }

    // Right-aligned model name (padded to fill width)
    let model_text = format!(" {} ", app.config.model);
    let model_text_ref: &str = &model_text;
    let model_width = unicode_width::UnicodeWidthStr::width(model_text_ref);
    let padding = (area.width as usize).saturating_sub(
        spans.iter().map(|s| {
            let content: &str = &s.content;
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

/// Render the current execution plan as a compact progress list.
fn render_plan(f: &mut Frame, area: Rect, app: &App) {
    if app.plan_steps.is_empty() {
        return;
    }
    let done_count = app.plan_steps.iter().filter(|s| s.done).count();

    let total = app.plan_steps.len();
    let max_show = total.min(5);

    let mut lines: Vec<Line> = Vec::with_capacity(max_show + 1);

    // Header with summary
    lines.push(Line::from(Span::styled(
        format!(" 📋 计划 ({}/{}) ", done_count, total),
        Style::default()
            .fg(app.config.theme.primary())
            .add_modifier(Modifier::BOLD),
    )));

    for (i, step) in app.plan_steps.iter().take(max_show).enumerate() {
        let (icon, color) = if step.done {
            ("✓", app.config.theme.secondary())
        } else if i == done_count || (done_count == 0 && i == 0) {
            ("⏳", app.config.theme.accent())
        } else {
            ("○", app.config.theme.dim_text())
        };
        lines.push(Line::from(Span::styled(
            format!("   {} {}", icon, step.description),
            Style::default().fg(color),
        )));
    }

    if total > max_show {
        lines.push(Line::from(Span::styled(
            format!("   ... 还有 {} 步", total - max_show),
            Style::default().fg(app.config.theme.dim_text()),
        )));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_chat(f: &mut Frame, area: Rect, app: &App) {
    // Available text width (minus indentation)
    let text_width = (area.width as usize).saturating_sub(4).max(20);
    // Subtract 1 line for the top border
    let area_lines = (area.height as usize).saturating_sub(1).max(1);

    // Pre-compute formatted ToolCall result lines — used by both line counting and rendering
    let mut format_cache: std::collections::HashMap<usize, Vec<Line<'static>>> =
        std::collections::HashMap::new();

    // Two-pass approach to ensure NEWEST messages are always fully visible:
    // Pass 1: compute heights for all visible (non-skipped) messages
    let mut heights: Vec<usize> = Vec::with_capacity(app.messages.len());
    for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
        if rev_idx < app.scroll_offset {
            continue;
        }
        let msg_index = app.messages.len() - 1 - rev_idx;
        heights.push(message_line_count(
            app,
            msg,
            text_width,
            msg_index,
            &mut format_cache,
        ));
    }
    // heights[0] = newest visible, heights[n-1] = oldest visible

    // Pass 2: determine how many fit from the NEWEST side
    let total = heights.len();
    let mut remaining = area_lines;
    let mut end_idx = 0usize; // exclusive index in heights
    for &h in heights.iter() {
        if h <= remaining {
            remaining -= h;
            end_idx += 1;
        } else {
            break;
        }
    }
    // If even the first message doesn't fit, force show at least it
    if end_idx == 0 && total > 0 {
        end_idx = 1;
    }

    // Pass 3: build items newest-to-oldest, then reverse for chronological order
    let mut items: Vec<ListItem> = Vec::new();
    for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
        if rev_idx < app.scroll_offset {
            continue;
        }
        let array_idx = rev_idx - app.scroll_offset; // index in heights
        if array_idx >= end_idx {
            continue; // too old, clipped from top
        }
        let msg_index = app.messages.len() - 1 - rev_idx;
        items.push(build_message_item(app, msg, text_width, msg_index, &format_cache));
    }
    items.reverse();

    // Show an indicator when scrolled up
    let at_bottom = app.scroll_offset == 0;
    let hidden_extra = total.saturating_sub(end_idx); // extra hidden by area, not by scroll

    let mut block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(if at_bottom && hidden_extra == 0 {
            Color::DarkGray
        } else {
            Color::Rgb(100, 120, 200)
        }));

    let total_hidden = app.scroll_offset + hidden_extra;
    if total_hidden > 0 && !items.is_empty() {
        block = block.title(format!(" ▲ {} 条历史消息 ", total_hidden));
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

    // Split area: first line for spinner+status, rest for reasoning
    let (status_area, reason_area) = if area.height > 1 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        (chunks[0], chunks[1])
    } else {
        (area, Rect::default())
    };

    let label = Line::from(Span::styled(
        format!(" {}  {}", spinner, app.status_text),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, status_area);

    // Render reasoning content below the processing line if available
    if !app.current_reasoning.is_empty() && reason_area.height >= 1 {
        let mut reason_lines: Vec<Line> = app
            .current_reasoning
            .lines()
            .take(reason_area.height as usize)
            .map(|line| {
                let display = truncate_str(line, (area.width as usize).saturating_sub(4).max(20));
                Line::from(Span::styled(
                    format!(" 🤔 {}", display),
                    Style::default().fg(Color::Rgb(120, 120, 140)),
                ))
            })
            .collect();

        // Fill remaining lines
        while reason_lines.len() < reason_area.height as usize {
            reason_lines.push(Line::from(Span::raw("")));
        }

        f.render_widget(Paragraph::new(reason_lines), reason_area);
    }
}

/// Calculate required height for input area based on content line count.
fn input_height(input: &str) -> u16 {
    let content_lines = input.lines().count().max(1);
    // content lines + hint line + top/bottom borders
    (content_lines + 1 + 2).clamp(3, 20) as u16
}

/// Return the input hint line showing available shortcuts.
/// Voice input hint is only shown on macOS where Fn key is available.
fn input_hint_text() -> &'static str {
    if cfg!(target_os = "macos") {
        "  [Fn] 语音输入  [Alt+Enter] 换行"
    } else {
        "  [Alt+Enter] 换行"
    }
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

    let input = Paragraph::new(lines).block(input_block);

    f.render_widget(input, area);

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

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let bg = if app.is_processing() {
        Color::Blue
    } else if app.selection_mode {
        Color::Rgb(40, 30, 10)
    } else {
        app.config.theme.background()
    };

    // Fill full-width background
    f.render_widget(
        ratatui::widgets::Block::default()
            .style(Style::default().bg(bg)),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    // Copy feedback (transient, highest priority)
    if let Some(fb) = &app.copy_feedback {
        spans.push(Span::styled(
            format!(" {} ", fb),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
    }

    if app.selection_mode {
        // Selection mode indicator
        spans.push(Span::styled(
            " ● [选择模式] ".to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "↑↓选择  Space展开  Ctrl+D删除  Ctrl+Shift+C复制  退出Esc",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
    } else if app.is_processing() {
        // Processing state
        spans.push(Span::styled(
            format!(" ⏳ {} ", app.status_text),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
        // Separator
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
        // Tool & message stats
        spans.push(Span::styled(
            format!("⚙ {} ", app.tool_call_count),
            Style::default().fg(app.config.theme.primary()),
        ));
        spans.push(Span::styled(
            format!("💬 {} ", app.messages.len()),
            Style::default().fg(app.config.theme.primary()),
        ));
    } else {
        // Idle state — green dot + bold
        spans.push(Span::styled(
            " ● 就绪 ".to_string(),
            Style::default().fg(app.config.theme.secondary()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{} ", app.config.model),
            Style::default().fg(app.config.theme.primary()),
        ));
        // Separator
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
        // Tool & message stats
        spans.push(Span::styled(
            format!("⚙ {} ", app.tool_call_count),
            Style::default().fg(app.config.theme.primary()),
        ));
        spans.push(Span::styled(
            format!("💬 {} ", app.messages.len()),
            Style::default().fg(app.config.theme.primary()),
        ));
        // Token usage
        if let Some(usage) = &app.token_usage {
            spans.push(Span::styled(
                "│ ",
                Style::default().fg(app.config.theme.dim_text()),
            ));
            spans.push(Span::styled(
                format!("tok: {}p+{}c ", usage.prompt_tokens, usage.completion_tokens),
                Style::default().fg(app.config.theme.accent()),
            ));
        }
        // Keybindings (right side)
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
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
            // Only show Ctrl+R if there are HTTP logs to inspect
            if !app.http_logs.is_empty() {
                spans.push(Span::styled(
                    "Ctrl+R  ",
                    Style::default().fg(Color::Rgb(140, 140, 160)),
                ));
            }
        }
        spans.push(Span::styled(
            "Ctrl+L",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  Ctrl+P ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "Ctrl+Shift+C",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  Ctrl+S",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  Ctrl+H",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
    }

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

    // Filter sessions by search text
    let q = app.session_search.to_lowercase();
    let filtered: Vec<&crate::session::SessionMeta> = if q.is_empty() {
        app.session_list.iter().collect()
    } else {
        app.session_list
            .iter()
            .filter(|s| s.title.to_lowercase().contains(&q))
            .collect()
    };

    let empty = filtered.is_empty();
    let theme_primary = app.config.theme.primary();

    let mut items: Vec<ListItem> = Vec::new();

    // Header with optional search bar
    let search_display = if app.session_search_mode {
        let search_line = if app.session_search.is_empty() {
            " 🔍 输入搜索关键词…".to_string()
        } else {
            format!(" 🔍 {}", app.session_search)
        };
        items.push(ListItem::new(vec![
            Line::from(Span::styled(
                search_line,
                Style::default()
                    .fg(theme_primary)
                    .add_modifier(Modifier::BOLD),
            )),
        ]));
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
            if q.is_empty() {
                " 暂无会话"
            } else {
                " 未找到匹配会话"
            },
            Style::default().fg(Color::DarkGray),
        ))]));
    } else {
        for (i, session) in filtered.iter().enumerate() {
            let selected = i == app.session_list_index;
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

    // Rename input field
    if !app.session_rename_buf.is_empty() {
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
                app.session_rename_buf.clone(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ▌",
                Style::default().fg(Color::Yellow),
            ),
        ])]));
    }

    // Delete confirmation
    if app.session_confirm_delete {
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
        if app.session_confirm_delete {
            " 确认删除? (y/n)"
        } else if !app.session_rename_buf.is_empty() {
            " 输入新名称  Enter 确认  Esc 取消"
        } else if empty && !app.session_search_mode {
            " Ctrl+N 新建会话  Ctrl+L 关闭"
        } else if app.session_search_mode {
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

/// Overlay showing the agent profile picker.
fn render_agent_picker(f: &mut Frame, area: Rect, app: &App) {
    let popup_width = 40u16.min(area.width.saturating_sub(4));
    let popup_height = (app.agent_list.len() as u16 + 3).min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let theme_primary = app.config.theme.primary();

    let mut items: Vec<ListItem> = Vec::new();

    for (i, agent_id) in app.agent_list.iter().enumerate() {
        let is_selected = i == app.agent_picker_index;
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

/// Help panel overlay showing all keyboard shortcuts.
fn render_help_panel(f: &mut Frame, area: Rect) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 28u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let shortcuts = [
        ("Ctrl+Q", "退出程序"),
        ("Ctrl+N", "新建会话"),
        ("Ctrl+S", "消息选择模式"),
        ("Ctrl+H", "显示/隐藏帮助"),
        ("Ctrl+L", "清屏 / 重置滚动"),
        ("Ctrl+P", "会话列表"),
        ("Ctrl+R", "HTTP 调试面板"),
        ("Ctrl+Shift+C", "复制当前消息"),
        ("Alt+Enter", "输入换行"),
        ("Fn", "语音输入 (macOS)"),
        ("", ""),
        ("--- 选择模式 ---", ""),
        ("↑/↓", "切换选中消息"),
        ("Space", "展开/折叠工具调用"),
        ("Ctrl+D", "删除选中消息"),
        ("Esc/q", "退出选择模式"),
        ("", ""),
        ("--- 会话列表 ---", ""),
        ("Ctrl+F", "搜索会话"),
        ("Enter", "切换到会话"),
        ("Ctrl+D", "删除会话"),
        ("Ctrl+R", "重命名会话"),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (key, desc) in &shortcuts {
        if key.is_empty() {
            lines.push(Line::from(Span::raw("")));
            continue;
        }
        if desc.is_empty() {
            // Section header
            lines.push(Line::from(Span::styled(
                *key,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<18}", key),
                    Style::default().fg(Color::Rgb(180, 180, 120)),
                ),
                Span::styled(
                    desc.to_string(),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" ⌨ 快捷键帮助 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, popup_area);
}

/// Render a dark backdrop over the entire terminal area when overlays are shown.
fn render_backdrop(f: &mut Frame, area: Rect) {
    f.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Rgb(20, 20, 30))),
        area,
    );
}

/// Overlay showing tab completion candidates near the input area.
fn render_completions(f: &mut Frame, area: Rect, app: &App) {
    if app.tab_completions.is_empty() {
        return;
    }

    let count = app.tab_completions.len();
    let popup_height = (count as u16).min(12).saturating_add(2); // header + footer
    let popup_width = (area.width as f32 * 0.45) as u16;
    let popup_x = area.x + 2;
    let popup_y = area.bottom().saturating_sub(
        1  // status bar
        + input_height(&app.input)
        + 1  // processing
        + popup_height
        + 2
    );

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let idx = app.tab_completion_index;
    let theme_primary = app.config.theme.primary();

    let mut items: Vec<ListItem> = Vec::new();

    // Header
    items.push(ListItem::new(vec![
        Line::from(Span::styled(
            format!(" Tab 补全 ({} 个)", count),
            Style::default()
                .fg(theme_primary)
                .add_modifier(Modifier::BOLD),
        )),
    ]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));

    for (i, completion) in app.tab_completions.iter().enumerate() {
        if i >= 12 {
            let remaining = count - 12;
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!("   ... 还有 {} 个", remaining),
                Style::default().fg(Color::DarkGray),
            ))]));
            break;
        }
        let selected = i == idx;
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
            Span::styled(completion.clone(), style),
        ])]));
    }

    // Footer
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " Tab 选择  Shift+Tab 反向  Esc 关闭",
        Style::default().fg(Color::DarkGray),
    ))]));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme_primary)),
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
    let width = unicode_width::UnicodeWidthStr::width(s);
    if width <= max_len {
        s.to_string()
    } else {
        let mut out = String::new();
        let mut w = 0usize;
        for c in s.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if w + cw + 3 > max_len {
                break;
            }
            out.push(c);
            w += cw;
        }
        out.push_str("...");
        out
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

/// Convert a NaiveDateTime to a relative time string.
fn relative_time_naive(ts: chrono::NaiveDateTime) -> String {
    relative_time(ts.and_utc().timestamp())
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

    // Walk raw char by char (via char_indices to guarantee char boundaries),
    // skip ANSI escapes, and track corresponding byte position in plain.
    let line_end_byte = line_start + line.len();
    let mut raw_bytes = raw.char_indices();
    let mut plain_byte_pos: usize = 0;
    let mut raw_start: Option<usize> = None;
    let mut raw_end: usize = raw.len();

    while let Some((raw_offset, c)) = raw_bytes.next() {
        if c == '\x1b' {
            // Skip ANSI escape sequence
            for (_, esc_c) in &mut raw_bytes {
                if esc_c.is_ascii_alphabetic() || esc_c == '~' {
                    break;
                }
            }
            continue;
        }

        if raw_start.is_none() && plain_byte_pos >= line_start {
            raw_start = Some(raw_offset);
        }

        if raw_start.is_some() && plain_byte_pos >= line_end_byte {
            raw_end = raw_offset;
            break;
        }

        plain_byte_pos += c.len_utf8();
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
fn message_line_count(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &mut std::collections::HashMap<usize, Vec<Line<'static>>>,
) -> usize {
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
            ..
        } => {
            // Collapsed: only header + optional explanation
            if !app.tool_call_expanded.contains(&msg_index) {
                let mut lines = 1; // header
                // optional explanation line
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(args)
                    && name == "i_rs" && val.get("explanation").and_then(|v| v.as_str()).is_some() {
                        lines += 1;
                    }
                return lines;
            }

            let mut lines = 1; // header
            // optional explanation line
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(args)
                && name == "i_rs" && val.get("explanation").and_then(|v| v.as_str()).is_some() {
                    lines += 1;
                }
            // result lines — use cached format_json_result for accurate counting
            if !result.is_empty() {
                format_cache.entry(msg_index).or_insert_with(|| format_json_result(result, text_width).0);
                let cached = format_cache.get(&msg_index).unwrap();
                lines += cached.len();
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
                w.div_ceil(max_width)
            }
        })
        .sum()
}

/// Quick check if a string contains markdown syntax worth rendering.
fn is_markdown(text: &str) -> bool {
    text.contains("**")
        || text.contains('*')
        || text.contains('`')
        || (text.len() > 1 && text.as_bytes()[0] == b'#')
        || (text.len() > 1 && text.as_bytes()[0] == b'-' && text.as_bytes()[1] == b' ')
        || text.contains("\n- ")
        || text.contains("---")
        || text.contains("___")
}

/// Render markdown text into styled ratatui lines.
///
/// Supports: **bold**, *italic*, `inline code`, ```code blocks```,
/// headings (# ## ###), lists (-), and horizontal rules (---).
fn render_markdown(text: &str, max_width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Accumulator for one logical line (paragraph fragment)
    struct MdLine {
        spans: Vec<(String, Style)>,
        width: usize,
    }
    impl MdLine {
        fn new() -> Self { Self { spans: Vec::new(), width: 0 } }
        fn add(&mut self, text: &str, style: Style) {
            self.width += unicode_width::UnicodeWidthStr::width(text);
            if let Some(last) = self.spans.last_mut()
                && last.1 == style {
                    last.0.push_str(text);
                    return;
                }
            self.spans.push((text.to_string(), style));
        }
        fn flush(&mut self, out: &mut Vec<Line<'static>>, max_width: usize) {
            if self.spans.is_empty() {
                return;
            }
            if self.width <= max_width {
                let spans: Vec<Span> = self.spans.drain(..)
                    .map(|(t, s)| Span::styled(t, s))
                    .collect();
                out.push(Line::from(spans));
            } else {
                // Wrap long lines: rebuild from plain text (loses inner styles)
                let plain: String = self.spans.iter().map(|(t, _)| t.as_str()).collect();
                self.spans.clear();
                for w in wrap_text(&plain, max_width) {
                    out.push(Line::from(Span::styled(w, Style::default().fg(Color::White))));
                }
            }
        }
    }

    let mut acc = MdLine::new();
    let mut bold = false;
    let mut italic = false;
    let mut in_code_block = false;
    let mut code_text = String::new();

    for event in Parser::new(text) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    acc.flush(&mut lines, max_width);
                    let n = level as u8;
                    let prefix = if n <= 3 && n > 0 {
                        format!("{} ", "#".repeat(n as usize))
                    } else {
                        String::new()
                    };
                    acc.add(&prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
                }
                Tag::List(_) => {}
                Tag::Item => {
                    acc.flush(&mut lines, max_width);
                    acc.add("• ", Style::default().fg(Color::Cyan));
                }
                Tag::Emphasis => italic = true,
                Tag::Strong => bold = true,
                Tag::CodeBlock(_) => {
                    acc.flush(&mut lines, max_width);
                    in_code_block = true;
                    code_text.clear();
                }
                Tag::Link { .. } => {}
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    acc.flush(&mut lines, max_width);
                }
                TagEnd::Heading(_) => {
                    acc.flush(&mut lines, max_width);
                }
                TagEnd::Item => {
                    acc.flush(&mut lines, max_width);
                }
                TagEnd::Emphasis => italic = false,
                TagEnd::Strong => bold = false,
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    if code_text.lines().any(|l| !l.trim().is_empty()) {
                        // Code block header
                        lines.push(Line::from(Span::styled(
                            format!("{:─^width$}", " code ", width = max_width.min(40)),
                            Style::default()
                                .fg(Color::Rgb(120, 120, 80))
                                .bg(Color::Rgb(20, 20, 25)),
                        )));
                        for code_line in code_text.lines() {
                            lines.push(Line::from(Span::styled(
                                format!("  {}", code_line),
                                Style::default()
                                    .fg(Color::Rgb(200, 200, 120))
                                    .bg(Color::Rgb(20, 20, 25)),
                            )));
                        }
                        lines.push(Line::from(Span::styled(
                            "".to_string(),
                            Style::default().bg(Color::Rgb(20, 20, 25)),
                        )));
                    }
                }
                TagEnd::Link => {}
                TagEnd::List(_) => {}
                _ => {}
            },
            Event::Text(t) => {
                if in_code_block {
                    code_text.push_str(&t);
                } else {
                    let mut style = Style::default().fg(Color::White);
                    if bold {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if italic {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    acc.add(&t, style);
                }
            }
            Event::Code(t) => {
                acc.add(
                    &t,
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Rgb(30, 30, 30)),
                );
            }
            Event::SoftBreak | Event::HardBreak => {
                acc.flush(&mut lines, max_width);
            }
            Event::Rule => {
                acc.flush(&mut lines, max_width);
                lines.push(Line::from(Span::styled(
                    "  ─────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            _ => {}
        }
    }

    acc.flush(&mut lines, max_width);
    lines
}

/// Build a ListItem widget from a Message.
fn build_message_item(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Vec<Line<'static>>>,
) -> ListItem<'static> {
    let is_selected = app.selection_mode && app.selected_message == Some(msg_index);

    match msg {
        Message::User { text } => {
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", relative_time_naive(*ts)))
                .unwrap_or_default();
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("  You:{}", ts_label),
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
            let bg = if is_selected { Color::Rgb(25, 35, 25) } else { Color::Rgb(12, 18, 14) };
            ListItem::new(lines).style(Style::default().bg(bg))
        }
        Message::Assistant { text } => {
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", relative_time_naive(*ts)))
                .unwrap_or_default();
            let mut lines = vec![Line::from(Span::styled(
                format!("  Claw:{}", ts_label),
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
                let md_lines = render_markdown(text, text_width.saturating_sub(3));
                if !is_markdown(text) || md_lines.is_empty() {
                    // Fallback to simple wrapping for plain text
                    for wrapped in wrap_text(text, text_width) {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", wrapped),
                            Style::default().fg(Color::White),
                        )));
                    }
                } else {
                    for md_line in &md_lines {
                        lines.push(md_line.clone());
                    }
                }
            }
            lines.push(Line::from(Span::raw("")));
            let bg = if is_selected { Color::Rgb(25, 30, 45) } else { Color::Rgb(0, 0, 0) };
            let mut item = ListItem::new(lines);
            if is_selected {
                item = item.style(Style::default().bg(bg));
            }
            item
        }
        Message::ToolCall {
            name,
            args,
            result,
            step,
            total_steps,
        } => {
            let mut lines = Vec::new();
            let is_expanded = app.tool_call_expanded.contains(&msg_index);

            // Build step prefix for multi-call progress
            let step_prefix = if *total_steps > 1 {
                format!("[{}/{}] ", *step + 1usize, total_steps)
            } else {
                String::new()
            };

            let (header, detail) =
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
                    if name == "i_rs" {
                        let tool = val.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
                        let cmd = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
                        let explanation = val.get("explanation").and_then(|v| v.as_str());
                        (
                            format!(" ⚡ {}{} {}", step_prefix, tool, cmd),
                            explanation.map(|s| s.to_string()),
                        )
                    } else if name == "search_conversations" {
                        let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                        (format!(" 🔍 搜索历史: {}", q), None)
                    } else if name == "search_tools" {
                        let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                        (format!(" 🔍 search: {}", q), None)
                    } else if name == "update_user_memory" {
                        (" 💾 记住用户信息".to_string(), None)
                    } else if name == "file_ops" {
                        let op = val.get("operation").and_then(|v| v.as_str()).unwrap_or("?");
                        let p = val.get("path").and_then(|v| v.as_str()).unwrap_or("?");
                        (format!(" 📁 {}: {}", op, p), None)
                    } else if name == "web_search" {
                        let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                        (format!(" 🔍 搜索网络: {}", q), None)
                    } else {
                        (format!(" ⚡ {}{}", step_prefix, name), None)
                    }
                } else {
                    (format!(" ⚡ {}{} {}", step_prefix, name, args), None)
                };

            let indicator = if is_expanded { " [-]" } else { " [+]" };
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", relative_time_naive(*ts)))
                .unwrap_or_default();
            lines.push(Line::from(Span::styled(
                format!("{}{}{}", header, indicator, ts_label),
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

            if is_expanded && !result.is_empty()
                && let Some(cached_lines) = format_cache.get(&msg_index) {
                    if !cached_lines.is_empty() {
                        lines.extend(cached_lines.clone());
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

            let bg = if is_selected { Color::Rgb(25, 25, 35) } else { Color::Rgb(10, 10, 16) };
            ListItem::new(lines).style(Style::default().bg(bg))
        }
        Message::Error { text } => {
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", relative_time_naive(*ts)))
                .unwrap_or_default();
            let mut lines = vec![
                Line::from(Span::styled(
                    format!(" ✗ Error:{}", ts_label),
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
            let bg = if is_selected { Color::Rgb(35, 15, 15) } else { Color::Rgb(18, 8, 8) };
            ListItem::new(lines).style(Style::default().bg(bg))
        }
    }
}
