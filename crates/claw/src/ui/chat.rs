use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
};
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

use super::utils;
use crate::app::{App, Message};

// ── Shared helpers ──

fn timestamp_label(app: &App, msg_index: usize, now: chrono::NaiveDateTime) -> String {
    let now_ts = now.and_utc().timestamp();
    app.message_timestamps
        .get(msg_index)
        .map(|ts| {
            format!(
                "  [{}]",
                utils::relative_time_at(ts.and_utc().timestamp(), now_ts)
            )
        })
        .unwrap_or_default()
}

fn indent_line(text: &str, style: Style) -> Line<'static> {
    let mut spans = vec![Span::raw("   ")];
    spans.push(Span::styled(text.to_string(), style));
    Line::from(spans)
}

fn padded_line(text: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), style))
}

fn get_or_render_md(
    msg_index: usize,
    text: &str,
    width: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
) -> Arc<Vec<Line<'static>>> {
    format_cache
        .get(&msg_index)
        .cloned()
        .unwrap_or_else(|| Arc::new(render_markdown(text, width)))
}

// ── render_chat ──

pub(super) fn render_chat(f: &mut Frame, area: Rect, app: &mut App) {
    let text_width = (area.width as usize).saturating_sub(4).max(20);
    let area_lines = (area.height as usize).saturating_sub(1).max(1);
    let total_msgs = app.messages.len();
    let now = chrono::Local::now().naive_local();

    let mut format_cache = std::mem::take(&mut app.render_state.format_cache);
    let mut heights = std::mem::take(&mut app.render_state.heights);

    if format_cache.len() > total_msgs + 20 {
        format_cache.retain(|k, _| *k < total_msgs);
    }

    if heights.len() != total_msgs || app.render_state.cached_width != text_width {
        heights.clear();
        format_cache.clear();
        heights.reserve(total_msgs);
        for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
            let msg_index = total_msgs - 1 - rev_idx;
            heights.push(message_line_count(
                app,
                msg,
                text_width,
                msg_index,
                &mut format_cache,
            ));
        }
    }

    let total_content_height: usize = heights.iter().sum();
    let max_scroll = total_content_height.saturating_sub(1);
    let scroll_lines = app.scroll_lines.min(max_scroll);

    let mut skipped_lines = 0usize;
    let mut msg_skip_count = 0usize;
    for (i, &h) in heights.iter().enumerate() {
        if skipped_lines + h <= scroll_lines {
            skipped_lines += h;
            msg_skip_count = i + 1;
        } else {
            break;
        }
    }
    let partial_skip = scroll_lines - skipped_lines;

    let mut end_idx = msg_skip_count;
    let mut accumulated = 0usize;
    for &h in heights[msg_skip_count..].iter() {
        if accumulated + h > area_lines + partial_skip && end_idx > msg_skip_count {
            break;
        }
        accumulated += h;
        end_idx += 1;
    }
    if end_idx == msg_skip_count && end_idx < heights.len() {
        end_idx = msg_skip_count + 1;
    }
    end_idx = end_idx.min(heights.len());

    let mut items: Vec<ListItem> = Vec::with_capacity(end_idx.saturating_sub(msg_skip_count));
    for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
        if rev_idx < msg_skip_count || rev_idx >= end_idx {
            continue;
        }
        let msg_index = total_msgs - 1 - rev_idx;
        let skip = if rev_idx == msg_skip_count {
            partial_skip
        } else {
            0
        };
        items.push(build_message_item_with_skip(
            app,
            msg,
            text_width,
            msg_index,
            &format_cache,
            skip,
            now,
        ));
    }
    items.reverse();

    let at_bottom = app.scroll_lines == 0;
    let hidden_extra = heights.len().saturating_sub(end_idx);

    let mut block = ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(if at_bottom && hidden_extra == 0 {
            app.config.theme.dim_text()
        } else {
            app.config.theme.primary()
        }));

    let total_hidden = msg_skip_count + hidden_extra;
    if total_msgs > 0 {
        let visible_end = total_msgs.saturating_sub(msg_skip_count);
        let pct = if total_msgs <= 1 {
            100
        } else {
            (visible_end * 100) / total_msgs
        };
        let bar_width = 10;
        let filled = ((pct * bar_width) / 100).max(1).min(bar_width);
        let empty = bar_width - filled;
        let scroll_bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        if total_hidden > 0 && !items.is_empty() {
            block = block.title(format!(
                " ▲ {} 条历史消息  {scroll_bar} {pct}% ",
                total_hidden
            ));
            block = block.title_alignment(ratatui::layout::Alignment::Center);
        } else {
            block = block.title(format!(" {scroll_bar} {pct}% "));
            block = block.title_alignment(ratatui::layout::Alignment::Right);
        }
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);

    app.render_state.heights = heights;
    app.render_state.format_cache = format_cache;
    app.render_state.cached_width = text_width;
    app.max_scroll = max_scroll;
}

