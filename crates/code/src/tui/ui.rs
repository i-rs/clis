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

use super::utils::short_path;

fn is_diff_output(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let has_unified = lines
        .iter()
        .any(|l| l.starts_with("--- ") || l.starts_with("+++ ") || l.starts_with("@@ "));
    let has_plus = lines
        .iter()
        .any(|l| l.starts_with('+') && !l.starts_with("+++"));
    let has_minus = lines
        .iter()
        .any(|l| l.starts_with('-') && !l.starts_with("---"));
    has_unified || (has_plus && has_minus)
}

fn render_diff_line(line: &str) -> Vec<Span<'static>> {
    if let Some(rest) = line.strip_prefix('+') {
        vec![
            Span::styled("+", Style::new().green().bold()),
            Span::styled(rest.to_owned(), Style::new().fg(C_DIFF_GREEN)),
        ]
    } else if let Some(rest) = line.strip_prefix('-') {
        vec![
            Span::styled("-", Style::new().red().bold()),
            Span::styled(rest.to_owned(), Style::new().fg(C_DIFF_RED)),
        ]
    } else if line.starts_with("@@") {
        vec![Span::styled(line.to_owned(), Style::new().cyan())]
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
                code_lang = line.strip_prefix("```")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
            continue;
        }

        if in_code {
            code_buffer.push_str(line);
            code_buffer.push('\n');
        } else {
            result.push(Line::from(Span::raw(format!(" {}", line))));
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

    let [chat_body, sidebar_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(SIDEBAR_WIDTH)]).areas(body);

    if app.show_slash_picker {
        let picker_height = filtered_slash_commands(app).len().min(8) as u16 + 2;
        let [chat_area, picker_area, input_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(picker_height),
            Constraint::Length(input_lines),
        ])
        .areas(chat_body);
        render_chat(frame, chat_area, app);
        render_slash_picker(frame, picker_area, app);
        render_input_bar(frame, input_area, app);
    } else {
        let [chat_area, input_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(input_lines)])
                .areas(chat_body);
        render_chat(frame, chat_area, app);
        render_input_bar(frame, input_area, app);
    }
    crate::tui::sidebar::render_sidebar(frame, sidebar_area, app);

    if app.show_shortcuts {
        super::overlays::render_shortcuts_overlay(frame, area);
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
            Color::Red
        } else if pct > 0.6 {
            Color::Yellow
        } else {
            Color::Green
        };
        vec![
            Span::raw("  "),
            Span::styled(pct_str, Style::new().fg(ctx_color)),
        ]
    } else {
        vec![]
    };

    let sel_text = app
        .selected_message
        .map(|idx| Span::styled(format!(" #{} ", idx), Style::new().fg(C_YELLOW)));

    let mut spans = vec![
        Span::styled(" i-rs-code ", Style::new().fg(C_GREEN).bold()),
        Span::styled(format!("v{}", app.version), Style::new().fg(C_ACCENT)),
        Span::raw("  "),
        Span::styled(dir, Style::new().fg(C_DIM)),
    ];
    spans.extend(context_text);
    if let Some(s) = sel_text {
        spans.push(s);
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled("[?]", Style::new().fg(C_YELLOW)));

    let text = Line::from(spans);
    let title_bg = Style::new().bg(C_BG_TITLE);
    frame.render_widget(Clear, area);
    let bar = Paragraph::new(text).style(title_bg);
    frame.render_widget(bar, area);

    // Bottom separator
    let sep_line = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::new().fg(C_RAIL),
    ))]));
    let sep_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    frame.render_widget(sep_line, sep_area);
}

