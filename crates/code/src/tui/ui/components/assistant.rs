//! Assistant message component with cached height.
//!
//! - Rounded block chrome (`╭─╮`/`╰─╯`)
//! - Header with `◆ Assistant` label
//! - AI content body via `render_ai_content`
//! - Collapsible `🧠 思考过程` reasoning section with gutter bar
//! - Height cached per width

use super::{ComponentOp, blend, block_border, render_block_chrome, body_line};
use crate::tui::colors::*;
use crate::tui::ui::chat::render_ai_content;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use std::cell::{Cell, RefCell};

pub(crate) struct AssistantBlock {
    text: String,
    reasoning: String,
    pub reasoning_expanded: bool,
    /// Pre-computed body row count (cached once in new()).
    body_rows: usize,
    /// `(width, height)` cache
    height_cache: Cell<Option<(u16, u16)>>,
    reasoning_height_cache: Cell<Option<u16>>,
    /// Cache for parsed body: `(text_len, width, lines)`
    body_render_cache: RefCell<Option<(usize, u16, Vec<Line<'static>>)>>,
}

impl AssistantBlock {
    pub fn new(text: &str, reasoning: &str, reasoning_expanded: bool) -> Self {
        let body_rows = if text.is_empty() { 1 } else {
            render_ai_content(text).len().max(1)
        };
        Self {
            text: text.to_string(),
            reasoning: reasoning.to_string(),
            reasoning_expanded,
            body_rows,
            height_cache: Cell::new(None),
            reasoning_height_cache: Cell::new(None),
            body_render_cache: RefCell::new(None),
        }
    }

    fn reasoning_rows(&self) -> u16 {
        if self.reasoning.is_empty() { return 0; }
        if let Some(h) = self.reasoning_height_cache.get() { return h; }
        let h = self.reasoning.lines().count() as u16;
        self.reasoning_height_cache.set(Some(h));
        h
    }
}

impl super::MessageComponent for AssistantBlock {
    /// 1 (top border) + 1 (header) + body + (optional reasoning toggle + rows) + 1 (bottom)
    fn height(&self, width: u16) -> u16 {
        if let Some((cw, ch)) = self.height_cache.get() && cw == width {
            return ch;
        }
        let mut h = 1 + 1 + self.body_rows as u16 + 1;
        if !self.reasoning.is_empty() {
            h += 1; // toggle row
            if self.reasoning_expanded {
                h += self.reasoning_rows();
            }
        }
        self.height_cache.set(Some((width, h)));
        h
    }

    fn clickable(&self) -> bool {
        !self.reasoning.is_empty()
    }

    fn apply(&mut self, op: ComponentOp) {
        if op == ComponentOp::Toggle {
            self.reasoning_expanded = !self.reasoning_expanded;
            self.height_cache.set(None); // invalidate height
        }
    }

    fn extra_click_targets(&self, _width: u16) -> Vec<(u16, u16, ComponentOp)> {
        if self.reasoning.is_empty() { return vec![]; }
        let toggle_y = 1 + 1 + self.body_rows as u16;
        vec![(toggle_y, 1, ComponentOp::Toggle)]
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, selected: bool) {
        let border = block_border(selected);
        let interior_bg = if selected {
            blend(c_bg_ai(), c_accent(), 0.12)
        } else {
            c_bg_ai()
        };

        let header = super::header_line(
            "Assistant", "◆", c_green(), c_text(), None,
        );

        let body = render_block_chrome(area, buf, border, interior_bg, header, _y_offset);
        let mut y = body.top;

        // ─── Body content (cached render) ─────────────────────────────────
        let body_lines = if self.text.is_empty() {
            vec![body_line("...", Style::default().fg(c_dim()).italic())]
        } else {
            let mut cache = self.body_render_cache.borrow_mut();
            let key = (self.text.len(), area.width);
            if let Some((ref cached_len, ref cached_w, ref cached_lines)) = *cache {
                if *cached_len == key.0 && *cached_w == key.1 {
                    cached_lines.clone()
                } else {
                    let lines = render_ai_content(&self.text);
                    *cache = Some((key.0, key.1, lines.clone()));
                    lines
                }
            } else {
                let lines = render_ai_content(&self.text);
                *cache = Some((key.0, key.1, lines.clone()));
                lines
            }
        };

        for line in &body_lines {
            if !body.contains(y) { break; }
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 2);
            spans.push(Span::raw(" ".repeat(super::BLOCK_INDENT)));
            for s in &line.spans {
                spans.push(Span::styled(s.content.clone(), s.style));
            }
            Paragraph::new(Line::from(spans))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        // ─── Reasoning toggle + content ──────────────────────────────
        if !self.reasoning.is_empty() {
            let chevron = if self.reasoning_expanded { "▾" } else { "▸" };
            let char_count = self.reasoning.chars().count();
            let chars_label = if char_count >= 1000 {
                format!("{}k chars", char_count / 1000)
            } else {
                format!("{} chars", char_count)
            };

            let mut spans: Vec<Span<'static>> = Vec::with_capacity(6);
            spans.push(Span::raw(" ".repeat(super::BLOCK_INDENT)));
            spans.push(Span::styled("🧠", Style::default().fg(c_yellow())));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                "思考过程".to_string(),
                Style::default().fg(c_yellow()).add_modifier(Modifier::BOLD),
            ));
            let used: usize = spans.iter().map(|s| s.content.len()).sum();
            if used + chars_label.chars().count() + 3 <= (area.width as usize).saturating_sub(2) {
                spans.push(Span::styled(
                    format!("  {}  {}", chars_label, chevron),
                    Style::default().fg(c_muted()),
                ));
            } else if used + 3 <= area.width as usize {
                spans.push(Span::styled(
                    format!("  {}", chevron),
                    Style::default().fg(c_muted()).add_modifier(Modifier::BOLD),
                ));
            }

            if body.contains(y) {
                Paragraph::new(Line::from(spans))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }

            if self.reasoning_expanded {
                let reason_style = Style::default()
                    .fg(c_dim())
                    .add_modifier(Modifier::ITALIC);
                for rl in self.reasoning.lines() {
                    if !body.contains(y) { break; }
                    let indent = super::BLOCK_INDENT + 2; // extra indent for thinking content
                    let usable = (area.width as usize).saturating_sub(indent + 4).max(8);
                    let truncated: String = {
                        let mut s = String::with_capacity(usable.min(rl.len()));
                        let mut col = 0usize;
                        for c in rl.chars() {
                            let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
                            if col + w > usable { s.push('…'); break; }
                            col += w;
                            s.push(c);
                        }
                        s
                    };
                    let spans: Vec<Span<'static>> = vec![
                        Span::raw(" ".repeat(indent)),
                        Span::styled("┊ ", Style::default().fg(c_muted())),
                        Span::styled(truncated, reason_style),
                    ];
                    Paragraph::new(Line::from(spans))
                        .style(Style::default().bg(interior_bg))
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
            }
        }
    }
}
