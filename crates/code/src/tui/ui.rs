use std::sync::atomic::Ordering;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use crate::app::{AgentMessage, App, AppMode};

const SIDEBAR_WIDTH: u16 = 32;

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

fn is_diff_like(text: &str) -> bool {
    text.lines().any(|l| l.starts_with("--- ") || l.starts_with("+++ ") || l.starts_with("@@ "))
}

fn render_diff_line(line: &str) -> Vec<Span<'static>> {
    if let Some(rest) = line.strip_prefix("+") {
        vec![Span::styled("+", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
             Span::styled(rest.to_string(), Style::default().fg(Color::Rgb(140, 200, 140)))]
    } else if let Some(rest) = line.strip_prefix("-") {
        vec![Span::styled("-", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
             Span::styled(rest.to_string(), Style::default().fg(Color::Rgb(200, 140, 140)))]
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

    // Top-level vertical: Title | Content+Sidebar | Input
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(input_lines),
        ])
        .split(area);

    // Title bar full width
    render_title_bar(frame, vert[0], app);

    // Below title: horizontal split for chat + sidebar
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(SIDEBAR_WIDTH)])
        .split(vert[1]);

    render_chat(frame, horiz[0], app);
    render_sidebar(frame, horiz[1], app);

    // Input bar full width
    render_input_bar(frame, vert[2], app);

    if app.show_shortcuts {
        render_shortcuts_overlay(frame, area);
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
                        lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Rgb(113, 113, 122)).add_modifier(Modifier::ITALIC))));
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
                lines.push(Line::from(Span::styled("── System ──", Style::default().fg(Color::Rgb(113, 113, 122)))));
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

    let items = vec![
        Line::from(""),
        Line::from(Span::styled("  ? / Esc       关闭此面板", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Enter         发送消息", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Alt+Enter     换行", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Esc / q       退出", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Ctrl+C        取消当前生成", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Ctrl+Z        撤销文件修改", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Ctrl+T        转录模式（完整输出）", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Ctrl+D        HTTP 调试面板", Style::default().fg(Color::White))),
        Line::from(Span::styled("  [ / ]         选择上/下一条消息", Style::default().fg(Color::White))),
        Line::from(Span::styled("  r             展开/折叠选中消息的思考过程", Style::default().fg(Color::White))),
        Line::from(Span::styled("  ↑ / ↓ / PgUp  滚动聊天", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Tab           工具名补全", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("     Press any key to close", Style::default().fg(Color::Rgb(113, 113, 122)))),
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

    let context_text = if let Some(pct) = app.context_usage {
        let pct_str = format!("{:.0}%", pct * 100.0);
        let ctx_color = if pct > 0.8 { Color::Red } else if pct > 0.6 { Color::Yellow } else { Color::Green };
        vec![Span::raw(" "), Span::styled(pct_str, Style::default().fg(ctx_color).bg(Color::Blue))]
    } else {
        vec![]
    };

    let sel_text = app.selected_message.map(|idx| {
        Span::styled(format!("#{}", idx), Style::default().fg(Color::Rgb(200, 200, 100)).bg(Color::Blue))
    });

    let mut spans = vec![
        Span::styled(" i-rs-code ", Style::default().fg(Color::White).bg(Color::Blue)),
        Span::styled(format!(" v{} ", app.version), Style::default().fg(Color::Cyan).bg(Color::Blue)),
        Span::raw("  "),
        Span::styled(dir, Style::default().fg(Color::White).bg(Color::Blue)),
    ];
    spans.extend(context_text);
    if let Some(s) = sel_text {
        spans.push(Span::raw(" "));
        spans.push(s);
    }
    spans.push(Span::raw(" "));
    spans.push(Span::styled("[?]", Style::default().fg(Color::Rgb(200, 200, 100)).bg(Color::Blue)));

    let text = Line::from(spans);
    let bar = Paragraph::new(text).style(Style::default().bg(Color::Blue));
    frame.render_widget(bar, area);
}

fn render_chat(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);

    let mut lines: Vec<Line> = Vec::new();
    let mut last_was_tool = false;

    for (msg_idx, msg) in app.messages.iter().enumerate() {
        let is_selected = app.selected_message == Some(msg_idx);
        let sel_prefix = if is_selected { "▶" } else { " " };

        match msg {
            AgentMessage::User { content } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(format!(" {} ", sel_prefix), Style::default().fg(Color::White).bg(Color::Blue)),
                    Span::styled(" You ", Style::default().fg(Color::White).bg(Color::Blue)),
                ]));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
            AgentMessage::Assistant { content, reasoning, tool_calls: _, reasoning_expanded } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(format!(" {} ", sel_prefix), if is_selected { Style::default().fg(Color::Yellow).bg(Color::Green) } else { Style::default().fg(Color::White).bg(Color::Green) }),
                    Span::styled(" AI ", Style::default().fg(Color::White).bg(Color::Green)),
                ]));
                if !reasoning.is_empty() {
                    if *reasoning_expanded {
                        lines.push(Line::from(vec![
                            Span::styled(" ▼ ", Style::default().fg(Color::Rgb(180, 180, 100)).bg(Color::Rgb(30, 30, 30))),
                            Span::styled(" 思考过程（按 r 折叠）", Style::default().fg(Color::Rgb(120, 120, 120)).add_modifier(Modifier::ITALIC)),
                        ]));
                        for line in reasoning.lines() {
                            lines.push(Line::from(Span::styled(
                                format!(" {}", line),
                                Style::default().fg(Color::Rgb(113, 113, 122)).add_modifier(Modifier::ITALIC),
                            )));
                        }
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(" ▶ ", Style::default().fg(Color::Rgb(180, 180, 100)).bg(Color::Rgb(30, 30, 30))),
                            Span::styled(" 思考过程（按 r 展开）", Style::default().fg(Color::Rgb(120, 120, 120)).add_modifier(Modifier::ITALIC)),
                        ]));
                    }
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
                    Span::styled(format!(" {} ", sel_prefix), if is_selected { Style::default().fg(Color::Yellow).bg(Color::Yellow) } else { Style::default().fg(Color::Black).bg(Color::Yellow) }),
                    Span::styled(format!(" {} ", glyph), Style::default().fg(Color::Black).bg(Color::Yellow)),
                    Span::styled(format!(" {} ", label), Style::default().fg(Color::Black).bg(Color::Yellow)),
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
                                Style::default().fg(Color::Rgb(180, 150, 100)),
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
                    Span::styled(format!(" {} ", sel_prefix), if is_selected { Style::default().fg(Color::Yellow).bg(Color::Magenta) } else { Style::default().fg(Color::White).bg(Color::Magenta) }),
                    Span::styled(" ✎ ", Style::default().fg(Color::White).bg(Color::Magenta)),
                    Span::styled(format!(" {} ", path), Style::default().fg(Color::White).bg(Color::Magenta)),
                ]));
                for line in summary.lines() {
                    if is_diff_like(line) {
                        let diff_spans = render_diff_line(line);
                        lines.push(Line::from(diff_spans));
                    } else {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(Color::Rgb(200, 160, 200)))));
                    }
                }
            }
            AgentMessage::System { content } => {
                last_was_tool = false;
                for line in content.lines() {
                    if line.starts_with("──") {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(Color::Rgb(80, 180, 80)).add_modifier(Modifier::BOLD))));
                    } else if line.starts_with("📄") || line.starts_with("🔧") {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(Color::Rgb(180, 180, 100)))));
                    } else {
                        lines.push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(Color::Rgb(100, 100, 120)))));
                    }
                }
            }
            AgentMessage::Separator { label } => {
                last_was_tool = false;
                let sep = if label.is_empty() { " ── done ── ".to_string() } else { format!(" ── {} ── ", label) };
                lines.push(Line::from(Span::styled(sep, Style::default().fg(Color::Rgb(60, 60, 70)))));
            }
        }
        lines.push(Line::from(""));
    }

    // Streaming block
    if let Some(ref s) = app.streaming {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default().fg(Color::White).bg(Color::Green)),
            Span::styled(" AI ", Style::default().fg(Color::White).bg(Color::Green)),
        ]));

        for tool in &s.tool_calls {
            lines.push(Line::from(""));
            let glyph = tool_glyph(&tool.name);
            lines.push(Line::from(vec![
                Span::styled(format!(" {} {} done", glyph, tool.name), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
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
                Span::styled(format!(" {} {} running...", glyph, tool.name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            let preview: String = tool.args.chars().take(area.width.saturating_sub(8) as usize).collect();
            for line in preview.lines() {
                lines.push(Line::from(Span::styled(format!("   └ {}", line), Style::default().fg(Color::DarkGray))));
            }
        }

        if !s.reasoning.is_empty() {
            let reasoning_lines: Vec<&str> = s.reasoning.lines().collect();
            let total = reasoning_lines.len();
            let start = total.saturating_sub(6);
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!(" ╎ … {} earlier lines", start),
                    Style::default().fg(Color::Rgb(150, 150, 100)).add_modifier(Modifier::ITALIC),
                )));
            }
            for line in &reasoning_lines[start..] {
                lines.push(Line::from(Span::styled(
                    format!(" ╎ {}", line),
                    Style::default().fg(Color::Rgb(113, 113, 122)).add_modifier(Modifier::ITALIC),
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
            lines.push(Line::from(vec![Span::styled(" ▊", Style::default().fg(Color::Green))]));
        } else if s.current_tool.is_none() && s.reasoning.is_empty() && s.content.is_empty() {
            lines.push(Line::from(vec![Span::styled(" ╎ ...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))]));
        }
    }

    // Bottom padding
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let scroll = if app.auto_scroll { max_scroll } else { max_scroll.saturating_sub(app.scroll_offset).min(max_scroll) };

    let hidden_msgs = app.messages.len().saturating_sub(1);
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    if !app.auto_scroll && (scroll > 0 || hidden_msgs > 0) {
        block = block.title(format!(" ↑ {} 条历史消息 ", hidden_msgs));
        block = block.title_alignment(Alignment::Center);
    }
    if let Some(idx) = app.selected_message {
        block = block.title(format!(" 📍 #{} ", idx));
        block = block.title_alignment(Alignment::Right);
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" ⚙ Status ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut items = Vec::new();
    let w = inner.width as usize;

    // --- Directory ---
    items.push(Line::from(Span::styled("─ Dir ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    let dir = if app.current_dir.len() > w.saturating_sub(2) {
        format!("..{}", &app.current_dir[app.current_dir.len().saturating_sub(w.saturating_sub(4))..])
    } else {
        app.current_dir.clone()
    };
    items.push(Line::from(Span::styled(format!(" {}", dir), Style::default().fg(Color::White))));
    items.push(Line::from(""));

    // --- Session & Version ---
    items.push(Line::from(Span::styled("─ Session ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    let sid = app.session_id.as_deref().unwrap_or("new");
    let sid_short = if sid.len() > 10 { format!("{}..", &sid[..10]) } else { sid.to_string() };
    items.push(Line::from(vec![
        Span::styled(" id ", Style::default().fg(Color::Rgb(100, 100, 110))),
        Span::styled(sid_short, Style::default().fg(Color::Rgb(150, 150, 150))),
    ]));
    items.push(Line::from(vec![
        Span::styled(" ver", Style::default().fg(Color::Rgb(100, 100, 110))),
        Span::styled(format!(" {}", app.version), Style::default().fg(Color::Cyan)),
    ]));
    items.push(Line::from(""));

    // --- Model ---
    items.push(Line::from(Span::styled("─ Model ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    let model = app.config.effective_model();
    let model_short = if model.len() > w.saturating_sub(2) { format!("{}..", &model[..w.saturating_sub(4)]) } else { model.to_string() };
    items.push(Line::from(vec![
        Span::styled(" ▸ ", Style::default().fg(Color::Cyan)),
        Span::styled(model_short, Style::default().fg(Color::White)),
    ]));
    items.push(Line::from(""));

    // --- Tokens ---
    items.push(Line::from(Span::styled("─ Tokens ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    let (tok_in, tok_out) = if let Some(ref s) = app.streaming {
        (app.token_usage.input, app.token_usage.output + s.content.len() as u32)
    } else {
        (app.token_usage.input, app.token_usage.output)
    };
    items.push(Line::from(Span::styled(
        format!(" IN  {}    OUT  {}", fmt_count(tok_in), fmt_count(tok_out)),
        Style::default().fg(Color::Rgb(150, 200, 150)),
    )));
    items.push(Line::from(""));

    // --- Mode + Messages ---
    let (mode_text, mode_color) = match app.mode {
        AppMode::Idle => ("idle", Color::Green),
        AppMode::Waiting => ("busy", Color::Yellow),
    };
    items.push(Line::from(Span::styled("─ Status ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    items.push(Line::from(vec![
        Span::styled(" mode ", Style::default().fg(Color::Rgb(100, 100, 110))),
        Span::styled(mode_text, Style::default().fg(mode_color)),
    ]));
    items.push(Line::from(vec![
        Span::styled(" msgs", Style::default().fg(Color::Rgb(100, 100, 110))),
        Span::styled(format!(" {}", app.messages.len()), Style::default().fg(Color::Rgb(150, 150, 150))),
    ]));
    items.push(Line::from(""));

    // --- Changes ---
    let change_count = app.file_changes.len();
    items.push(Line::from(Span::styled("─ Changes ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    if change_count == 0 {
        items.push(Line::from(Span::styled(" none", Style::default().fg(Color::DarkGray))));
    } else {
        items.push(Line::from(Span::styled(format!(" {} file(s)", change_count), Style::default().fg(Color::Yellow))));
        let mut changes: Vec<&String> = app.file_changes.iter().collect();
        changes.sort();
        for path in changes.iter().take(4) {
            let display = if path.len() > w.saturating_sub(4) {
                format!(" ..{}", &path[path.len().saturating_sub(w.saturating_sub(6))..])
            } else {
                format!(" {}", path)
            };
            items.push(Line::from(Span::styled(display, Style::default().fg(Color::Yellow))));
        }
        if change_count > 4 {
            items.push(Line::from(Span::styled(format!(" … +{}", change_count - 4), Style::default().fg(Color::DarkGray))));
        }
    }
    items.push(Line::from(""));

    // --- LSP ---
    items.push(Line::from(Span::styled("─ LSP ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    let (lsp_label, lsp_color) = if crate::runtime::LSP_INITIALIZED.load(Ordering::Relaxed) {
        ("✓ ready", Color::Green)
    } else {
        ("… waiting", Color::Yellow)
    };
    items.push(Line::from(vec![
        Span::styled(" ra ", Style::default().fg(Color::Rgb(100, 100, 110))),
        Span::styled(lsp_label, Style::default().fg(lsp_color)),
    ]));
    items.push(Line::from(""));

    // --- MCP ---
    items.push(Line::from(Span::styled("─ MCP ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
    items.push(Line::from(vec![
        Span::styled(" srv", Style::default().fg(Color::Rgb(100, 100, 110))),
        Span::styled(" config-based", Style::default().fg(Color::DarkGray)),
    ]));
    items.push(Line::from(""));

    // --- Streaming ---
    if let Some(ref s) = app.streaming {
        items.push(Line::from(Span::styled("─ Live ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
        items.push(Line::from(Span::styled(
            format!(" {}c · {} tools", s.content.len(), s.tool_calls.len()),
            Style::default().fg(Color::Cyan),
        )));
        if s.current_tool.is_some() {
            items.push(Line::from(Span::styled(" ▸ executing...", Style::default().fg(Color::Yellow))));
        }
        items.push(Line::from(""));
    }

    // --- Status ---
    if let Some(ref msg) = app.status_message {
        items.push(Line::from(Span::styled("─ Status ─", Style::default().fg(Color::Rgb(80, 80, 90)))));
        let preview: String = msg.chars().take(w.saturating_sub(4)).collect();
        items.push(Line::from(Span::styled(format!(" {}", preview), Style::default().fg(Color::Yellow))));
    }

    let paragraph = Paragraph::new(Text::from(items));
    frame.render_widget(paragraph, inner);
}

fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let border_color = if matches!(app.mode, AppMode::Waiting) {
        Color::DarkGray
    } else if app.input.content.is_empty() {
        Color::Rgb(80, 80, 90)
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let prefix = "> ";

    let hint = Line::from(Span::styled(
        "  [?] 键盘快捷键  [Enter] 发送  [Esc] 退出  [Ctrl+C] 取消  [Ctrl+Z] 撤销  [Ctrl+T] 转录  [[] []] 选择",
        Style::default().fg(Color::Rgb(80, 80, 90)),
    ));
    let lines: Vec<Line> = if matches!(app.mode, AppMode::Waiting) {
        vec![
            Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Rgb(113, 113, 122))),
                Span::styled(&app.input.content, Style::default().fg(Color::Rgb(113, 113, 122))),
            ]),
            hint,
        ]
    } else if app.input.content.is_empty() {
        vec![
            Line::from(Span::styled(format!("{}输入消息...", prefix), Style::default().fg(Color::Rgb(113, 113, 122)))),
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

    let input_widget = Paragraph::new(lines).block(Block::default());
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
