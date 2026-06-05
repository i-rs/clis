//! Simple message components: User bubble, File edit, System banner, Separator.

use super::{blend, block_border, render_block_chrome, header_line, body_line};
use crate::tui::colors::*;
use crate::tui::ui::chat::render_diff_line;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use std::cell::Cell;

// ═══════════════════════════════════════════════════════════════════════════
//  UserBubble
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) struct UserBubble {
    content: String,
    height_cache: Cell<Option<(u16, u16)>>,
}

impl UserBubble {
    pub fn new(content: &str) -> Self {
        Self { content: content.to_string(), height_cache: Cell::new(None) }
    }
}

impl super::MessageComponent for UserBubble {
    fn height(&self, width: u16) -> u16 {
        if let Some((cw, ch)) = self.height_cache.get() && cw == width { return ch; }
        let lines = self.content.lines().count().max(1);
        let h = 1 + 1 + lines as u16 + 1; // top + header + body + bottom
        self.height_cache.set(Some((width, h)));
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, selected: bool) {
        let border = block_border(selected);
        let bg = if selected { blend(c_bg_user(), c_accent(), 0.10) } else { c_bg_user() };
        let header = header_line("You", "✦", c_cyan(), c_text(), None);
        let body = render_block_chrome(area, buf, border, bg, header, _y_offset);
        for (y, line) in (body.top..).zip(self.content.lines()) {
            if !body.contains(y) { break; }
            let spans: Vec<Span<'static>> = vec![
                Span::raw(" ".repeat(super::BLOCK_INDENT)),
                Span::styled(line.to_string(), Style::default().fg(c_text())),
            ];
            Paragraph::new(Line::from(spans))
                .style(Style::default().bg(bg))
                .render(Rect { y, height: 1, ..area }, buf);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  FileEditBlock
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) struct FileEditBlock {
    path: String,
    summary: String,
    height_cache: Cell<Option<(u16, u16)>>,
}

impl FileEditBlock {
    pub fn new(path: &str, summary: &str) -> Self {
        Self {
            path: path.to_string(),
            summary: summary.to_string(),
            height_cache: Cell::new(None),
        }
    }
}

impl super::MessageComponent for FileEditBlock {
    fn height(&self, width: u16) -> u16 {
        if let Some((cw, ch)) = self.height_cache.get() && cw == width { return ch; }
        let summary_lines = self.summary.lines().count();
        let h = 1 + 1 + summary_lines.max(1) as u16 + 1;
        self.height_cache.set(Some((width, h)));
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, selected: bool) {
        let border = block_border(selected);
        let bg = if selected { blend(c_bg_file(), c_accent(), 0.10) } else { c_bg_file() };
        let header = header_line(&self.path, "✎", c_purple(), c_file_edit(), None);
        let body = render_block_chrome(area, buf, border, bg, header, _y_offset);
        for (y, line) in (body.top..).zip(self.summary.lines()) {
            if !body.contains(y) { break; }
            let mut spans: Vec<Span<'static>> = vec![
                Span::raw(" ".repeat(super::BLOCK_INDENT)),
            ];
            spans.extend(render_diff_line(line));
            Paragraph::new(Line::from(spans))
                .style(Style::default().bg(bg))
                .render(Rect { y, height: 1, ..area }, buf);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  SystemBanner
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) struct SystemBanner {
    content: String,
    height_cache: Cell<Option<(u16, u16)>>,
}

impl SystemBanner {
    pub fn new(content: &str) -> Self {
        Self { content: content.to_string(), height_cache: Cell::new(None) }
    }
}

impl super::MessageComponent for SystemBanner {
    fn height(&self, width: u16) -> u16 {
        if let Some((cw, ch)) = self.height_cache.get() && cw == width { return ch; }
        let lines = self.content.lines().count().max(1);
        let h = 1 + 1 + lines as u16 + 1;
        self.height_cache.set(Some((width, h)));
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, _selected: bool) {
        let bg = c_bg_system();
        let header = header_line("System", "⚙", c_dim(), c_dim(), None);
        let body = render_block_chrome(area, buf, c_border(), bg, header, _y_offset);
        for (y, line) in (body.top..).zip(self.content.lines()) {
            if !body.contains(y) { break; }
            let style = if line.starts_with("──") {
                Style::default().fg(c_summary()).add_modifier(Modifier::BOLD)
            } else if line.starts_with("📄") || line.starts_with("🔧") {
                Style::default().fg(c_orange())
            } else {
                Style::default().fg(c_dim())
            };
            Paragraph::new(body_line(line, style))
                .style(Style::default().bg(bg))
                .render(Rect { y, height: 1, ..area }, buf);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  SeparatorLine
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) struct SeparatorLine {
    label: String,
}

impl SeparatorLine {
    pub fn new(label: &str) -> Self {
        Self { label: label.to_string() }
    }
}

impl super::MessageComponent for SeparatorLine {
    fn height(&self, _w: u16) -> u16 { 1 }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, _selected: bool) {
        let sep = if self.label.is_empty() {
            "── ──".to_string()
        } else {
            format!("── {} ──", self.label)
        };
        let spans = vec![Span::styled(
            format!("    {}", sep),
            Style::default().fg(c_border()),
        )];
        Paragraph::new(Line::from(spans))
            .style(Style::default())
            .render(Rect { y: area.y, height: 1, ..area }, buf);
    }
}
