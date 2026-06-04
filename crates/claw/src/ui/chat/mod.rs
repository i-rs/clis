mod ansi;
mod builders;
mod markdown;

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{List, ListItem},
};
use std::sync::Arc;

use ansi::{has_ansi, wrapped_line_count};
use builders::build_message_lines;
use markdown::{is_markdown, render_markdown};

use crate::app::{App, Message};
use crate::ui::utils;

pub(super) fn render_chat(f: &mut Frame, area: Rect, app: &mut App) {
    let text_width = (area.width as usize).saturating_sub(4).max(20);
    let area_lines = (area.height as usize).saturating_sub(1).max(1);
    let total_msgs = app.messages.len();
    let now = chrono::Local::now().naive_local();

    let mut format_cache = std::mem::take(&mut app.render_state.format_cache);
    let mut heights = std::mem::take(&mut app.render_state.heights);
    let mut tool_call_headers = std::mem::take(&mut app.render_state.tool_call_headers);

    if format_cache.len() > total_msgs + 20 {
        format_cache.retain(|k, _| *k < total_msgs);
        tool_call_headers.retain(|k, _| *k < total_msgs);
    }

    let needs_full_rebuild = heights.len() != total_msgs;
    let width_changed = app.render_state.cached_width != text_width;
    if needs_full_rebuild {
        heights.clear();
        format_cache.clear();
        tool_call_headers.clear();
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
    } else if width_changed {
        heights.clear();
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
    } else if heights.len() > 0 && heights[0] == 0 {
        heights[0] = message_line_count(
            app,
            app.messages.last().unwrap(),
            text_width,
            total_msgs - 1,
            &mut format_cache,
        );
    }

    let total_content_height: usize = heights.iter().sum();
    let max_scroll = total_content_height.saturating_sub(area_lines);
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
            &tool_call_headers,
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
        let scroll_bar: String = std::iter::repeat_n('█', filled)
            .chain(std::iter::repeat_n('░', empty))
            .collect();

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
    app.render_state.tool_call_headers = tool_call_headers;
    app.render_state.cached_width = text_width;
    app.render_state.chat_height = area.height;
    app.max_scroll = max_scroll;
    app.scroll_lines = scroll_lines;
}

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
                    Arc::new(render_markdown(text, text_width.saturating_sub(3), &app.config.theme))
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
            let parsed_args = serde_json::from_str::<serde_json::Value>(args).ok();
            let has_explanation = parsed_args
                .as_ref()
                .is_some_and(|val| name == "i_rs"
                    && val.get("explanation").and_then(|v| v.as_str()).is_some());

            if !app.overlay.tool_call_expanded.contains(&msg_index) {
                return 1 + if has_explanation { 1 } else { 0 };
            }
            let mut lines = 1 + if has_explanation { 1 } else { 0 };
            if !result.is_empty() {
                if let Some(cached) = format_cache.get(&msg_index) {
                    if !cached.is_empty() {
                        lines += cached.len();
                    }
                } else {
                    let cached_result = utils::format_json_result(result, text_width);
                    if !cached_result.0.is_empty() {
                        let cached = format_cache
                            .entry(msg_index)
                            .or_insert_with(|| Arc::new(cached_result.0));
                        lines += cached.len();
                    } else {
                        lines += if has_ansi(result) {
                            ansi::ansi_line_count(result, text_width.saturating_sub(3))
                        } else {
                            wrapped_line_count(result, text_width.saturating_sub(3))
                        };
                    }
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

#[allow(clippy::too_many_arguments)]
fn build_message_item_with_skip(
    app: &App,
    msg: &Message,
    text_width: usize,
    msg_index: usize,
    format_cache: &std::collections::HashMap<usize, Arc<Vec<Line<'static>>>>,
    tool_call_headers: &std::collections::HashMap<usize, (String, Option<String>)>,
    skip_lines: usize,
    now: chrono::NaiveDateTime,
) -> ListItem<'static> {
    let is_selected = app.overlay.selection_mode && app.overlay.selected_message == Some(msg_index);

    let lines = build_message_lines(app, msg, text_width, msg_index, format_cache, tool_call_headers, now);
    let lines: Vec<Line> = if skip_lines > 0 && skip_lines < lines.len() {
        lines.into_iter().skip(skip_lines).collect()
    } else if skip_lines >= lines.len() {
        Vec::new()
    } else {
        lines
    };

    let mut item = ListItem::new(lines);
    if is_selected {
        let bg = match msg {
            Message::User { .. } => ratatui::style::Color::Rgb(35, 55, 35),
            Message::Assistant { .. } => ratatui::style::Color::Rgb(35, 45, 70),
            Message::ToolCall { .. } => ratatui::style::Color::Rgb(45, 40, 65),
            Message::Error { .. } => ratatui::style::Color::Rgb(70, 30, 30),
            Message::Evaluation { valid, .. } => {
                if *valid {
                    ratatui::style::Color::Rgb(30, 60, 40)
                } else {
                    ratatui::style::Color::Rgb(75, 40, 25)
                }
            }
            _ => ratatui::style::Color::Rgb(45, 45, 30),
        };
        item = item.style(Style::default().bg(bg));
    }
    item
}
