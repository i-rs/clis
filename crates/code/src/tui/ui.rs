use super::highlight::highlight_code_block;
use super::strings;
use crate::app::{AgentMessage, App, AppMode};
use crate::tui::colors::*;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::cell::RefCell;

thread_local! {
    static MSG_BLOCKS_CACHE: RefCell<(usize, Vec<Vec<Line<'static>>>)> =
        const { RefCell::new((0, Vec::new())) };
    static MSG_RECTS: RefCell<Vec<(u16, u16)>> = RefCell::new(Vec::new());
}

const SIDEBAR_WIDTH: u16 = 40;

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

use super::utils::short_path;

fn is_diff_output(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let has_unified = lines
        .iter()
        .any(|l| l.starts_with("--- ") || l.starts_with("+++ ") || l.starts_with("@@ "));
    let has_plus = lines.iter().any(|l| l.starts_with('+') && !l.starts_with("+++"));
    let has_minus = lines.iter().any(|l| l.starts_with('-') && !l.starts_with("---"));
    has_unified || (has_plus && has_minus)
}

fn render_diff_line(line: &str) -> Vec<Span<'static>> {
    if let Some(rest) = line.strip_prefix('+') {
        vec![
            Span::styled("+", Style::new().fg(c_diff_green()).bold()),
            Span::styled(rest.to_owned(), Style::new().fg(c_diff_green())),
        ]
    } else if let Some(rest) = line.strip_prefix('-') {
        vec![
            Span::styled("-", Style::new().fg(c_diff_red()).bold()),
            Span::styled(rest.to_owned(), Style::new().fg(c_diff_red())),
        ]
    } else if line.starts_with("@@") {
        vec![Span::styled(line.to_owned(), Style::new().fg(c_diff_hunk()))]
    } else {
        vec![Span::raw(line.to_owned())]
    }
}

