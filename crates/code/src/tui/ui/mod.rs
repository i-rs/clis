mod chat;
mod input_bar;

use crate::app::App;
use crate::tui::colors::*;
use chat::{build_all_msg_blocks, msg_bg, tool_glyph};
pub use input_bar::filtered_slash_commands;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph, Wrap},
};
use std::cell::{Cell, RefCell};

// Cache: (message_generation, chat_width, blocks)
thread_local! {
    static MSG_BLOCKS_CACHE: RefCell<(usize, u16, Vec<Vec<Line<'static>>>)> =
        const { RefCell::new((0, 0, Vec::new())) };
    /// Virtual Y ranges per message: (start_y, end_y) in virtual space (unscrolled).
    static MSG_VIRTUAL_YS: RefCell<Vec<(usize, usize)>> = const { RefCell::new(Vec::new()) };
    /// Virtual Y range of the streaming block, if present.
    static STREAMING_VIRTUAL_Y: RefCell<Option<(usize, usize)>> = const { RefCell::new(None) };
    static MAX_SCROLL: Cell<usize> = const { Cell::new(0) };
    /// Incremental syntect highlight cache: (content_len, rendered_lines)
    static STREAM_CONTENT_CACHE: RefCell<(usize, Vec<Line<'static>>)> =
        const { RefCell::new((0, Vec::new())) };
}

pub const SIDEBAR_WIDTH: u16 = 40;

/// The Y offset of the chat content area (below the 1-line title bar).
pub const CHAT_AREA_Y: u16 = 1;

use super::utils::short_path;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let input_lines = (app.input.content.lines().count() + 1).clamp(2, 8) as u16 + 2;

    let [title_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    render_title_bar(frame, title_area, app);

    let [chat_body, sidebar_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(SIDEBAR_WIDTH)]).areas(body);
    let [chat_area, input_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(input_lines)]).areas(chat_body);

    render_chat(frame, chat_area, app);
    input_bar::render_input_bar(frame, input_area, app);
    crate::tui::sidebar::render_sidebar(frame, sidebar_area, app);

    if app.show_slash_picker {
        let commands = filtered_slash_commands(app);
        if !commands.is_empty() {
            let max_picker = chat_area.height.saturating_sub(3);
            let desired = commands.len() as u16 + 2;
            let picker_height = desired.min(max_picker).max(3);
            let anchor_y = chat_area.y + chat_area.height.saturating_sub(picker_height);
            let picker_area = Rect {
                x: chat_area.x,
                y: anchor_y,
                width: chat_area.width,
                height: picker_height,
            };
            input_bar::render_slash_picker(frame, picker_area, app);
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


fn build_streaming_block(app: &App) -> Vec<Line<'static>> {
    let s = app.streaming.as_ref().expect("streaming must be Some inside build_streaming_block");
    let mut lines: Vec<Line<'static>> = vec![Line::from(vec![
        Span::styled("  ", Style::new().fg(c_muted())),
        Span::styled("▎", Style::new().fg(c_green())),
        Span::styled(" Assistant", Style::new().fg(c_text()).bold()),
    ])];

    if let Some(ref tool) = s.current_tool {
        let glyph = tool_glyph(&tool.name);
        lines.push(Line::from(vec![
            Span::styled("● ", Style::new().fg(c_accent())),
            Span::styled(
                format!("{} {} running...", glyph, tool.name),
                Style::new().fg(c_accent()).bold(),
            ),
        ]));
    }

    if !s.reasoning.is_empty() {
        if s.reasoning_collapsed {
            lines.push(Line::from(vec![
                Span::styled("▸ ", Style::new().fg(c_muted())),
                Span::styled(
                    format!("思考过程 ({} 行)", s.reasoning.lines().count()),
                    Style::new().fg(c_dim()),
                ),
            ]));
        } else {
            let reasoning_lines: Vec<&str> = s.reasoning.lines().collect();
            let total = reasoning_lines.len();
            let show_count = if s.content.is_empty() { 6.min(total) } else { 3.min(total) };
            let start = total.saturating_sub(show_count);
            lines.push(Line::from(Span::styled(
                "▼ 思考过程",
                Style::new().fg(c_yellow()),
            )));
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!("│ … {} earlier lines", start),
                    Style::new().fg(c_dim()).italic(),
                )));
            }
            for line in &reasoning_lines[start..] {
                lines.push(Line::from(Span::styled(
                    format!("│ {}", line),
                    Style::new().fg(c_dim()).italic(),
                )));
            }
            lines.push(Line::from(Span::styled("╰", Style::new().fg(c_border()))));
        }
    }

    if !s.content.is_empty() {
        STREAM_CONTENT_CACHE.with(|cache| {
            let (cached_len, cached_lines) = &mut *cache.borrow_mut();
            if *cached_len != s.content.len() {
                *cached_lines = chat::render_ai_content(&s.content);
                *cached_len = s.content.len();
            }
            lines.extend(cached_lines.iter().cloned());
        });
    }

    if !s.content.is_empty() {
        let glyph = if s.current_tool.is_some() { " ▸" } else { " ▊" };
        lines.push(Line::from(vec![Span::styled(glyph, Style::new().fg(c_green()))]));
    } else if s.current_tool.is_none()
        && s.reasoning.is_empty()
        && s.tool_calls.is_empty()
    {
        lines.push(Line::from(vec![Span::styled(" ⏳", Style::new().fg(c_dim()))]));
    }

    lines
}

