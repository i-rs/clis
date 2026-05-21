use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, Message};
use super::utils;

pub(super) fn render_chat(f: &mut Frame, area: Rect, app: &App) {
    let text_width = (area.width as usize).saturating_sub(4).max(20);
    let area_lines = (area.height as usize).saturating_sub(1).max(1);

    let mut format_cache: std::collections::HashMap<usize, Vec<Line<'static>>> =
        std::collections::HashMap::new();

    // heights[0] = newest message height, heights[n-1] = oldest
    let mut heights: Vec<usize> = Vec::with_capacity(app.messages.len());
    for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
        let msg_index = app.messages.len() - 1 - rev_idx;
        heights.push(message_line_count(
            app,
            msg,
            text_width,
            msg_index,
            &mut format_cache,
        ));
    }

    // Determine how many whole messages to skip + partial line offset
    let mut skipped_lines = 0usize;
    let mut msg_skip_count = 0usize;
    for (i, &h) in heights.iter().enumerate() {
        if skipped_lines + h <= app.scroll_lines {
            skipped_lines += h;
            msg_skip_count = i + 1;
        } else {
            break;
        }
    }
    let partial_skip = app.scroll_lines - skipped_lines;

    // Fill viewport, always include at least one message even if partial
    let mut end_idx = msg_skip_count;
    let mut accumulated = 0usize;
    for &h in heights[msg_skip_count..].iter() {
        accumulated += h;
        end_idx += 1;
        if end_idx > msg_skip_count + 1 && accumulated > area_lines + partial_skip {
            end_idx -= 1;
            break;
        }
    }
    if end_idx == msg_skip_count && end_idx < heights.len() {
        end_idx = msg_skip_count + 1;
    }

    // Build items with partial skip for the first visible message
    let mut items: Vec<ListItem> = Vec::new();
    for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
        if rev_idx < msg_skip_count {
            continue;
        }
        if rev_idx >= end_idx {
            continue;
        }
        let msg_index = app.messages.len() - 1 - rev_idx;
        let skip = if rev_idx == msg_skip_count { partial_skip } else { 0 };
        items.push(build_message_item_with_skip(
            app, msg, text_width, msg_index, &format_cache, skip,
        ));
    }
    items.reverse();

    let at_bottom = app.scroll_lines == 0;
    let hidden_extra = heights.len().saturating_sub(end_idx);

    let mut block = ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(if at_bottom && hidden_extra == 0 {
            Color::DarkGray
        } else {
            Color::Rgb(100, 120, 200)
        }));

    let total_hidden = msg_skip_count + hidden_extra;
    if total_hidden > 0 && !items.is_empty() {
        block = block.title(format!(" ▲ {} 条历史消息 ", total_hidden));
        block = block.title_alignment(ratatui::layout::Alignment::Center);
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// Check if a string contains ANSI escape codes.
fn has_ansi(text: &str) -> bool {
    text.contains("\x1b[")
}

/// Parse ANSI-colored text into ratatui Lines with proper styling.
/// Strips ANSI codes and wraps text to fit max_width.
fn ansi_to_lines(text: &str, max_width: usize) -> Vec<Line<'static>> {
    let plain = strip_ansi(text);
    let wrapped = utils::wrap_text(&plain, max_width.saturating_sub(3));

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

/// Estimate how many lines a block of text wraps to.
/// Strips ANSI codes for accurate width calculation.
fn wrapped_line_count(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count();
    }
    let clean = strip_ansi(text);
    clean.lines()
        .map(|line| {
            let w = UnicodeWidthStr::width(line);
            if w == 0 {
                1
            } else {
                w.div_ceil(max_width)
            }
        })
        .sum()
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
            // Use SAME rendering logic as build_message_item for accurate count
            // Pre-render and cache so Pass 3 can reuse
            let header_lines = 1;
            let trailing = 1;
            let body_lines = {
                let md_lines = format_cache
                    .entry(msg_index)
                    .or_insert_with(|| render_markdown(text, text_width.saturating_sub(3)));
                if !is_markdown(text) || md_lines.is_empty() {
                    wrapped_line_count(text, text_width)
                } else {
                    md_lines.len()
                }
            };
            header_lines + body_lines + trailing
        }
        Message::ToolCall {
            name,
            args,
            result,
            ..
        } => {
            // Collapsed: only header + optional explanation
            if !app.overlay.tool_call_expanded.contains(&msg_index) {
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
                let cached = format_cache.entry(msg_index).or_insert_with(|| utils::format_json_result(result, text_width).0);
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
            self.width += UnicodeWidthStr::width(text);
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
                for w in utils::wrap_text(&plain, max_width) {
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
fn build_message_item_with_skip(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Vec<Line<'static>>>,
    skip_lines: usize,
) -> ListItem<'static> {
    let is_selected = app.overlay.selection_mode && app.overlay.selected_message == Some(msg_index);

    let mut lines = build_message_lines(app, msg, text_width, msg_index, format_cache);
    if skip_lines > 0 && skip_lines < lines.len() {
        lines.drain(..skip_lines);
    } else if skip_lines >= lines.len() {
        lines.clear();
    }

    let mut item = ListItem::new(lines);
    if is_selected {
        let bg = match msg {
            Message::User { .. } => Color::Rgb(25, 35, 25),
            Message::Assistant { .. } => Color::Rgb(25, 30, 45),
            Message::ToolCall { .. } => Color::Rgb(25, 25, 35),
            Message::Error { .. } => Color::Rgb(35, 15, 15),
        };
        item = item.style(Style::default().bg(bg));
    }
    item
}

fn build_message_lines(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Vec<Line<'static>>>,
) -> Vec<Line<'static>> {
    match msg {
        Message::User { text } => {
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", utils::relative_time_naive(*ts)))
                .unwrap_or_default();
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("  You:{}", ts_label),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
            ];
            for wrapped in utils::wrap_text(text, text_width) {
                lines.push(Line::from(Span::styled(
                    format!("   {}", wrapped),
                    Style::default().fg(Color::White),
                )));
            }
            lines.push(Line::from(Span::raw("")));
            lines
        }
        Message::Assistant { text } => {
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", utils::relative_time_naive(*ts)))
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
                // Use cached rendering from Pass 1 if available
                let md_lines = format_cache
                    .get(&msg_index)
                    .cloned()
                    .unwrap_or_else(|| render_markdown(text, text_width.saturating_sub(3)));
                if !is_markdown(text) || md_lines.is_empty() {
                    // Fallback to simple wrapping for plain text
                    for wrapped in utils::wrap_text(text, text_width) {
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
            lines
        }
        Message::ToolCall {
            name,
            args,
            result,
            step,
            total_steps,
        } => {
            let mut lines = Vec::new();
            let is_expanded = app.overlay.tool_call_expanded.contains(&msg_index);

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
                .map(|ts| format!("  [{}]", utils::relative_time_naive(*ts)))
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
                        for wrapped in utils::wrap_text(result, text_width.saturating_sub(3)) {
                            lines.push(Line::from(Span::styled(
                                format!("   {}", wrapped),
                                Style::default().fg(Color::White),
                            )));
                        }
                    }
                }

            lines
        }
        Message::Error { text } => {
            let ts_label = app.message_timestamps
                .get(msg_index)
                .map(|ts| format!("  [{}]", utils::relative_time_naive(*ts)))
                .unwrap_or_default();
            let mut lines = vec![
                Line::from(Span::styled(
                    format!(" ✗ Error:{}", ts_label),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )),
            ];
            for wrapped in utils::wrap_text(text, text_width) {
                lines.push(Line::from(Span::styled(
                    format!("   {}", wrapped),
                    Style::default().fg(Color::Red),
                )));
            }
            lines.push(Line::from(Span::raw("")));
            lines
        }
    }
}