fn render_ai_content(content: &str) -> Vec<Line<'static>> {
    let mut result = Vec::with_capacity(content.lines().count());
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buffer = String::new();

    for line in content.lines() {
        if line.starts_with("```") {
            if in_code {
                let lang = if code_lang.is_empty() {
                    None
                } else {
                    Some(code_lang.as_str())
                };
                let highlighted = highlight_code_block(&code_buffer, lang);
                for hl_line in highlighted {
                    let mut spans = Vec::with_capacity(hl_line.len() + 1);
                    spans.push(Span::raw(" "));
                    spans.extend(hl_line);
                    result.push(Line::from(spans));
                }
                code_buffer.clear();
                code_lang.clear();
                in_code = false;
            } else {
                in_code = true;
                code_lang = line.strip_prefix("```").map(|s| s.trim().to_string()).unwrap_or_default();
            }
            continue;
        }

        if in_code {
            code_buffer.push_str(line);
            code_buffer.push('\n');
        } else if let Some(heading) = line.strip_prefix("# ") {
            // H1 - prominent title
            result.push(Line::from(Span::styled(
                format!("    {}", heading),
                Style::new().fg(c_accent()).bold(),
            )));
        } else if let Some(heading) = line.strip_prefix("## ") {
            // H2 - section header
            result.push(Line::from(Span::styled(
                format!("    {}", heading),
                Style::new().fg(c_cyan()).bold(),
            )));
        } else if let Some(item) = line.strip_prefix("- ") {
            // Bullet list
            result.push(Line::from(vec![
                Span::styled("    ", Style::new().fg(c_text())),
                Span::styled("• ", Style::new().fg(c_accent())),
                Span::styled(item.to_string(), Style::new().fg(c_text())),
            ]));
        } else if let Some(item) = line.strip_prefix("  - ") {
            // Nested bullet
            result.push(Line::from(vec![
                Span::styled("      ", Style::new().fg(c_text())),
                Span::styled("◦ ", Style::new().fg(c_dim())),
                Span::styled(item.to_string(), Style::new().fg(c_text())),
            ]));
        } else {
            result.push(Line::from(Span::styled(
                format!("    {}", line),
                Style::new().fg(c_text()),
            )));
        }
    }

    if in_code && !code_buffer.is_empty() {
        let lang = if code_lang.is_empty() {
            None
        } else {
            Some(code_lang.as_str())
        };
        let highlighted = highlight_code_block(&code_buffer, lang);
        for hl_line in highlighted {
            let mut spans = Vec::with_capacity(hl_line.len() + 1);
            spans.push(Span::raw(" "));
            spans.extend(hl_line);
            result.push(Line::from(spans));
        }
    }

    result
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let input_lines = (app.input.content.lines().count() + 1).clamp(2, 8) as u16 + 2;

    let [title_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    render_title_bar(frame, title_area, app);

    // Layout: horizontal split into chat body + sidebar
    let [chat_body, sidebar_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(SIDEBAR_WIDTH)]).areas(body);
    // Chat body: vertical split into chat area + input area
    let [chat_area, input_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(input_lines)]).areas(chat_body);

    render_chat(frame, chat_area, app);
    render_input_bar(frame, input_area, app);
    crate::tui::sidebar::render_sidebar(frame, sidebar_area, app);

    // Picker (if visible) is rendered as a floating overlay at the bottom of
    // the chat area, so it does NOT push the input bar. The picker auto-scales
    // to the chat area and scrolls to keep the selected item visible.
    if app.show_slash_picker {
        let commands = filtered_slash_commands(app);
        if !commands.is_empty() {
            // Leave at least 3 lines of chat visible above the picker.
            let max_picker = chat_area.height.saturating_sub(3);
            let desired = commands.len() as u16 + 2; // +2 for top border + padding
            let picker_height = desired.min(max_picker).max(3);
            // Anchor the picker just above the input bar, within the chat area
            let anchor_y = chat_area.y + chat_area.height.saturating_sub(picker_height);
            let picker_area = Rect {
                x: chat_area.x,
                y: anchor_y,
                width: chat_area.width,
                height: picker_height,
            };
            render_slash_picker(frame, picker_area, app);
        }
    }

    if app.show_shortcuts {
        super::overlays::render_shortcuts_overlay(frame, area);
    }

    if app.show_theme_picker {
        super::overlays::render_theme_picker(frame, area, app);
    }

    if app.show_debug {
        super::overlays::render_debug_overlay(frame, area, app);
    }
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let dir = short_path(&app.current_dir);

    let context_text = if let Some(pct) = app.context_usage {
        let pct_str = format!("{:.0}%", pct * 100.0);
        let ctx_color = if pct > 0.8 {
            c_red()
        } else if pct > 0.6 {
            c_orange()
        } else {
            c_green()
        };
        vec![Span::raw("  "), Span::styled(pct_str, Style::new().fg(ctx_color))]
    } else {
        vec![]
    };

    let sel_text = app.selected_message.map(|idx| {
        Span::styled(format!(" #{} ", idx), Style::new().fg(c_cyan()))
    });

    let mut spans = vec![
        Span::styled(" i-rs-code ", Style::new().fg(c_accent()).bold()),
        Span::styled(format!("v{}", app.version), Style::new().fg(c_dim())),
        Span::raw("  "),
        Span::styled(dir, Style::new().fg(c_dim())),
    ];
    spans.extend(context_text);
    if let Some(s) = sel_text {
        spans.push(s);
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled("[?]", Style::new().fg(c_orange())));

    let text = Line::from(spans);
    let title_bg = Style::new().bg(c_bg_title());
    frame.render_widget(Clear, area);
    let bar = Paragraph::new(text).style(title_bg);
    frame.render_widget(bar, area);

    // Subtle bottom border
    let sep_line = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::new().fg(c_border()),
    ))]));
    let sep_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    frame.render_widget(sep_line, sep_area);
}

fn msg_bg(msg: &AgentMessage) -> Color {
    match msg {
        AgentMessage::User { .. } => c_bg_user(),
        AgentMessage::Assistant { .. } => c_bg_ai(),
        AgentMessage::ToolResult { .. } => c_bg_tool(),
        AgentMessage::System { .. } => c_bg_system(),
        AgentMessage::FileEdit { .. } => c_bg_file(),
        AgentMessage::Separator { .. } => c_bg(),
    }
}