fn render_chat(frame: &mut Frame, area: Rect, app: &App) {
    let show_hint = !app.auto_scroll && app.messages.len() > 1;
    let hint_height = if show_hint { 1 } else { 0 };
    let content_area = Rect {
        x: area.x,
        y: area.y + hint_height,
        width: area.width,
        height: area.height.saturating_sub(hint_height),
    };

    if show_hint {
        let hint = super::strings::scrolled_up_hint(app.messages.len().saturating_sub(1));
        let hint_line = Line::from(Span::styled(hint, Style::new().fg(c_dim())));
        let hint_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let hint_para =
            Paragraph::new(Text::from(vec![hint_line])).style(Style::new().bg(c_bg()));
        frame.render_widget(hint_para, hint_area);
    }

    let stream_lines = if app.streaming.is_some() {
        build_streaming_block(app)
    } else {
        Vec::new()
    };

    MSG_BLOCKS_CACHE.with(|cache| {
        let msg_count = app.messages.len();
        let (cached_gen, cached_w, cached) = &mut *cache.borrow_mut();
        let w = area.width;
        if *cached_gen != app.message_generation || *cached_w != w {
            *cached = build_all_msg_blocks(app, w as usize);
            *cached_gen = app.message_generation;
            *cached_w = w;
        }

        let gap: usize = 1;
        let streaming_gap = if stream_lines.is_empty() { 0 } else { 1 };
        let total_height: usize = cached.iter().map(|b| b.len()).sum::<usize>()
            + cached.len().saturating_sub(1) * gap
            + streaming_gap
            + stream_lines.len();

        let max_scroll = total_height.saturating_sub(content_area.height as usize);
        MAX_SCROLL.with(|m| m.set(max_scroll));
        let scroll = if app.auto_scroll {
            max_scroll
        } else {
            app.scroll_offset.min(max_scroll)
        };

        frame.render_widget(Clear, content_area);

        // Compute per-message virtual Y ranges (unscrolled).
        STREAMING_VIRTUAL_Y.with(|r| *r.borrow_mut() = None);
        let mut virtual_ys: Vec<(usize, usize)> = Vec::with_capacity(msg_count);
        let mut virtual_y_accum: usize = 0;
        let mut screen_y: u16 = content_area.y;
        let area_bottom = content_area.y + content_area.height;

        for (block_idx, block_lines) in cached.iter().enumerate() {
            let block_height = block_lines.len();
            let is_msg = block_idx < msg_count;
            let bg = if is_msg {
                msg_bg(&app.messages[block_idx])
            } else {
                c_bg_ai()
            };

            if block_idx > 0 {
                virtual_y_accum += gap;
                if virtual_y_accum > scroll && screen_y < area_bottom {
                    screen_y = screen_y.saturating_add(1);
                }
            }

            let start_virtual = virtual_y_accum;
            let end_virtual = virtual_y_accum + block_height;
            virtual_y_accum = end_virtual;

            if is_msg {
                virtual_ys.push((start_virtual, end_virtual));
            }

            if end_virtual <= scroll {
                // Fully above viewport
                continue;
            }

            let skip = scroll.saturating_sub(start_virtual);
            let visible_count = block_height.saturating_sub(skip);
            let remaining = (area_bottom - screen_y) as usize;
            if remaining == 0 {
                continue;
            }

            let render_count = visible_count.min(remaining) as u16;
            let visible_slice = &block_lines[skip..];

            let block_area = Rect {
                x: content_area.x,
                y: screen_y,
                width: content_area.width,
                height: render_count,
            };

            let para = Paragraph::new(Text::from(visible_slice.to_vec()))
                .style(Style::new().bg(bg))
                .block(Block::default().padding(ratatui::widgets::Padding::new(0, 0, 0, 0)))
                .wrap(Wrap { trim: false });
            frame.render_widget(para, block_area);

            screen_y += render_count;
        }

        MSG_VIRTUAL_YS.with(|ys| *ys.borrow_mut() = virtual_ys);

        if !stream_lines.is_empty() {
            let start_virtual = virtual_y_accum + gap;
            let end_virtual = start_virtual + stream_lines.len();

            STREAMING_VIRTUAL_Y.with(|r| *r.borrow_mut() = Some((start_virtual, end_virtual)));

            let remaining = (area_bottom - screen_y) as usize;
            if remaining > 0 {
                let skip = scroll.saturating_sub(start_virtual);
                let visible_count = stream_lines.len().saturating_sub(skip);
                let render_count = visible_count.min(remaining) as u16;
                let visible_slice = &stream_lines[skip..];

                let block_area = Rect {
                    x: content_area.x,
                    y: screen_y,
                    width: content_area.width,
                    height: render_count,
                };

                let para = Paragraph::new(Text::from(visible_slice.to_vec()))
                    .style(Style::new().bg(c_bg_ai()))
                    .block(Block::default().padding(ratatui::widgets::Padding::new(0, 0, 0, 0)))
                    .wrap(Wrap { trim: false });
                frame.render_widget(para, block_area);
            }
        }
    });
}


