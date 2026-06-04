use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::sync::Arc;

use super::ansi::{has_ansi, ansi_to_lines};
use super::markdown::{is_markdown, render_markdown};
use crate::app::{App, Message};
use crate::ui::utils;

pub(super) fn timestamp_label(app: &App, msg_index: usize, now: chrono::NaiveDateTime) -> String {
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

pub(super) fn indent_line(text: &str, style: Style) -> Line<'static> {
    Line::from(vec![Span::raw("   "), Span::styled(text.to_string(), style)])
}

pub(super) fn padded_line(text: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), style))
}

pub(super) fn get_or_render_md(
    msg_index: usize,
    text: &str,
    width: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    theme: &crate::theme::Theme,
) -> Arc<Vec<Line<'static>>> {
    format_cache
        .get(&msg_index)
        .cloned()
        .unwrap_or_else(|| Arc::new(render_markdown(text, width, theme)))
}

pub(super) fn build_message_lines(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    tool_call_headers: &std::collections::HashMap<usize, (String, Option<String>)>,
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
            tool_call_headers,
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

pub(super) fn build_user_lines(
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

pub(super) fn build_assistant_lines(
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
        let md_lines = get_or_render_md(idx, text, width.saturating_sub(3), format_cache, &app.config.theme);
        if !is_markdown(text) || md_lines.is_empty() {
            for w in utils::wrap_text(text, width) {
                lines.push(indent_line(&w, Style::default().fg(txt)));
            }
        } else {
            lines.extend(md_lines.iter().cloned());
        }
    }
    lines.push(Line::from(Span::raw("")));
    lines
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_tool_call_lines(
    app: &App,
    name: &str,
    args: &str,
    result: &str,
    step: usize,
    total: usize,
    width: usize,
    idx: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    tool_call_headers: &std::collections::HashMap<usize, (String, Option<String>)>,
    now: chrono::NaiveDateTime,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let is_expanded = app.overlay.tool_call_expanded.contains(&idx);
    let step_prefix = if total > 1 {
        format!("[{}/{}] ", step + 1, total)
    } else {
        String::new()
    };

    let (header, detail) = tool_call_headers
        .get(&idx)
        .cloned()
        .unwrap_or_else(|| {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(args) {
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
            }
        });

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
            lines.extend(cached.iter().cloned());
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

pub(super) fn build_error_lines(
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

pub(super) fn build_evaluation_lines(
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

pub(super) fn build_quality_lines(
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

pub(super) fn build_feedback_lines(app: &App, positive: bool, message: &Option<String>) -> Vec<Line<'static>> {
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

pub(super) fn build_image_lines(
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