fn build_msg_lines(msg: &AgentMessage, is_selected: bool) -> Vec<Line<'static>> {
    match msg {
        AgentMessage::User { content } => {
            let mut lines = vec![Line::from("")];
            // OpenCode style: clean header
            lines.push(Line::from(vec![
                Span::styled("  ", Style::new().fg(c_muted())),
                Span::styled("▎", Style::new().fg(c_cyan())),
                Span::styled(" You", Style::new().fg(c_text()).bold()),
            ]));
            for line in content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("    {}", line),
                    Style::new().fg(c_text()),
                )));
            }
            lines.push(Line::from(""));
            lines
        }
        AgentMessage::Assistant {
            content,
            reasoning,
            tool_calls: _,
            reasoning_expanded,
            ..
        } => {
            let mut lines = vec![Line::from("")];
            // OpenCode style: clean, minimal header
            lines.push(Line::from(vec![
                Span::styled("  ", Style::new().fg(c_muted())),
                Span::styled("▎", Style::new().fg(c_green())),
                Span::styled(" Assistant", Style::new().fg(c_text()).bold()),
            ]));
            let content_lines = render_ai_content(content);
            lines.extend(content_lines);

            if !reasoning.is_empty() {
                lines.push(Line::from(""));
                if *reasoning_expanded {
                    // Expanded: "▎ Thought:" header with bulleted list
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::new().fg(c_muted())),
                        Span::styled("▎ ", Style::new().fg(c_yellow())),
                        Span::styled("Thought", Style::new().fg(c_yellow()).bold()),
                    ]));
                    for rline in reasoning.lines() {
                        let trimmed = rline.trim_start();
                        if trimmed.is_empty() {
                            continue;
                        }
                        // Try to detect command-like lines for special formatting
                        if let Some(cmd) = trimmed.strip_prefix("Explore Task") {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::new().fg(c_muted())),
                                Span::styled("▸ ", Style::new().fg(c_accent())),
                                Span::styled("Explore Task", Style::new().fg(c_text()).bold()),
                                Span::styled(cmd.to_string(), Style::new().fg(c_dim())),
                            ]));
                        } else if let Some(cmd) = trimmed.strip_prefix("Exa Web Search") {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::new().fg(c_muted())),
                                Span::styled("▸ ", Style::new().fg(c_accent())),
                                Span::styled("Exa Web Search", Style::new().fg(c_text()).bold()),
                                Span::styled(cmd.to_string(), Style::new().fg(c_dim())),
                            ]));
                        } else if let Some(rest) = trimmed.strip_prefix("Read ") {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::new().fg(c_muted())),
                                Span::styled("↳ ", Style::new().fg(c_muted())),
                                Span::styled(rest.to_string(), Style::new().fg(c_dim())),
                            ]));
                        } else if trimmed.starts_with("ctrl+x")
                            || trimmed.starts_with("ctrl+")
                        {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::new().fg(c_muted())),
                                Span::styled("▸ ", Style::new().fg(c_accent())),
                                Span::styled(trimmed.to_string(), Style::new().fg(c_text())),
                            ]));
                        } else {
                            lines.push(Line::from(Span::styled(
                                format!("    {}", trimmed),
                                Style::new().fg(c_dim()),
                            )));
                        }
                    }
                } else {
                    // Collapsed: just hint
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::new().fg(c_muted())),
                        Span::styled("▎ ", Style::new().fg(c_yellow())),
                        Span::styled("Thought", Style::new().fg(c_yellow())),
                        Span::styled("  (press to expand)", Style::new().fg(c_muted())),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines
        }
        AgentMessage::ToolResult {
            content,
            diff,
            step,
            total_steps,
            collapsed,
            ..
        } => {
            let (tool_name, tool_result) = content.split_once('\n').unwrap_or(("", content));
            let glyph = tool_glyph(tool_name);
            let label = if tool_name.is_empty() {
                "Tool".to_string()
            } else {
                tool_name.to_string()
            };

            let step_str = if *total_steps > 1 {
                format!("[{}/{}] ", step, total_steps)
            } else {
                String::new()
            };

            let mut lines = vec![Line::from("")];

            // OpenCode style: clean, single-line header
            if *collapsed {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::new().fg(c_muted())),
                    Span::styled("▸", Style::new().fg(c_muted())),
                    Span::styled(" ", Style::new()),
                    Span::styled(step_str.clone(), Style::new().fg(c_muted())),
                    Span::styled(
                        format!("{}{}", glyph, label),
                        Style::new().fg(c_text()).bold(),
                    ),
                    Span::styled("  completed", Style::new().fg(c_muted())),
                ]));
                if !tool_result.is_empty() {
                    let preview: String = tool_result.chars().take(400).collect();
                    let result_lines: Vec<&str> = preview.lines().collect();
                    let max_lines = 3;
                    for (i, line) in result_lines.iter().enumerate().take(max_lines) {
                        let truncated: String = line.chars().take(120).collect();
                        let mut content = truncated;
                        if line.chars().count() > 120 {
                            content.push('…');
                        }
                        lines.push(Line::from(Span::styled(
                            format!("    {}", content),
                            Style::new().fg(c_dim()),
                        )));
                        if i == max_lines - 1 && preview.len() < tool_result.len() {
                            lines.push(Line::from(Span::styled(
                                format!("    … {} more bytes", tool_result.len().saturating_sub(preview.len())),
                                Style::new().fg(c_muted()),
                            )));
                        }
                    }
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::new().fg(c_muted())),
                    Span::styled("▾", Style::new().fg(c_orange())),
                    Span::styled(" ", Style::new()),
                    Span::styled(step_str, Style::new().fg(c_muted())),
                    Span::styled(
                        format!("{}{}", glyph, label),
                        Style::new().fg(c_orange()).bold(),
                    ),
                ]));

                let tool_has_diff = is_diff_output(tool_result);
                if !tool_result.is_empty() {
                    let preview: String = tool_result.chars().take(1200).collect();
                    let result_lines: Vec<&str> = preview.lines().collect();
                    for line in result_lines.iter().take(12) {
                        if tool_has_diff {
                            let mut spans = vec![Span::styled("    ", Style::new().fg(c_muted()))];
                            spans.extend(render_diff_line(line));
                            lines.push(Line::from(spans));
                        } else {
                            lines.push(Line::from(Span::styled(
                                format!("    {}", line),
                                Style::new().fg(c_tool_output()),
                            )));
                        }
                    }
                    if preview.len() < tool_result.len() || tool_result.lines().count() > 12 {
                        lines.push(Line::from(Span::styled(
                            format!(
                                "    … {} more bytes",
                                tool_result.len().saturating_sub(preview.len())
                            ),
                            Style::new().fg(c_muted()),
                        )));
                    }
                }
                if let Some(diff_text) = diff && !diff_text.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("    ", Style::new().fg(c_muted())),
                        Span::styled("─ diff ─", Style::new().fg(c_dim())),
                    ]));
                    for diff_line in diff_text.lines().take(12) {
                        let mut spans = vec![Span::styled("    ", Style::new().fg(c_muted()))];
                        spans.extend(render_diff_line(diff_line));
                        lines.push(Line::from(spans));
                    }
                    if diff_text.lines().count() > 12 {
                        lines.push(Line::from(Span::styled(
                            format!(
                                "    ... +{} more lines",
                                diff_text.lines().count().saturating_sub(12)
                            ),
                            Style::new().fg(c_muted()),
                        )));
                    }
                }
            }
            lines.push(Line::from(""));
            lines
        }
        AgentMessage::FileEdit { path, summary } => {
            let prefix = if is_selected { "▎" } else { " " };
            let mut lines = vec![Line::from("")];
            // OpenCode style: clean, simple header
            lines.push(Line::from(vec![
                Span::styled("  ", Style::new().fg(c_muted())),
                Span::styled(prefix, Style::new().fg(c_purple())),
                Span::styled(" ", Style::new()),
                Span::styled("✎ ", Style::new().fg(c_purple())),
                Span::styled(path.clone(), Style::new().fg(c_file_edit()).bold()),
            ]));
            for diff_line in summary.lines() {
                let mut spans = vec![Span::styled("    ", Style::new().fg(c_muted()))];
                spans.extend(render_diff_line(diff_line));
                lines.push(Line::from(spans));
            }
            lines.push(Line::from(""));
            lines
        }
        AgentMessage::System { content } => {
            let mut lines = vec![Line::from("")];
            for line in content.lines() {
                if line.starts_with("──") {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", line),
                        Style::new().fg(c_summary()).bold(),
                    )));
                } else if line.starts_with("📄") || line.starts_with("🔧") {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", line),
                        Style::new().fg(c_orange()),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", line),
                        Style::new().fg(c_dim()),
                    )));
                }
            }
            lines.push(Line::from(""));
            lines
        }
        AgentMessage::Separator { label } => {
            let sep = if label.is_empty() {
                "── ──".to_string()
            } else {
                format!("── {} ──", label)
            };
            vec![Line::from(Span::styled(
                format!("    {}", sep),
                Style::new().fg(c_border()),
            ))]
        }
    }
}