/// Find the message index corresponding to a screen row.
///
/// Uses virtual Y ranges stored during the last render to compute the mapping,
/// so the result is accurate even when the terminal was resized or scrolled
/// between frames — no stale screen-coordinate capture.
pub fn find_message_idx_from_screen(screen_row: u16, scroll_offset: usize, hint_shown: bool) -> Option<usize> {
    let hint_off = if hint_shown { 1 } else { 0 };
    let virtual_y = (screen_row as usize).saturating_sub(CHAT_AREA_Y as usize + hint_off) + scroll_offset;
    MSG_VIRTUAL_YS.with(|ys| {
        for (idx, &(start, end)) in ys.borrow().iter().enumerate() {
            if virtual_y >= start && virtual_y < end {
                return Some(idx);
            }
        }
        None
    })
}

/// Returns `true` if the screen row falls inside the streaming block.
pub fn streaming_click_target(screen_row: u16, scroll_offset: usize, hint_shown: bool) -> bool {
    let hint_off = if hint_shown { 1 } else { 0 };
    let virtual_y = (screen_row as usize).saturating_sub(CHAT_AREA_Y as usize + hint_off) + scroll_offset;
    STREAMING_VIRTUAL_Y.with(|y| {
        y.borrow()
            .map(|(start, end)| virtual_y >= start && virtual_y < end)
            .unwrap_or(false)
    })
}

pub fn get_max_scroll() -> usize {
    MAX_SCROLL.with(|m| m.get())
}
