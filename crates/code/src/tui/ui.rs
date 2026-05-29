use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use crate::app::{AgentMessage, App, AppMode};
use crate::tui::colors::*;
use super::strings;

const SIDEBAR_WIDTH: u16 = 38;

fn tool_glyph(name: &str) -> &'static str {
    match name {
        "write" | "edit" | "create" => "◆",
        "read" | "glob" | "grep" | "ls" => "▷",
        "bash" | "run" | "exec" => "▶",
        "web_fetch" | "web_search" | "search" => "⌕",
        "delete" => "✕",
        "rename" => "→",
        "create_crate" => "+",
        "call_claw" | "delegate" => "◐",
        "register_tool" => "◎",
        "git" => "±",
        _ => "•",
    }
}

fn short_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(&*home_str) {
            return format!("~{}", rest);
        }
    }
    path.to_string()
}

fn is_diff_like(text: &str) -> bool {
    text.lines().any(|l| l.starts_with("--- ") || l.starts_with("+++ ") || l.starts_with("@@ "))
}

fn render_diff_line(line: &str) -> Vec<Span<'static>> {
    if let Some(rest) = line.strip_prefix("+") {
        vec![Span::styled("+", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
             Span::styled(rest.to_string(), Style::default().fg(C_DIFF_GREEN))]
    } else if let Some(rest) = line.strip_prefix("-") {
        vec![Span::styled("-", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
             Span::styled(rest.to_string(), Style::default().fg(C_DIFF_RED))]
    } else if line.starts_with("@@") {
        vec![Span::styled(line.to_string(), Style::default().fg(Color::Cyan))]
    } else {
        vec![Span::raw(line.to_string())]
    }
}

fn fmt_count(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let input_lines = (app.input.content.lines().count() + 1).clamp(2, 8) as u16 + 2;

    // Title (full width) | below title: chat | sidebar
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    render_title_bar(frame, vert[0], app);

    // Below title: chat column | sidebar column
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(SIDEBAR_WIDTH)])
        .split(vert[1]);

    // Chat column: chat area + input bar
    let chat_col = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(input_lines)])
        .split(horiz[0]);

    render_chat(frame, chat_col[0], app);
    render_input_bar(frame, chat_col[1], app);

    // Sidebar column: full height (spans chat + input rows)
    crate::tui::sidebar::render_sidebar(frame, horiz[1], app);

    if app.show_shortcuts {
        render_shortcuts_overlay(frame, area);
    }

    if app.show_debug {
        render_debug_overlay(frame, area, app);
    }
}