fn build_all_msg_blocks(app: &App) -> Vec<Vec<Line<'static>>> {
    app.messages
        .iter()
        .enumerate()
        .map(|(idx, msg)| build_msg_lines(msg, app.selected_message == Some(idx)))
        .collect()
}

fn render_chat(frame: &mut Frame, area: Rect, app: &App) {
    let mut blocks: Vec<Vec<Line<'static>>> = MSG_BLOCKS_CACHE.with(|cache| {
        let (cached_gen, cached) = &mut *cache.borrow_mut();
        if *cached_gen != app.message_generation {
            *cached = build_all_msg_blocks(app);
            *cached_gen = app.message_generation;
        }
        cached.clone()
    });

    if let Some(ref s) = app.streaming {
        let mut stream_lines: Vec<Line<'static>> = vec![Line::from(vec![
            Span::styled("  ", Style::new().fg(c_muted())),
            Span::styled("▎", Style::new().fg(c_green())),
            Span::styled(" Assistant", Style::new().fg(c_text()).bold()),
        ])];

        for tool in &s.tool_calls {
            let glyph = tool_glyph(&tool.name);
            stream_lines.push(Line::from(vec![
                Span::styled("▷ ", Style::new().fg(c_dim())),
                Span::styled(
                    format!("✓ {} {}", glyph, tool.name),
                    Style::new().fg(c_green()).bold(),
                ),
            ]));
        }

        if let Some(ref tool) = s.current_tool {
            let glyph = tool_glyph(&tool.name);
            stream_lines.push(Line::from(vec![
                Span::styled("● ", Style::new().fg(c_accent())),
                Span::styled(
                    format!("{} {} running...", glyph, tool.name),
                    Style::new().fg(c_accent()).bold(),
                ),
            ]));
        }

        if !s.reasoning.is_empty() {
            let reasoning_lines: Vec<&str> = s.reasoning.lines().collect();
            let total = reasoning_lines.len();
            let show_count = if s.content.is_empty() { 6.min(total) } else { 3.min(total) };
            let start = total.saturating_sub(show_count);
            stream_lines.push(Line::from(Span::styled(
                "▼ 思考过程",
                Style::new().fg(c_yellow()),
            )));
            if start > 0 {
                stream_lines.push(Line::from(Span::styled(
                    format!("│ … {} earlier lines", start),
                    Style::new().fg(c_dim()).italic(),
                )));
            }
            for line in &reasoning_lines[start..] {
                stream_lines.push(Line::from(Span::styled(
                    format!("│ {}", line),
                    Style::new().fg(c_dim()).italic(),
                )));
            }
            stream_lines.push(Line::from(Span::styled("╰", Style::new().fg(c_border()))));
        }

        if !s.content.is_empty() {
            let content_lines = render_ai_content(&s.content);
            stream_lines.extend(content_lines);
        }

        if s.current_tool.is_none() && !s.content.is_empty() {
            stream_lines.push(Line::from(vec![Span::styled(" ▊", Style::new().fg(c_green()))]));
        } else if s.current_tool.is_none()
            && s.reasoning.is_empty()
            && s.tool_calls.is_empty()
            && s.content.is_empty()
        {
            stream_lines.push(Line::from(vec![Span::styled(" ⏳", Style::new().fg(c_dim()))]));
        }

        blocks.push(stream_lines);
    }

    // Build a flat list: (block_idx_in_messages, lines, bg_color)
    let gap: usize = 1;
    let msg_count = app.messages.len();
    let total_height: usize = blocks.iter().map(|b| b.len()).sum::<usize>()
        + blocks.len().saturating_sub(1) * gap
        + 4;

    let max_scroll = total_height.saturating_sub(area.height as usize);
    let scroll = if app.auto_scroll {
        max_scroll
    } else {
        app.scroll_offset.min(max_scroll)
    };

    frame.render_widget(Clear, area);

    // Walk virtual coordinates, render only visible blocks
    let mut rects: Vec<(u16, u16)> = Vec::with_capacity(msg_count);
    let mut virtual_y: usize = 0;
    let mut screen_y: u16 = area.y;
    let area_bottom = area.y + area.height;

    for (block_idx, block_lines) in blocks.iter().enumerate() {
        let block_height = block_lines.len();
        let is_msg = block_idx < msg_count;
        let bg = if is_msg {
            msg_bg(&app.messages[block_idx])
        } else {
            c_bg_ai()
        };

        // gap before this block (except first)
        if block_idx > 0 {
            virtual_y += gap;
            if virtual_y > scroll && screen_y < area_bottom {
                screen_y = screen_y.saturating_add(1);
            }
        }

        let block_end = virtual_y + block_height;

        // Entirely above viewport — skip
        if block_end <= scroll {
            if is_msg {
                rects.push((0, 0));
            }
            virtual_y += block_height;
            continue;
        }

        let skip = scroll.saturating_sub(virtual_y);
        let visible_count = block_height.saturating_sub(skip);
        let remaining = (area_bottom - screen_y) as usize;
        if remaining == 0 {
            if is_msg {
                rects.push((0, 0));
            }
            virtual_y += block_height;
            continue;
        }

        let render_count = visible_count.min(remaining) as u16;
        let visible_slice = &block_lines[skip..];

        let block_area = Rect {
            x: area.x,
            y: screen_y,
            width: area.width,
            height: render_count,
        };

        let para = Paragraph::new(Text::from(visible_slice.to_vec()))
            .style(Style::new().bg(bg))
            .block(Block::default().padding(ratatui::widgets::Padding::new(0, 0, 0, 0)))
            .wrap(Wrap { trim: false });
        frame.render_widget(para, block_area);

        if is_msg {
            rects.push((block_area.y, block_area.height));
        }

        screen_y += render_count;
        virtual_y += block_height;
    }

    MSG_RECTS.with(|r| *r.borrow_mut() = rects);

    // Scroll indicator
    if !app.auto_scroll && app.messages.len() > 1 {
        let hint = strings::scrolled_up_hint(app.messages.len().saturating_sub(1));
        let hint_line = Line::from(Span::styled(hint, Style::new().fg(c_dim())));
        let hint_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let hint_para = Paragraph::new(Text::from(vec![hint_line])).style(Style::new().bg(c_bg()));
        frame.render_widget(hint_para, hint_area);
    }
}

