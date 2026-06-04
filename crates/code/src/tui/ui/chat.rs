use crate::tui::highlight::highlight_code_block;
use crate::app::AgentMessage;
use crate::tui::colors::*;
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};

pub fn tool_glyph(name: &str) -> &'static str {
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

pub fn is_diff_output(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let has_unified = lines
        .iter()
        .any(|l| l.starts_with("--- ") || l.starts_with("+++ ") || l.starts_with("@@ "));
    let has_plus = lines.iter().any(|l| l.starts_with('+') && !l.starts_with("+++"));
    let has_minus = lines.iter().any(|l| l.starts_with('-') && !l.starts_with("---"));
    has_unified || (has_plus && has_minus)
}

pub fn render_diff_line(line: &str) -> Vec<Span<'static>> {
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

pub fn render_ai_content(content: &str) -> Vec<Line<'static>> {
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
            result.push(Line::from(Span::styled(
                format!("    {}", heading),
                Style::new().fg(c_accent()).bold(),
            )));
        } else if let Some(heading) = line.strip_prefix("## ") {
            result.push(Line::from(Span::styled(
                format!("    {}", heading),
                Style::new().fg(c_cyan()).bold(),
            )));
        } else if let Some(item) = line.strip_prefix("- ") {
            result.push(Line::from(vec![
                Span::styled("    ", Style::new().fg(c_text())),
                Span::styled("• ", Style::new().fg(c_accent())),
                Span::styled(item.to_string(), Style::new().fg(c_text())),
            ]));
        } else if let Some(item) = line.strip_prefix("  - ") {
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

pub fn msg_bg(msg: &AgentMessage) -> Color {
    match msg {
        AgentMessage::User { .. } => c_bg_user(),
        AgentMessage::Assistant { .. } => c_bg_ai(),
        AgentMessage::ToolResult { .. } => c_bg_tool(),
        AgentMessage::System { .. } => c_bg_system(),
        AgentMessage::FileEdit { .. } => c_bg_file(),
        AgentMessage::Separator { .. } => c_bg(),
    }
}

pub fn rounded_top_line(width: usize) -> Line<'static> {
    if width < 2 {
        return Line::from("");
    }
    Line::from(Span::styled(
        format!("╭{}╮", "─".repeat(width - 2)),
        Style::new().fg(c_border()),
    ))
}

pub fn rounded_bottom_line(width: usize) -> Line<'static> {
    if width < 2 {
        return Line::from("");
    }
    Line::from(Span::styled(
        format!("╰{}╯", "─".repeat(width - 2)),
        Style::new().fg(c_border()),
    ))
}

pub fn build_msg_lines(
    msg: &AgentMessage,
    is_selected: bool,
    chat_width: usize,
) -> Vec<Line<'static>> {
    let top = rounded_top_line(chat_width);
    let bottom = rounded_bottom_line(chat_width);
    match msg {
        AgentMessage::User { content } => {
            let mut lines = vec![top];
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
            lines.push(bottom);
            lines
        }
        AgentMessage::Assistant {
            content,
            reasoning,
            tool_calls: _,
            reasoning_expanded,
            ..
        } => {
            let mut lines = vec![top];
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
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::new().fg(c_muted())),
                        Span::styled("▎ ", Style::new().fg(c_yellow())),
                        Span::styled("Thought", Style::new().fg(c_yellow())),
                        Span::styled("  (press to expand)", Style::new().fg(c_muted())),
                    ]));
                }
            }
            lines.push(bottom);
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

            let mut lines = vec![top];

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
            let mut lines = vec![top];
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
            lines.push(bottom);
            lines
        }
        AgentMessage::System { content } => {
            let mut lines = vec![top];
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
            lines.push(bottom);
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

pub fn build_all_msg_blocks(app: &crate::app::App, chat_width: usize) -> Vec<Vec<Line<'static>>> {
    app.messages
        .iter()
        .enumerate()
        .map(|(idx, msg)| {
            build_msg_lines(msg, app.selected_message == Some(idx), chat_width)
        })
        .collect()
}