fn render_chat(frame: &mut Frame, area: Rect, app: &App) {
    let estimated: usize = app.messages.iter().map(|m| match m {
        AgentMessage::User { content } | AgentMessage::System { content } => {
            content.lines().count().max(1) + 1
        }
        AgentMessage::Assistant { content, .. } => content.lines().count().max(1) + 2,
        AgentMessage::ToolResult { .. } => 8,
        AgentMessage::FileEdit { summary, .. } => summary.lines().count().max(1) + 1,
        AgentMessage::Separator { .. } => 1,
    }).sum::<usize>() + 40;
    let mut lines: Vec<Line> = Vec::with_capacity(estimated);
    let mut last_was_tool = false;

    for (msg_idx, msg) in app.messages.iter().enumerate() {
        let is_selected = app.selected_message == Some(msg_idx);
        let sel_prefix = if is_selected { "▶" } else { " " };

        match msg {
            AgentMessage::User { content } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::new().fg(C_ACCENT)),
                    Span::styled("▎You", Style::new().fg(Color::Rgb(59, 130, 246)).bold()),
                ]));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(format!(" {}", line))));
                }
            }
            AgentMessage::Assistant {
                content,
                reasoning,
                tool_calls: _,
                reasoning_expanded,
            } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::new().fg(C_YELLOW)),
                    Span::styled("▎AI", Style::new().fg(C_GREEN).bold()),
                ]));
                if !reasoning.is_empty() {
                    lines.push(Line::from(Span::styled(
                        if *reasoning_expanded {
                            strings::REASONING_VISIBLE
                        } else {
                            strings::REASONING_HIDDEN
                        },
                        Style::new().fg(C_YELLOW),
                    )));
                }
                let content_lines = render_ai_content(content);
                lines.extend(content_lines);
            }
            AgentMessage::ToolResult { content, diff } => {
                let (tool_name, tool_result) = content.split_once('\n').unwrap_or(("", content));
                let glyph = tool_glyph(tool_name);
                let label = if tool_name.is_empty() {
                    "Tool"
                } else {
                    tool_name
                };

                let (rail_top, rail_mid) = if last_was_tool {
                    ("│", "│")
                } else {
                    ("╭", "│")
                };

                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::new().fg(C_YELLOW)),
                    Span::styled(format!(" {} ", glyph), Style::new().fg(C_YELLOW)),
                    Span::styled(label, Style::new().fg(C_YELLOW).bold()),
                ]));
                let tool_has_diff = is_diff_output(tool_result);
                if !tool_result.is_empty() {
                    let preview: String = tool_result.chars().take(1200).collect();
                    let result_lines: Vec<&str> = preview.lines().collect();
                    for (i, line) in result_lines.iter().enumerate().take(12) {
                        let prefix = if i == result_lines.len().saturating_sub(1).min(11) {
                            "╰"
                        } else {
                            rail_mid
                        };
                        if tool_has_diff {
                            let diff_spans = render_diff_line(line);
                            let mut spans = vec![Span::styled(
                                format!(" {} ", prefix),
                                Style::new().fg(Color::DarkGray),
                            )];
                            spans.extend(diff_spans);
                            lines.push(Line::from(spans));
                        } else {
                            lines.push(Line::from(Span::styled(
                                format!(" {} {}", prefix, line),
                                Style::new().fg(C_TOOL_OUTPUT),
                            )));
                        }
                    }
                    if preview.len() < tool_result.len() || tool_result.lines().count() > 12 {
                        lines.push(Line::from(Span::styled(
                            format!(
                                " {} … {} more bytes (Ctrl+T)",
                                rail_top,
                                tool_result.len().saturating_sub(preview.len())
                            ),
                            Style::new().fg(Color::DarkGray),
                        )));
                    }
                }
                if let Some(diff_text) = diff
                    && !diff_text.is_empty()
                {
                    lines.push(Line::from(Span::styled(
                        format!(" {} ─ diff ─", rail_mid),
                        Style::new().fg(C_DIM),
                    )));
                    for line in diff_text.lines().take(12) {
                        let diff_spans = render_diff_line(line);
                        lines.push(Line::from(diff_spans));
                    }
                    if diff_text.lines().count() > 12 {
                        lines.push(Line::from(Span::styled(
                            format!(
                                " {} ... +{} more lines",
                                rail_mid,
                                diff_text.lines().count().saturating_sub(12)
                            ),
                            Style::new().fg(Color::DarkGray),
                        )));
                    }
                }
                last_was_tool = true;
            }
            AgentMessage::FileEdit { path, summary } => {
                last_was_tool = false;
                lines.push(Line::from(vec![
                    Span::styled(sel_prefix, Style::new().fg(C_FILE_EDIT)),
                    Span::styled(format!(" ✎ {} ", path), Style::new().fg(C_FILE_EDIT).bold()),
                ]));
                for line in summary.lines() {
                    let diff_spans = render_diff_line(line);
                    lines.push(Line::from(diff_spans));
                }
            }
            AgentMessage::System { content } => {
                last_was_tool = false;
                for line in content.lines() {
                    if line.starts_with("──") {
                        lines.push(Line::from(Span::styled(
                            format!(" {}", line),
                            Style::new().fg(C_SUMMARY_GREEN).bold(),
                        )));
                    } else if line.starts_with("📄") || line.starts_with("🔧") {
                        lines.push(Line::from(Span::styled(
                            format!(" {}", line),
                            Style::new().fg(C_YELLOW),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            format!(" {}", line),
                            Style::new().fg(C_DIM),
                        )));
                    }
                }
            }
            AgentMessage::Separator { label } => {
                last_was_tool = false;
                let sep = if label.is_empty() {
                    " ── done ── ".to_string()
                } else {
                    format!(" ── {} ── ", label)
                };
                lines.push(Line::from(Span::styled(sep, Style::new().fg(C_SEP))));
            }
        }
        lines.push(Line::from(""));
    }

    // Streaming block
    if let Some(ref s) = app.streaming {
        lines.push(Line::from(vec![Span::styled(
            "▎AI",
            Style::new().fg(C_GREEN).bold(),
        )]));

        for tool in &s.tool_calls {
            lines.push(Line::from(""));
            let glyph = tool_glyph(&tool.name);
            lines.push(Line::from(vec![Span::styled(
                format!(" {} {} done", glyph, tool.name),
                Style::new().fg(C_GREEN).bold(),
            )]));
            if let Some(ref result) = tool.result {
                let preview: String = result.chars().take(300).collect();
                for line in preview.lines().take(4) {
                    lines.push(Line::from(Span::styled(
                        format!("   └ {}", line),
                        Style::new().fg(Color::DarkGray),
                    )));
                }
                if preview.len() < result.len() || result.lines().count() > 4 {
                    lines.push(Line::from(Span::styled(
                        "   └ ...",
                        Style::new().fg(Color::DarkGray),
                    )));
                }
            }
        }

        if let Some(ref tool) = s.current_tool {
            lines.push(Line::from(""));
            let glyph = tool_glyph(&tool.name);
            lines.push(Line::from(vec![Span::styled(
                format!(" {} {} running...", glyph, tool.name),
                Style::new().fg(C_ACCENT).bold(),
            )]));
            let preview: String = tool
                .args
                .chars()
                .take(area.width.saturating_sub(8) as usize)
                .collect();
            for line in preview.lines() {
                lines.push(Line::from(Span::styled(
                    format!("   └ {}", line),
                    Style::new().fg(Color::DarkGray),
                )));
            }
            if let Some(ref diff) = tool.diff {
                let diff_lines: Vec<&str> = diff.lines().collect();
                let show = if diff_lines.len() > 10 {
                    &diff_lines[..10]
                } else {
                    &diff_lines[..]
                };
                for line in show {
                    let (sign, _rest) = line.split_at(1);
                    let style = match sign {
                        "+" => Style::new().fg(Color::Green),
                        "-" => Style::new().fg(Color::Red),
                        _ => Style::new().fg(Color::DarkGray),
                    };
                    lines.push(Line::from(Span::styled(format!("   {}", line), style)));
                }
                if diff_lines.len() > 10 {
                    lines.push(Line::from(Span::styled(
                        "   ... (diff truncated)",
                        Style::new().fg(Color::DarkGray),
                    )));
                }
            }
        }

        if !s.reasoning.is_empty() {
            let reasoning_lines: Vec<&str> = s.reasoning.lines().collect();
            let total = reasoning_lines.len();
            // Show last 3-6 lines as live preview
            let show_count = if s.content.is_empty() {
                6.min(total)
            } else {
                3.min(total)
            };
            let start = total.saturating_sub(show_count);
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!(" ╎ … {} earlier lines", start),
                    Style::new().fg(C_DIM).italic(),
                )));
            }
            for line in &reasoning_lines[start..] {
                lines.push(Line::from(Span::styled(
                    format!(" ╎ {}", line),
                    Style::new().fg(C_DIM).italic(),
                )));
            }
        }

        if !s.content.is_empty() {
            let content_lines = render_ai_content(&s.content);
            lines.extend(content_lines);
        }

        if s.current_tool.is_none() && !s.content.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                " ▊",
                Style::new().fg(C_GREEN),
            )]));
        } else if s.current_tool.is_none() && s.reasoning.is_empty() && s.tool_calls.is_empty() && s.content.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                " ⏳",
                Style::new().fg(C_DIM),
            )]));
        }
    }

    // Bottom padding (prevent messages touching input bar)
    for _ in 0..4 {
        lines.push(Line::from(""));
    }

    let content_len = lines.len();
    let max_scroll = content_len.saturating_sub(area.height as usize);
    let scroll = if app.auto_scroll {
        max_scroll
    } else {
        app.scroll_offset.min(max_scroll)
    };

    // Show "↑ N 条历史消息" at the top if scrolled away from bottom
    let hidden_msgs = app.messages.len().saturating_sub(1);
    if !app.auto_scroll && hidden_msgs > 0 {
        let mut header = vec![Line::from(Span::styled(
            strings::scrolled_up_hint(hidden_msgs),
            Style::new().fg(C_DIM),
        ))];
        if let Some(idx) = app.selected_message {
            header.push(Line::from(Span::styled(
                format!(" 📍 #{}  ", idx),
                Style::new().fg(C_YELLOW),
            )));
        }
        for h in header {
            lines.insert(0, h);
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::new().bg(C_BG))
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(1)))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let prefix = "> ";
    let hint = Line::from(Span::styled(strings::STATUS_BAR, Style::new().fg(C_DIM)));
    let lines: Vec<Line> = if matches!(app.mode, AppMode::Waiting) {
        vec![
            Line::from(vec![
                Span::styled("⏳ ", Style::new().fg(C_DIM)),
                Span::styled(&app.input.content, Style::new().fg(C_DIM)),
            ]),
            hint,
        ]
    } else if app.input.content.is_empty() {
        vec![
            Line::from(Span::styled(
                format!("{}{}", prefix, strings::INPUT_PLACEHOLDER),
                Style::new().fg(C_DIM),
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
                    Style::new().fg(Color::Rgb(250, 250, 250)),
                ))
            })
            .collect();
        result.push(hint);
        result
    };

    let sep_color = if matches!(app.mode, AppMode::Waiting) {
        C_SEP
    } else {
        Color::Rgb(42, 42, 50)
    };
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(sep_color))
        .padding(ratatui::widgets::Padding::horizontal(1))
        .style(Style::new().bg(C_BG_INPUT));

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

    let mut items: Vec<Line> = Vec::new();
    for (i, cmd) in commands.iter().enumerate() {
        let selected = i == app.slash_selected.min(commands.len().saturating_sub(1));
        let marker = if selected { " ▌" } else { "  " };
        let name_style = if selected {
            Style::new().fg(Color::Cyan).bold()
        } else {
            Style::new().fg(Color::White)
        };
        let args = if cmd.args.is_empty() {
            String::new()
        } else {
            format!(" {}", cmd.args)
        };

        items.push(Line::from(vec![
            Span::styled(
                marker,
                if selected {
                    Style::new().fg(Color::Cyan)
                } else {
                    Style::new().fg(C_DIM)
                },
            ),
            Span::styled(format!("/{}{}", cmd.name, args), name_style),
            Span::raw("  "),
            Span::styled(cmd.desc, Style::new().fg(C_DIM)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(C_RAIL))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0))
        .style(Style::new().bg(C_BG_INPUT));

    let paragraph = Paragraph::new(Text::from(items)).block(block);
    frame.render_widget(paragraph, area);
}