// ── Message line count ──

fn message_line_count(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &mut std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
) -> usize {
    match msg {
        Message::User { text } => 1 + wrapped_line_count(text, text_width) + 1,
        Message::Assistant { text, reasoning } if text.is_empty() && reasoning.is_empty() => 3,
        Message::Assistant { text, reasoning } => {
            let mut extra = 0usize;
            if !reasoning.is_empty() {
                extra += 1;
                if app.overlay.reasoning_expanded.contains(&msg_index) {
                    extra += reasoning.lines().count();
                }
            }
            let body_lines = {
                let md = format_cache.entry(msg_index).or_insert_with(|| {
                    Arc::new(render_markdown(text, text_width.saturating_sub(3)))
                });
                if !is_markdown(text) || md.is_empty() {
                    wrapped_line_count(text, text_width)
                } else {
                    md.len()
                }
            };
            1 + body_lines + 1 + extra
        }
        Message::ToolCall {
            name, args, result, ..
        } => {
            if !app.overlay.tool_call_expanded.contains(&msg_index) {
                let mut lines = 1;
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(args)
                    && name == "i_rs"
                    && val.get("explanation").and_then(|v| v.as_str()).is_some()
                {
                    lines += 1;
                }
                return lines;
            }
            let mut lines = 1;
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(args)
                && name == "i_rs"
                && val.get("explanation").and_then(|v| v.as_str()).is_some()
            {
                lines += 1;
            }
            if !result.is_empty() {
                let cached_result = utils::format_json_result(result, text_width);
                if !cached_result.0.is_empty() {
                    let cached = format_cache
                        .entry(msg_index)
                        .or_insert_with(|| Arc::new(cached_result.0));
                    lines += cached.len();
                } else {
                    lines += if has_ansi(result) {
                        ansi_line_count(result, text_width.saturating_sub(3))
                    } else {
                        wrapped_line_count(result, text_width.saturating_sub(3))
                    };
                }
            }
            lines
        }
        Message::Error { text } => 1 + wrapped_line_count(text, text_width) + 1,
        Message::Evaluation { valid, issues, .. } => {
            if *valid {
                0
            } else {
                1 + issues.len()
            }
        }
        _ => 0,
    }
}

// ── build_message_item_with_skip ──

fn build_message_item_with_skip(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    skip_lines: usize,
    now: chrono::NaiveDateTime,
) -> ListItem<'static> {
    let is_selected = app.overlay.selection_mode && app.overlay.selected_message == Some(msg_index);

    let mut lines = build_message_lines(app, msg, text_width, msg_index, format_cache, now);
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
            Message::Evaluation { valid, .. } => {
                if *valid {
                    Color::Rgb(20, 35, 25)
                } else {
                    Color::Rgb(40, 20, 15)
                }
            }
            _ => Color::Rgb(20, 20, 20),
        };
        item = item.style(Style::default().bg(bg));
    }
    item
}

// ── build_message_lines (dispatcher) ──