pub fn render_transcript(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Transcript (Ctrl+T to close, ↑↓ scroll) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for msg in &app.messages {
        match msg {
            AgentMessage::User { content } => {
                lines.push(Line::from(Span::styled("── User ──", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
            }
            AgentMessage::Assistant { content, reasoning, tool_calls, reasoning_expanded: _ } => {
                lines.push(Line::from(Span::styled("── Assistant ──", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
                if !reasoning.is_empty() {
                    for line in reasoning.lines() {
                        lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC))));
                    }
                }
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
                if let Some(tcs) = tool_calls {
                    for tc in tcs {
                        let name = tc.get("name").and_then(|v| v.as_str()).unwrap_or("tool");
                        lines.push(Line::from(Span::styled(format!("  [tool_call] {}", name), Style::default().fg(Color::Yellow))));
                    }
                }
            }
            AgentMessage::ToolResult { content } => {
                let (tool_name, tool_result) = content.split_once('\n').unwrap_or(("", content));
                lines.push(Line::from(Span::styled(
                    format!("── {} ──", if tool_name.is_empty() { "Tool" } else { tool_name }),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                if !tool_result.is_empty() {
                    for line in tool_result.lines() {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
            }
            AgentMessage::FileEdit { path, summary } => {
                lines.push(Line::from(Span::styled(format!("── File Edit: {} ──", path), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
                for line in summary.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
            }
            AgentMessage::System { content } => {
                lines.push(Line::from(Span::styled("── System ──", Style::default().fg(C_DIM))));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
            }
            AgentMessage::Separator { .. } => {}
        }
        lines.push(Line::from(""));
    }

    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let scroll = app.transcript_scroll.min(max_scroll);
    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, inner);
}

fn render_shortcuts_overlay(frame: &mut Frame, area: Rect) {
    let w = 56.min(area.width.saturating_sub(4));
    let h = 22;
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    let overlay = Rect { x, y, width: w, height: h };

    frame.render_widget(Clear, overlay);

    let items: Vec<Line> = std::iter::once(Line::from(""))
        .chain(strings::SHORTCUTS.iter().map(|(key, label)| {
            Line::from(Span::styled(
                format!("  {:<14} {}", key, label),
                Style::default().fg(Color::White),
            ))
        }))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(Span::styled(
            "     Press any key to close", Style::default().fg(C_DIM),
        ))))
        .collect();

    let block = Block::default()
        .title(format!(" {} ", strings::SHORTCUT_TITLE))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(Text::from(items))
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, overlay);
}

fn render_debug_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let w = area.width.saturating_sub(4).min(80);
    let h = area.height.saturating_sub(4).min(30);
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    let overlay = Rect { x, y, width: w, height: h };

    frame.render_widget(Clear, overlay);

    let logs = crate::debug::get_log();
    let scroll = app.debug_scroll.min(logs.len().saturating_sub(1));
    let visible: Vec<Line> = logs.iter().skip(scroll).take((h as usize).saturating_sub(3)).map(|entry| {
        let status_style = match entry.response_status {
            200 => Style::default().fg(Color::Green),
            s if s >= 400 => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Yellow),
        };
        Line::from(vec![
            Span::styled(format!("{} ", entry.time_short()), Style::default().fg(C_DIM)),
            Span::styled(entry.status_label(), status_style),
            Span::raw(format!(" {} ({}ms)", entry.path(), entry.duration_ms)),
        ])
    }).collect();

    let block = Block::default()
        .title(" Debug Log (Ctrl+D close, Ctrl+L clear, ↑↓ scroll) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(Text::from(visible))
        .block(block)
        .scroll((if scroll > 0 { scroll as u16 } else { 0 }, 0));
    frame.render_widget(paragraph, overlay);
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let dir = short_path(&app.current_dir);

    let context_text = if let Some(pct) = app.context_usage {
        let pct_str = format!("{:.0}%", pct * 100.0);
        let ctx_color = if pct > 0.8 { Color::Red } else if pct > 0.6 { Color::Yellow } else { Color::Green };
        vec![Span::raw("  "), Span::styled(pct_str, Style::default().fg(ctx_color))]
    } else {
        vec![]
    };

    let sel_text = app.selected_message.map(|idx| {
        Span::styled(format!(" #{} ", idx), Style::default().fg(C_YELLOW))
    });

    let mut spans = vec![
        Span::styled(" i-rs-code ", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(format!("v{}", app.version), Style::default().fg(C_ACCENT)),
        Span::raw("  "),
        Span::styled(dir, Style::default().fg(C_DIM)),
    ];
    spans.extend(context_text);
    if let Some(s) = sel_text {
        spans.push(s);
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled("[?]", Style::default().fg(C_YELLOW)));

    let text = Line::from(spans);
    frame.render_widget(Clear, area);
    let bar = Paragraph::new(text);
    frame.render_widget(bar, area);

    // Bottom separator
    let sep_line = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(C_RAIL),
    ))]));
    let sep_area = Rect { x: area.x, y: area.y + area.height.saturating_sub(1), width: area.width, height: 1 };
    frame.render_widget(sep_line, sep_area);
}

fn render_chat(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    let mut last_was_tool = false;

    for (msg_idx, msg) in app.messages.iter().enumerate() {
        let is_selected = app.selected_message == Some(msg_idx);
        let sel_prefix = if is_selected { "▶" } else { " " };

        match msg {
            AgentMessage::User { content } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::default().fg(C_ACCENT)),
                    Span::styled("▎You", Style::default().fg(Color::Rgb(59, 130, 246)).add_modifier(Modifier::BOLD)),
                ]));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
            AgentMessage::Assistant { content, reasoning, tool_calls: _, reasoning_expanded } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::default().fg(C_YELLOW)),
                    Span::styled("▎AI", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
                ]));
                if !reasoning.is_empty() {
                    lines.push(Line::from(Span::styled(
                        if *reasoning_expanded { strings::REASONING_VISIBLE } else { strings::REASONING_HIDDEN },
                        Style::default().fg(C_YELLOW),
                    )));
                }
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
            AgentMessage::ToolResult { content } => {
                let (tool_name, tool_result) = content.split_once('\n').unwrap_or(("", content));
                let glyph = tool_glyph(tool_name);
                let label = if tool_name.is_empty() { "Tool" } else { tool_name };

                let (rail_top, rail_mid) = if last_was_tool { ("│", "│") } else { ("╭", "│") };

                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::default().fg(C_YELLOW)),
                    Span::styled(format!(" {} ", glyph), Style::default().fg(C_YELLOW)),
                    Span::styled(label, Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD)),
                ]));
                if !tool_result.is_empty() {
                    let preview: String = tool_result.chars().take(1200).collect();
                    let result_lines: Vec<&str> = preview.lines().collect();
                    for (i, line) in result_lines.iter().enumerate().take(12) {
                        let prefix = if i == result_lines.len().saturating_sub(1).min(11) { "╰" } else { rail_mid };
                        if is_diff_like(line) {
                            let diff_spans = render_diff_line(line);
                            let mut spans = vec![Span::styled(format!(" {} ", prefix), Style::default().fg(Color::DarkGray))];
                            spans.extend(diff_spans);
                            lines.push(Line::from(spans));
                        } else {
                            lines.push(Line::from(Span::styled(
                                format!(" {} {}", prefix, line),
                                Style::default().fg(C_TOOL_OUTPUT),
                            )));
                        }
                    }
                    if preview.len() < tool_result.len() || tool_result.lines().count() > 12 {
                        lines.push(Line::from(Span::styled(
                            format!(" {} … {} more bytes (Ctrl+T)", rail_top, tool_result.len().saturating_sub(preview.len())),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
                last_was_tool = true;
            }
            AgentMessage::FileEdit { path, summary } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::default().fg(C_FILE_EDIT)),
                    Span::styled(format!(" ✎ {} ", path), Style::default().fg(C_FILE_EDIT).add_modifier(Modifier::BOLD)),
                ]));
                for line in summary.lines() {
                    if is_diff_like(line) {
                        let diff_spans = render_diff_line(line);
                        lines.push(Line::from(diff_spans));
                    } else {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(C_FILE_EDIT))));
                    }
                }
            }
            AgentMessage::System { content } => {
                last_was_tool = false;
                for line in content.lines() {
                    if line.starts_with("──") {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(C_SUMMARY_GREEN).add_modifier(Modifier::BOLD))));
                    } else if line.starts_with("📄") || line.starts_with("🔧") {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(C_YELLOW))));
                    } else {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(C_DIM))));
                    }
                }
            }
            AgentMessage::Separator { label } => {
                last_was_tool = false;
                let sep = if label.is_empty() { " ── done ── ".to_string() } else { format!(" ── {} ── ", label) };
                lines.push(Line::from(Span::styled(sep, Style::default().fg(C_SEP))));
            }
        }
        lines.push(Line::from(""));
    }

    // Streaming block
    if let Some(ref s) = app.streaming {
        lines.push(Line::from(vec![
            Span::styled("▎AI", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
        ]));

        for tool in &s.tool_calls {
            lines.push(Line::from(""));
            let glyph = tool_glyph(&tool.name);
            lines.push(Line::from(vec![
                Span::styled(format!(" {} {} done", glyph, tool.name), Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
            ]));
            if let Some(ref result) = tool.result {
                let preview: String = result.chars().take(300).collect();
                for line in preview.lines().take(4) {
                    lines.push(Line::from(Span::styled(format!("   └ {}", line), Style::default().fg(Color::DarkGray))));
                }
                if preview.len() < result.len() || result.lines().count() > 4 {
                    lines.push(Line::from(Span::styled("   └ ...", Style::default().fg(Color::DarkGray))));
                }
            }
        }

        if let Some(ref tool) = s.current_tool {
            lines.push(Line::from(""));
            let glyph = tool_glyph(&tool.name);
            lines.push(Line::from(vec![
                Span::styled(format!(" {} {} running...", glyph, tool.name), Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
            ]));
            let preview: String = tool.args.chars().take(area.width.saturating_sub(8) as usize).collect();
            for line in preview.lines() {
                lines.push(Line::from(Span::styled(format!("   └ {}", line), Style::default().fg(Color::DarkGray))));
            }
            if let Some(ref diff) = tool.diff {
                let diff_lines: Vec<&str> = diff.lines().collect();
                let show = if diff_lines.len() > 10 { &diff_lines[..10] } else { &diff_lines[..] };
                for line in show {
                    let (sign, rest) = line.split_at(1);
                    let style = match sign {
                        "+" => Style::default().fg(Color::Green),
                        "-" => Style::default().fg(Color::Red),
                        _ => Style::default().fg(Color::DarkGray),
                    };
                    lines.push(Line::from(Span::styled(format!("   {}", line), style)));
                }
                if diff_lines.len() > 10 {
                    lines.push(Line::from(Span::styled("   ... (diff truncated)", Style::default().fg(Color::DarkGray))));
                }
            }
        }

        if !s.reasoning.is_empty() {
            let reasoning_lines: Vec<&str> = s.reasoning.lines().collect();
            let total = reasoning_lines.len();
            // Show last 3-6 lines as live preview
            let show_count = if s.content.is_empty() { 6.min(total) } else { 3.min(total) };
            let start = total.saturating_sub(show_count);
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!(" ╎ … {} earlier lines", start),
                    Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC),
                )));
            }
            for line in &reasoning_lines[start..] {
                lines.push(Line::from(Span::styled(
                    format!(" ╎ {}", line),
                    Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC),
                )));
            }
        }

        if !s.content.is_empty() {
            for line in s.content.lines() {
                lines.push(Line::from(Span::raw(format!(" {}", line))));
            }
        }

        if s.current_tool.is_none() && !s.content.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(" ▊", Style::default().fg(C_GREEN))]));
        } else if s.current_tool.is_none() && s.reasoning.is_empty() && s.content.is_empty() {
            lines.push(Line::from(vec![Span::styled(" ╎ ...", Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC))]));
        }
    }

    // Bottom padding
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let max_scroll = lines.len().saturating_sub(area.height as usize);
    let scroll = if app.auto_scroll { max_scroll } else { max_scroll.saturating_sub(app.scroll_offset).min(max_scroll) };


    // Show "↑ N 条历史消息" at the top if scrolled
    let hidden_msgs = app.messages.len().saturating_sub(1);
    if scroll > 0 && hidden_msgs > 0 {
        let mut header = vec![Line::from(Span::styled(
            strings::scrolled_up_hint(hidden_msgs),
            Style::default().fg(C_DIM),
        ))];
        if let Some(idx) = app.selected_message {
            header.push(Line::from(Span::styled(
                format!(" 📍 #{}  ", idx),
                Style::default().fg(C_YELLOW),
            )));
        }
        for h in header {
            lines.insert(0, h);
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(1)))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let input_bg = Style::default().bg(C_BG_INPUT);
    frame.render_widget(Clear, area);
    let bg_fill = Paragraph::new(Text::from(vec![Line::from("")])).style(input_bg);
    frame.render_widget(bg_fill, area);

    // Top separator line
    let sep_color = if matches!(app.mode, AppMode::Waiting) { C_SEP } else { Color::Rgb(42, 42, 50) };
    let sep = Span::styled("─".repeat(area.width as usize), Style::default().fg(sep_color));
    let sep_line = Paragraph::new(Text::from(vec![Line::from(sep)])).style(input_bg);
    let sep_area = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
    frame.render_widget(sep_line, sep_area);

    let inner = Rect { x: area.x + 1, y: area.y + 1, width: area.width.saturating_sub(2), height: area.height.saturating_sub(1) };
    let prefix = "> ";

    let hint = Line::from(Span::styled(
        strings::STATUS_BAR,
        Style::default().fg(C_DIM),
    ));
    let lines: Vec<Line> = if matches!(app.mode, AppMode::Waiting) {
        vec![
            Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(C_DIM)),
                Span::styled(&app.input.content, Style::default().fg(C_DIM)),
            ]),
            hint,
        ]
    } else if app.input.content.is_empty() {
        vec![
            Line::from(Span::styled(format!("{}{}", prefix, strings::INPUT_PLACEHOLDER), Style::default().fg(C_DIM))),
            hint,
        ]
    } else {
        let mut result: Vec<Line> = app.input.content.lines().enumerate().map(|(i, line)| {
            let p = if i == 0 { prefix } else { "  " };
            Line::from(Span::styled(format!("{}{}", p, line), Style::default().fg(Color::Rgb(250, 250, 250))))
        }).collect();
        result.push(hint);
        result
    };

    let input_widget = Paragraph::new(lines).style(input_bg);
    frame.render_widget(input_widget, inner);

    let prefix_width = unicode_width::UnicodeWidthStr::width(prefix) as u16;
    if matches!(app.mode, AppMode::Idle) && !app.input.content.is_empty() {
        let input_before = &app.input.content[..app.input.cursor_pos];
        let line_idx = input_before.matches('\n').count();
        let current_line_start = input_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let pos_in_line = unicode_width::UnicodeWidthStr::width(&input_before[current_line_start..]);
        let cursor_x = inner.x + prefix_width + pos_in_line as u16;
        let cursor_y = inner.y + line_idx as u16;
        frame.set_cursor_position((cursor_x, cursor_y));
    } else if matches!(app.mode, AppMode::Idle) {
        let cursor_x = inner.x + prefix_width;
        let cursor_y = inner.y;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
