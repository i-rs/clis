mod chat;
pub mod components;
mod input_bar;

use crate::app::App;
use crate::tui::colors::*;
use crate::tui::ui::components::Scroller;
pub use input_bar::filtered_slash_commands;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph},
};
use std::cell::{Cell, RefCell};

// Scroller cache: (msg_gen, width, layout_gen, scroller)
type StreamingCache = (
    usize,
    String,
    Option<Box<dyn crate::tui::ui::components::MessageComponent>>,
);
thread_local! {
    static SCROLLER_CACHE: RefCell<(usize, u16, usize, Scroller)> = const {
        RefCell::new((0, 0, 0, Scroller::new_empty()))
    };
    static MAX_SCROLL: Cell<usize> = const { Cell::new(0) };
    static STREAMING_CACHE: RefCell<StreamingCache> = RefCell::new((0, String::new(), None));
}

pub const SIDEBAR_WIDTH: u16 = 40;

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
        vec![
            Span::raw("  "),
            Span::styled(pct_str, Style::new().fg(ctx_color)),
        ]
    } else {
        vec![]
    };

    let sel_text = app
        .selected_message
        .map(|idx| Span::styled(format!(" #{} ", idx), Style::new().fg(c_cyan())));

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
        let hint_para = Paragraph::new(Text::from(vec![hint_line])).style(Style::new().bg(c_bg()));
        frame.render_widget(hint_para, hint_area);
    }

    frame.render_widget(Clear, content_area);

    // ─── Get or compute scroller ─────────────────────────────────
    SCROLLER_CACHE.with(|cache| {
        let mut cache_ref = cache.borrow_mut();
        let (ref mut cached_gen, ref mut cached_w, ref mut cached_layout, ref mut scroller) =
            *cache_ref;

        let need_recompute = *cached_w != content_area.width
            || *cached_gen != app.msg_gen
            || *cached_layout != app.layout_gen
            || app.components.len() != scroller.component_count();

        if need_recompute {
            let new_scroller = Scroller::compute(&app.components, content_area.width);
            *cached_gen = app.msg_gen;
            *cached_w = content_area.width;
            *cached_layout = app.layout_gen;
            *scroller = new_scroller;
        }

        let max_scroll = scroller.max_scroll(content_area.height);
        MAX_SCROLL.with(|m| m.set(max_scroll));

        let scroll = if app.auto_scroll {
            max_scroll
        } else {
            app.scroll_offset.min(max_scroll)
        };

        // Render visible components
        scroller.render(
            &app.components,
            scroll,
            content_area,
            frame.buffer_mut(),
            app.selected_message,
        );

        // ─── Streaming block (cached, rebuilt only on content change) ──
        if let Some(ref s) = app.streaming {
            STREAMING_CACHE.with(|cache_ref| {
                let mut cache = cache_ref.borrow_mut();
                let rebuild =
                    cache.1 != s.reasoning || cache.0 != s.content.len() || cache.2.is_none();
                if rebuild {
                    cache.0 = s.content.len();
                    cache.1 = s.reasoning.clone();
                    cache.2 = Some(components::build_streaming_component(s));
                }
                if let Some(ref streaming_comp) = cache.2 {
                    let h = streaming_comp.height(content_area.width) as usize;

                    let total_height = scroller.total_height();
                    let viewport_end = scroll + content_area.height as usize;

                    if total_height < viewport_end {
                        let content_bottom = total_height.saturating_sub(scroll);
                        let screen_for_stream = content_area.y
                            + content_bottom.min(content_area.height as usize) as u16;

                        if screen_for_stream < content_area.y + content_area.height {
                            let stream_area = Rect {
                                x: content_area.x,
                                y: screen_for_stream,
                                width: content_area.width,
                                height: (content_area.height
                                    - (screen_for_stream - content_area.y))
                                    .min(h as u16),
                            };
                            if stream_area.height > 0 {
                                streaming_comp.render(stream_area, frame.buffer_mut(), 0, false);
                            }
                        }
                    }
                }
            });
        }
    });
}

/// Find message index from a screen row.
pub fn find_message_idx_from_screen(
    screen_row: u16,
    scroll_offset: usize,
    hint_shown: bool,
    chat_area_y: u16,
) -> Option<usize> {
    SCROLLER_CACHE.with(|cache| {
        let (_, _, _, ref scroller) = *cache.borrow();
        let hit_off = if hint_shown { 1 } else { 0 };
        let content_row = (screen_row as usize).saturating_sub(chat_area_y as usize + hit_off);
        let virtual_y = content_row + scroll_offset;
        scroller
            .component_at_virtual_y(virtual_y)
            .map(|(idx, _)| idx)
    })
}

/// Find click target (extra_click_targets or fallback toggle) at a screen row.
pub fn click_op_at_screen(
    screen_row: u16,
    scroll_offset: usize,
    hint_shown: bool,
    chat_area_y: u16,
    components: &[crate::tui::ui::components::ComponentCell],
) -> Option<crate::tui::ui::components::ComponentOp> {
    SCROLLER_CACHE.with(|cache| {
        let (_, _, _, ref scroller) = *cache.borrow();
        let hit_off = if hint_shown { 1 } else { 0 };
        let content_row = (screen_row as usize).saturating_sub(chat_area_y as usize + hit_off);
        let virtual_y = content_row + scroll_offset;
        scroller.click_target_at(virtual_y, components)
    })
}

/// Returns `true` if the screen row falls inside the streaming block.
pub fn streaming_click_target(
    screen_row: u16,
    scroll_offset: usize,
    hint_shown: bool,
    chat_area_y: u16,
) -> bool {
    SCROLLER_CACHE.with(|cache| {
        let (_, _, _, ref scroller) = *cache.borrow();
        let hit_off = if hint_shown { 1 } else { 0 };
        let content_row = (screen_row as usize).saturating_sub(chat_area_y as usize + hit_off);
        let virtual_y = content_row + scroll_offset;
        virtual_y >= scroller.total_height()
    })
}

pub fn get_max_scroll() -> usize {
    MAX_SCROLL.with(|m| m.get())
}
