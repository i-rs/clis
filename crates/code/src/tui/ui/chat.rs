use crate::tui::highlight::highlight_code_block;
use crate::tui::colors::*;
use ratatui::text::{Line, Span};
use ratatui::style::Style;

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

