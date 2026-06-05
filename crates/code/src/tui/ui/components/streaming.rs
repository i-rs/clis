//! Streaming-block component — rendered every frame while the
//! assistant is generating. Uses incremental content-length caching
//! for the AI body text. Now renders actual reasoning content and
//! uses consistent gutter bar styling.

use super::{BLOCK_INDENT, header_line, rounded_top};
use crate::app::StreamingState;
use crate::tui::colors::*;
use crate::tui::ui::chat::{render_ai_content, tool_glyph};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use std::cell::{Cell, RefCell};

pub(crate) struct StreamingBlock {
    /// Snapshot of streaming state at last rebuild.
    content_len: usize,
    reasoning: String,
    has_tool: bool,
    tool_name: String,
    reasoning_collapsed: bool,
    /// Incremental cache: (content_len, rendered_lines)
    content_cache: RefCell<(usize, Vec<Line<'static>>)>,
    height_cache: Cell<Option<(u16, u16)>>,
}

impl StreamingBlock {
    pub fn new(s: &StreamingState) -> Self {
        let content_cache = if s.content.is_empty() {
            (0, Vec::new())
        } else {
            (s.content.len(), render_ai_content(&s.content))
        };

        Self {
            content_len: s.content.len(),
            reasoning: s.reasoning.clone(),
            has_tool: s.current_tool.is_some(),
            tool_name: s.current_tool.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
            reasoning_collapsed: s.reasoning_collapsed,
            content_cache: RefCell::new(content_cache),
            height_cache: Cell::new(None),
        }
    }

    fn reasoning_rows(&self) -> u16 {
        if self.reasoning.is_empty() || self.reasoning_collapsed {
            0
        } else {
            self.reasoning.lines().count() as u16
        }
    }
}

impl super::MessageComponent for StreamingBlock {
    fn height(&self, width: u16) -> u16 {
        if let Some((cw, ch)) = self.height_cache.get() && cw == width { return ch; }
        let mut h = 2u16; // top border + header
        if self.has_tool { h += 1; }
        if !self.reasoning.is_empty() {
            h += 1; // toggle row
            if !self.reasoning_collapsed {
                h += self.reasoning_rows();
            }
        }
        h += 1; // cursor row (min body)
        self.height_cache.set(Some((width, h)));
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, _selected: bool) {
        let bg = c_bg_ai();
        let bg_style = Style::default().bg(bg);

        let header = header_line("Assistant", "◆", c_green(), c_green(), None);

        // Top border
        Paragraph::new(rounded_top(area.width, c_border()))
            .style(bg_style)
            .render(Rect { y: area.y, height: 1, ..area }, buf);
        // Header
        Paragraph::new(header)
            .style(bg_style)
            .render(Rect { y: area.y + 1, height: 1, ..area }, buf);

        let mut y = area.y + 2;
        let body_end = area.y + area.height;

        // ─── Tool indicator ──────────────────────────────────────
        if self.has_tool {
            let tc_spans: Vec<Span<'static>> = vec![
                Span::raw(" ".repeat(BLOCK_INDENT)),
                Span::styled("● ", Style::default().fg(c_accent())),
                Span::styled(
                    format!("{} {} running...", tool_glyph(&self.tool_name), self.tool_name),
                    Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
                ),
            ];
            if y < body_end {
                Paragraph::new(Line::from(tc_spans))
                    .style(bg_style)
                    .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }
        }

        // ─── Reasoning ───────────────────────────────────────────
        if !self.reasoning.is_empty() {
            if self.reasoning_collapsed {
                if y < body_end {
                    let spans = vec![
                        Span::raw(" ".repeat(BLOCK_INDENT)),
                        Span::styled("🧠", Style::default().fg(c_yellow())),
                        Span::raw(" "),
                        Span::styled("思考过程...", Style::default().fg(c_dim())),
                    ];
                    Paragraph::new(Line::from(spans))
                        .style(bg_style)
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
            } else {
                if y < body_end {
                    let spans = vec![
                        Span::raw(" ".repeat(BLOCK_INDENT)),
                        Span::styled("🧠", Style::default().fg(c_yellow())),
                        Span::raw(" "),
                        Span::styled("思考过程", Style::default().fg(c_yellow()).add_modifier(Modifier::BOLD)),
                    ];
                    Paragraph::new(Line::from(spans))
                        .style(bg_style)
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
                // ─── Render actual reasoning lines ───────────────
                let reason_style = Style::default().fg(c_dim()).add_modifier(Modifier::ITALIC);
                for rl in self.reasoning.lines() {
                    if y >= body_end { break; }
                    let indent = BLOCK_INDENT + 2;
                    let spans = vec![
                        Span::raw(" ".repeat(indent)),
                        Span::styled("┊ ", Style::default().fg(c_muted())),
                        Span::styled(rl.to_string(), reason_style),
                    ];
                    Paragraph::new(Line::from(spans))
                        .style(bg_style)
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
            }
        }

        // ─── Content (cached render) ─────────────────────────────
        let cache = self.content_cache.borrow();
        let lines = &cache.1;
        for line in lines {
            if y >= body_end { break; }
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 2);
            spans.push(Span::raw(" ".repeat(BLOCK_INDENT)));
            for s in &line.spans {
                spans.push(Span::styled(s.content.clone(), s.style));
            }
            Paragraph::new(Line::from(spans))
                .style(bg_style)
                .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        // ─── Cursor glyph ────────────────────────────────────────
        let glyph = if self.content_len > 0 { " ▊" } else if self.has_tool { " ▸" } else { " ⏳" };
        if y < body_end {
            let cursor = Span::styled(glyph, Style::default().fg(c_green()));
            Paragraph::new(Line::from(vec![
                Span::raw(" ".repeat(BLOCK_INDENT)),
                cursor,
            ]))
            .style(bg_style)
            .render(Rect { y, height: 1, ..area }, buf);
        }
    }
}