fn build_message_lines(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    match msg {
        Message::User { text } => build_user_lines(app, text, text_width, msg_index, now),
        Message::Assistant { text, reasoning } => build_assistant_lines(
            app,
            text,
            reasoning,
            text_width,
            msg_index,
            format_cache,
            now,
        ),
        Message::ToolCall {
            name,
            args,
            result,
            step,
            total_steps,
        } => build_tool_call_lines(
            app,
            name,
            args,
            result,
            *step,
            *total_steps,
            text_width,
            msg_index,
            format_cache,
            now,
        ),
        Message::Error { text } => build_error_lines(app, text, text_width, msg_index, now),
        Message::Evaluation {
            tool,
            valid,
            issues,
        } => build_evaluation_lines(app, tool, *valid, issues),
        Message::Quality {
            score,
            complete,
            issues,
            ..
        } => build_quality_lines(app, *score, *complete, issues),
        Message::Feedback { positive, message } => build_feedback_lines(app, *positive, message),
        Message::Image {
            path,
            alt_text,
            width: _,
            height: _,
            format: _,
        } => build_image_lines(app, path, alt_text, text_width, msg_index, now),
    }
}

// ── Per-message builders ──

fn build_user_lines(
    app: &App,
    text: &str,
    width: usize,
    idx: usize,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    let ts = timestamp_label(app, idx, now);
    let sec = app.config.theme.secondary();
    let mut lines = vec![Line::from(vec![
        Span::styled("▌ ", Style::default().fg(sec)),
        Span::styled("You", Style::default().fg(sec).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(":{}", ts),
            Style::default().fg(app.config.theme.dim_text()),
        ),
    ])];
    for w in utils::wrap_text(text, width) {
        lines.push(indent_line(
            &w,
            Style::default().fg(app.config.theme.text()),
        ));
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

fn build_assistant_lines(
    app: &App,
    text: &str,
    reasoning: &str,
    width: usize,
    idx: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    let ts = timestamp_label(app, idx, now);
    let pri = app.config.theme.primary();
    let dim = app.config.theme.dim_text();
    let txt = app.config.theme.text();

    let mut lines = vec![Line::from(vec![
        Span::styled("◆ ", Style::default().fg(pri)),
        Span::styled(
            "Claw",
            Style::default().fg(pri).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(":{}", ts), Style::default().fg(dim)),
    ])];

    if !reasoning.is_empty() {
        let is_expanded = app.overlay.reasoning_expanded.contains(&idx);
        let toggle = if is_expanded { " [-]" } else { " [+]" };
        lines.push(padded_line(
            &format!("   💭 思考过程{}", toggle),
            Style::default().fg(Color::Rgb(120, 120, 140)),
        ));
        if is_expanded {
            for rl in reasoning.lines() {
                let d = utils::truncate_str(rl, width.saturating_sub(6).max(20));
                lines.push(indent_line(
                    &format!("   {}", d),
                    Style::default().fg(Color::Rgb(100, 100, 130)),
                ));
            }
        }
    }

    if text.is_empty() && reasoning.is_empty() {
        lines.push(padded_line("   ...", Style::default().fg(dim)));
    } else if !text.is_empty() {
        let md_lines = get_or_render_md(idx, text, width.saturating_sub(3), format_cache);
        if !is_markdown(text) || md_lines.is_empty() {
            for w in utils::wrap_text(text, width) {
                lines.push(indent_line(&w, Style::default().fg(txt)));
            }
        } else {
            for md_line in md_lines.iter() {
                lines.push(md_line.clone());
            }
        }
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

fn build_tool_call_lines(
    app: &App,
    name: &str,
    args: &str,
    result: &str,
    step: usize,
    total: usize,
    width: usize,
    idx: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let is_expanded = app.overlay.tool_call_expanded.contains(&idx);
    let step_prefix = if total > 1 {
        format!("[{}/{}] ", step + 1, total)
    } else {
        String::new()
    };

    let (header, detail) = if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
        if name == "i_rs" {
            let tool = val.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
            let cmd = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
            let exp = val.get("explanation").and_then(|v| v.as_str());
            (
                format!("▸▸ {}{} {}", step_prefix, tool, cmd),
                exp.map(|s| s.to_string()),
            )
        } else if name == "search_conversations" {
            let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
            (format!("◉ 搜索历史: {}", q), None)
        } else if name == "search_tools" {
            let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
            (format!("◉ search: {}", q), None)
        } else if name == "update_user_memory" {
            ("◎ 记住用户信息".to_string(), None)
        } else if name == "file_ops" {
            let op = val.get("operation").and_then(|v| v.as_str()).unwrap_or("?");
            let p = val.get("path").and_then(|v| v.as_str()).unwrap_or("?");
            (format!("▤ {}: {}", op, p), None)
        } else if name == "web_search" {
            let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
            (format!("◉ 搜索网络: {}", q), None)
        } else {
            (format!("▸▸ {}{}", step_prefix, name), None)
        }
    } else {
        (format!("▸▸ {} {}", step_prefix, name), None)
    };

    let indicator = if is_expanded { " [-]" } else { " [+]" };
    let ts = timestamp_label(app, idx, now);
    lines.push(Line::from(Span::styled(
        format!("{}{}{}", header, indicator, ts),
        Style::default()
            .fg(app.config.theme.accent())
            .add_modifier(Modifier::BOLD),
    )));

    if let Some(exp) = &detail {
        lines.push(padded_line(
            &format!("   └─ {}", exp),
            Style::default().fg(app.config.theme.dim_text()),
        ));
    }

    if is_expanded
        && !result.is_empty()
        && let Some(cached) = format_cache.get(&idx)
    {
        if !cached.is_empty() {
            lines.extend((**cached).clone());
        } else if has_ansi(result) {
            lines.extend(ansi_to_lines(result, width));
        } else {
            for w in utils::wrap_text(result, width.saturating_sub(3)) {
                lines.push(indent_line(
                    &w,
                    Style::default().fg(app.config.theme.text()),
                ));
            }
        }
    }

    lines
}

fn build_error_lines(
    app: &App,
    text: &str,
    width: usize,
    idx: usize,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    let err = app.config.theme.error();
    let ts = timestamp_label(app, idx, now);
    let mut lines = vec![Line::from(vec![
        Span::styled("✗ ", Style::default().fg(err)),
        Span::styled(
            format!("Error:{}", ts),
            Style::default().fg(err).add_modifier(Modifier::BOLD),
        ),
    ])];
    for w in utils::wrap_text(text, width) {
        lines.push(indent_line(&w, Style::default().fg(err)));
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

fn build_evaluation_lines(
    app: &App,
    tool: &str,
    valid: bool,
    issues: &[String],
) -> Vec<Line<'static>> {
    if valid {
        return Vec::new();
    }
    let accent = app.config.theme.accent();
    let text = app.config.theme.text();
    let mut lines = vec![Line::from(vec![
        Span::styled("⚠ ", Style::default().fg(accent)),
        Span::styled(
            format!("工具结果检查: {}", tool),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
    ])];
    for issue in issues {
        lines.push(padded_line(
            &format!("   • {}", issue),
            Style::default().fg(text),
        ));
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

fn build_quality_lines(
    app: &App,
    score: Option<f64>,
    complete: bool,
    issues: &[String],
) -> Vec<Line<'static>> {
    let accent = app.config.theme.accent();
    let text = app.config.theme.text();
    let mut lines = vec![Line::from(vec![
        Span::styled("📊 ", Style::default().fg(accent)),
        Span::styled(
            "回答质量评估",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
    ])];
    if let Some(s) = score {
        lines.push(padded_line(
            &format!("   评分: {:.0}%", s * 100.0),
            Style::default().fg(text),
        ));
    }
    lines.push(padded_line(
        &format!("   完整性: {}", if complete { "✅" } else { "❌" }),
        Style::default().fg(text),
    ));
    for issue in issues {
        lines.push(padded_line(
            &format!("   • {}", issue),
            Style::default().fg(accent),
        ));
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

fn build_feedback_lines(app: &App, positive: bool, message: &Option<String>) -> Vec<Line<'static>> {
    let icon = if positive { "👍" } else { "👎" };
    let color = if positive {
        app.config.theme.accent()
    } else {
        app.config.theme.error()
    };
    let mut lines = vec![Line::from(Span::styled(
        format!("{} 用户反馈", icon),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))];
    if let Some(msg) = message {
        lines.push(padded_line(
            &format!("   {}", msg),
            Style::default().fg(app.config.theme.text()),
        ));
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

fn build_image_lines(
    app: &App,
    path: &str,
    alt_text: &str,
    _width: usize,
    idx: usize,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    let accent = app.config.theme.accent();
    let dim = app.config.theme.dim_text();
    let text = app.config.theme.text();
    let ts = timestamp_label(app, idx, now);

    let mut lines = vec![Line::from(vec![
        Span::styled("🖼 ", Style::default().fg(accent)),
        Span::styled(
            format!("Image:{}", ts),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
    ])];
    lines.push(padded_line(
        &format!("   描述: {}", alt_text),
        Style::default().fg(text),
    ));
    lines.push(padded_line(
        &format!("   路径: ~/.i-rs/claw/images/{} (按 Enter 打开)", path),
        Style::default().fg(dim),
    ));
    lines.push(Line::from(Span::raw("")));
    lines
}

// ── ANSI handling ──

fn has_ansi(text: &str) -> bool {
    text.contains("\x1b[")
}

fn ansi_to_lines(text: &str, max_width: usize) -> Vec<Line<'static>> {
    let plain = strip_ansi(text);
    let wrapped = utils::wrap_text(&plain, max_width.saturating_sub(3));
    let mut lines = Vec::new();
    let mut plain_offset = 0usize;
    for w in &wrapped {
        let line_end = plain_offset + w.len();
        let spans = parse_ansi_line_at(text, plain_offset, line_end);
        if spans.is_empty() {
            lines.push(indent_line(w, Style::default().fg(Color::White)));
        } else {
            let mut result = vec![Span::raw("   ")];
            result.extend(spans);
            lines.push(Line::from(result));
        }
        plain_offset = line_end;
    }
    lines
}

fn strip_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let b = bytes[i];
                i += 1;
                if b.is_ascii_alphabetic() || b == b'~' {
                    break;
                }
            }
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != 0x1b {
                i += 1;
            }
            result.push_str(&text[start..i]);
        }
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AnsiState {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    dim: bool,
}

impl AnsiState {
    fn reset(&mut self) {
        self.fg = None;
        self.bg = None;
        self.bold = false;
        self.italic = false;
        self.underline = false;
        self.dim = false;
    }
    fn to_style(self) -> Style {
        let mut s = Style::default();
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        if self.italic {
            s = s.add_modifier(Modifier::ITALIC);
        }
        if self.underline {
            s = s.add_modifier(Modifier::UNDERLINED);
        }
        if self.dim {
            s = s.add_modifier(Modifier::DIM);
        }
        if let Some(c) = self.fg {
            s = s.fg(c);
        }
        if let Some(c) = self.bg {
            s = s.bg(c);
        }
        s
    }
}

fn parse_ansi_line_at(raw: &str, line_start: usize, line_end: usize) -> Vec<Span<'static>> {
    let raw_bytes = raw.as_bytes();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut state = AnsiState {
        fg: None,
        bg: None,
        bold: false,
        italic: false,
        underline: false,
        dim: false,
    };
    let mut cur_text = String::new();
    let mut raw_i = 0;
    let mut plain_i = 0;

    while raw_i < raw_bytes.len() && plain_i < line_end {
        if raw_bytes[raw_i] == 0x1b && raw_i + 1 < raw_bytes.len() && raw_bytes[raw_i + 1] == b'[' {
            if !cur_text.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut cur_text),
                    state.to_style(),
                ));
            }
            raw_i += 2;
            let (next_raw, next_state) = parse_sgr(raw_bytes, raw_i, state);
            raw_i = next_raw;
            state = next_state;
        } else {
            let c = raw[raw_i..].chars().next().unwrap_or('\0');
            let c_len = c.len_utf8();
            raw_i += c_len;
            if plain_i >= line_start && plain_i < line_end {
                cur_text.push(c);
            }
            plain_i += c_len;
        }
    }
    if !cur_text.is_empty() {
        spans.push(Span::styled(cur_text, state.to_style()));
    }
    spans
}

fn parse_sgr(bytes: &[u8], start: usize, mut state: AnsiState) -> (usize, AnsiState) {
    let mut i = start;
    let mut params = Vec::new();
    let mut buf = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b';' || b == b':' {
            if !buf.is_empty() {
                if let Ok(n) = buf.parse::<i32>() {
                    params.push(n);
                }
                buf.clear();
            }
            i += 1;
        } else if b == b'm' {
            if !buf.is_empty() {
                if let Ok(n) = buf.parse::<i32>() {
                    params.push(n);
                }
            } else if params.is_empty() {
                params.push(0);
            }
            i += 1;
            break;
        } else if b.is_ascii_digit() {
            buf.push(b as char);
            i += 1;
        } else {
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'm' {
                j += 1;
            }
            i = if j < bytes.len() { j + 1 } else { j };
            break;
        }
    }
    apply_sgr_params(&params, &mut state);
    (i, state)
}

fn apply_sgr_params(params: &[i32], state: &mut AnsiState) {
    let mut pi = 0;
    while pi < params.len() {
        match params[pi] {
            0 => state.reset(),
            1 => {
                state.bold = true;
                state.dim = false;
            }
            2 => state.dim = true,
            3 => state.italic = true,
            4 => state.underline = true,
            22 => {
                state.bold = false;
                state.dim = false;
            }
            23 => state.italic = false,
            24 => state.underline = false,
            30 => state.fg = Some(Color::Black),
            31 => state.fg = Some(Color::Red),
            32 => state.fg = Some(Color::Green),
            33 => state.fg = Some(Color::Yellow),
            34 => state.fg = Some(Color::Blue),
            35 => state.fg = Some(Color::Magenta),
            36 => state.fg = Some(Color::Cyan),
            37 => state.fg = Some(Color::White),
            38 if pi + 2 < params.len() => match params[pi + 1] {
                2 if pi + 4 < params.len() => {
                    state.fg = Some(Color::Rgb(
                        params[pi + 2].clamp(0, 255) as u8,
                        params[pi + 3].clamp(0, 255) as u8,
                        params[pi + 4].clamp(0, 255) as u8,
                    ));
                    pi += 4;
                }
                5 if pi + 2 < params.len() => {
                    state.fg = Some(indexed_color(params[pi + 2]));
                    pi += 2;
                }
                _ => {}
            },
            39 => state.fg = None,
            40 => state.bg = Some(Color::Black),
            41 => state.bg = Some(Color::Red),
            42 => state.bg = Some(Color::Green),
            43 => state.bg = Some(Color::Yellow),
            44 => state.bg = Some(Color::Blue),
            45 => state.bg = Some(Color::Magenta),
            46 => state.bg = Some(Color::Cyan),
            47 => state.bg = Some(Color::White),
            48 if pi + 2 < params.len() => match params[pi + 1] {
                2 if pi + 4 < params.len() => {
                    state.bg = Some(Color::Rgb(
                        params[pi + 2].clamp(0, 255) as u8,
                        params[pi + 3].clamp(0, 255) as u8,
                        params[pi + 4].clamp(0, 255) as u8,
                    ));
                    pi += 4;
                }
                5 if pi + 2 < params.len() => {
                    state.bg = Some(indexed_color(params[pi + 2]));
                    pi += 2;
                }
                _ => {}
            },
            49 => state.bg = None,
            90 => state.fg = Some(Color::Rgb(128, 128, 128)),
            91 => state.fg = Some(Color::Rgb(255, 128, 128)),
            92 => state.fg = Some(Color::Rgb(128, 255, 128)),
            93 => state.fg = Some(Color::Rgb(255, 255, 128)),
            94 => state.fg = Some(Color::Rgb(128, 128, 255)),
            95 => state.fg = Some(Color::Rgb(255, 128, 255)),
            96 => state.fg = Some(Color::Rgb(128, 255, 255)),
            97 => state.fg = Some(Color::White),
            100 => state.bg = Some(Color::Rgb(128, 128, 128)),
            101 => state.bg = Some(Color::Rgb(255, 128, 128)),
            102 => state.bg = Some(Color::Rgb(128, 255, 128)),
            103 => state.bg = Some(Color::Rgb(255, 255, 128)),
            104 => state.bg = Some(Color::Rgb(128, 128, 255)),
            105 => state.bg = Some(Color::Rgb(255, 128, 255)),
            106 => state.bg = Some(Color::Rgb(128, 255, 255)),
            107 => state.bg = Some(Color::White),
            _ => {}
        }
        pi += 1;
    }
}

fn indexed_color(n: i32) -> Color {
    let n = n.clamp(0, 255) as u8;
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::Rgb(128, 128, 128),
        9 => Color::Rgb(255, 128, 128),
        10 => Color::Rgb(128, 255, 128),
        11 => Color::Rgb(255, 255, 128),
        12 => Color::Rgb(128, 128, 255),
        13 => Color::Rgb(255, 128, 255),
        14 => Color::Rgb(128, 255, 255),
        15 => Color::Rgb(255, 255, 255),
        n if n < 232 => {
            let n = n as u32 - 16;
            Color::Rgb(
                ((n / 36) * 51) as u8,
                ((n % 36 / 6) * 51) as u8,
                ((n % 6) * 51) as u8,
            )
        }
        n => {
            let g = ((n as u32 - 232) * 10 + 8) as u8;
            Color::Rgb(g, g, g)
        }
    }
}

// ── Markdown rendering ──

fn ansi_line_count(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count();
    }
    let clean = strip_ansi(text);
    clean
        .lines()
        .map(|line| {
            let w = UnicodeWidthStr::width(line);
            if w == 0 { 1 } else { w.div_ceil(max_width) }
        })
        .sum()
}