/// Find message index from a screen row. Uses cached rects from last render.
pub fn find_message_idx_from_screen(screen_row: u16) -> Option<usize> {
    MSG_RECTS.with(|r| {
        let rects = r.borrow();
        for (idx, &(y, h)) in rects.iter().enumerate() {
            if h > 0 && screen_row >= y && screen_row < y + h {
                return Some(idx);
            }
        }
        None
    })
}

fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let prefix = "▎ ";
    let hint = Line::from(Span::styled(strings::STATUS_BAR, Style::new().fg(c_muted())));
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
                format!("{}{}", prefix, strings::INPUT_PLACEHOLDER),
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

fn render_slash_picker(frame: &mut Frame, area: Rect, app: &App) {
    let commands = filtered_slash_commands(app);
    if commands.is_empty() {
        return;
    }

    // Visible rows inside the block (subtract top border + 1 padding row).
    let visible_rows = area.height.saturating_sub(2) as usize;
    let total = commands.len();

    // Auto-scroll so the selected item is always visible.
    let selected = app.slash_selected.min(total.saturating_sub(1));
    let (start, end) = if total <= visible_rows || visible_rows == 0 {
        (0usize, total.min(visible_rows))
    } else {
        // Try to keep selection roughly centered when scrolling.
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
                if is_selected { Style::new().fg(c_accent()) } else { Style::new().fg(c_muted()) },
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

    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(c_border_active()))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0))
        .style(Style::new().bg(c_bg_surface()));

    let paragraph = Paragraph::new(Text::from(items)).block(block);
    frame.render_widget(paragraph, area);
}