fn wrapped_line_count(text: &str, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines().count();
    }
    let clean = strip_ansi(text);
    utils::wrap_text(&clean, max_width).len()
}

fn is_markdown(text: &str) -> bool {
    text.contains("**")
        || text.contains("__")
        || text.contains("``")
        || (text.len() > 1 && text.as_bytes()[0] == b'#' && text.as_bytes()[1] == b' ')
        || (text.len() > 1 && text.as_bytes()[0] == b'-' && text.as_bytes()[1] == b' ')
        || text.contains("\n- ")
        || text.contains("---")
}

fn render_markdown(text: &str, max_width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    struct MdLine {
        spans: Vec<(String, Style)>,
        width: usize,
    }
    impl MdLine {
        fn new() -> Self {
            Self {
                spans: Vec::new(),
                width: 0,
            }
        }
        fn add(&mut self, text: &str, style: Style) {
            self.width += UnicodeWidthStr::width(text);
            if let Some(last) = self.spans.last_mut()
                && last.1 == style
            {
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
                let spans: Vec<Span> = self
                    .spans
                    .drain(..)
                    .map(|(t, s)| Span::styled(t, s))
                    .collect();
                out.push(Line::from(spans));
            } else {
                for (text, style) in &self.spans {
                    for w in utils::wrap_text(text, max_width) {
                        out.push(Line::from(Span::styled(w, *style)));
                    }
                }
                self.spans.clear();
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
                    let heading_color = match level as u8 {
                        1 => Color::Rgb(34, 211, 238),
                        2 => Color::Rgb(150, 200, 220),
                        _ => Color::Rgb(180, 180, 200),
                    };
                    let prefix = if level as u8 <= 3 && level as u8 > 0 {
                        format!("{} ", "#".repeat(level as usize))
                    } else {
                        String::new()
                    };
                    acc.add(
                        &prefix,
                        Style::default()
                            .fg(heading_color)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                Tag::List(_) => {}
                Tag::Item => {
                    acc.flush(&mut lines, max_width);
                    acc.add("▸ ", Style::default().fg(Color::Rgb(251, 191, 36)));
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
                TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item => {
                    acc.flush(&mut lines, max_width)
                }
                TagEnd::Emphasis => italic = false,
                TagEnd::Strong => bold = false,
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    if code_text.lines().any(|l| !l.trim().is_empty()) {
                        let code_bg = Color::Rgb(15, 15, 22);
                        lines.push(Line::from(Span::styled(
                            format!("{:─^width$}", " code ", width = max_width.min(40)),
                            Style::default().fg(Color::Rgb(100, 100, 120)).bg(code_bg),
                        )));
                        for cl in code_text.lines() {
                            lines.push(Line::from(Span::styled(
                                format!("  {}", cl),
                                Style::default().fg(Color::Rgb(220, 180, 120)).bg(code_bg),
                            )));
                        }
                        lines.push(Line::from(Span::styled(
                            "".to_string(),
                            Style::default().bg(code_bg),
                        )));
                    }
                }
                TagEnd::Link | TagEnd::List(_) => {}
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
            Event::Code(t) => acc.add(
                &t,
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Rgb(30, 30, 30)),
            ),
            Event::SoftBreak | Event::HardBreak => acc.flush(&mut lines, max_width),
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
