//! Virtual scroll manager for chat messages.
//!
//! Tracks per-component heights and scroll offset, determines which
//! components are visible and where to render them.

use super::components::MessageComponent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use crate::theme::Theme;

/// A pre-computed snapshot of component visibility for one frame.
pub(crate) struct Scroller {
    /// Component index → y-offset from top of content area (0 = top).
    offsets: Vec<u16>,
    /// Total content height in rows.
    total_height: u16,
    /// Current scroll offset (rows scrolled from top).
    pub scroll: u16,
    /// Viewport height.
    pub viewport_h: u16,
    /// Width for layout calculations.
    layout_w: u16,
}

impl Scroller {
    pub fn new(components: &[Box<dyn MessageComponent>], width: u16, viewport_h: u16) -> Self {
        let mut offsets = Vec::with_capacity(components.len());
        let mut total = 0u16;
        for c in components {
            offsets.push(total);
            total += c.height(width);
        }
        Self { offsets, total_height: total, scroll: 0, viewport_h, layout_w: width }
    }

    pub fn total(&self) -> u16 { self.total_height }
    pub fn max_scroll(&self) -> u16 { self.total_height.saturating_sub(self.viewport_h) }

    /// Set scroll, clamped to valid range.
    pub fn set_scroll(&mut self, s: u16) {
        self.scroll = s.min(self.max_scroll());
    }

    /// Rebuild heights after component state change (expand/collapse).
    pub fn rebuild(&mut self, components: &[Box<dyn MessageComponent>]) {
        self.offsets.clear();
        let mut total = 0u16;
        for c in components {
            self.offsets.push(total);
            total += c.height(self.layout_w);
        }
        self.total_height = total;
        self.scroll = self.scroll.min(self.max_scroll());
    }

    /// Returns (first_visible_idx, last_visible_idx_exclusive, skip_lines_for_first).
    pub fn visible_range(&self) -> (usize, usize, u16) {
        if self.offsets.is_empty() { return (0, 0, 0); }
        let scroll_end = self.scroll + self.viewport_h;

        // Find first visible: binary search for the last offset <= scroll
        let first = match self.offsets.binary_search(&self.scroll) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let first_offset = self.offsets[first];
        let skip = self.scroll.saturating_sub(first_offset);

        let mut last = first;
        while last < self.offsets.len() {
            let last_end = if last + 1 < self.offsets.len() {
                self.offsets[last + 1]
            } else {
                self.total_height
            };
            if last_end >= scroll_end { last = (last + 1).min(self.offsets.len()); break; }
            last += 1;
        }

        (first, last.min(self.offsets.len()), skip)
    }

    pub fn offset_of(&self, idx: usize) -> u16 {
        self.offsets.get(idx).copied().unwrap_or(0)
    }

    /// Render all visible components into the given frame area.
    pub fn render(
        &self,
        components: &[Box<dyn MessageComponent>],
        area: Rect,
        buf: &mut Buffer,
        theme: &Theme,
        selected: Option<usize>,
    ) {
        let (first, last, skip) = self.visible_range();
        let scroll_top = area.y;

        for idx in first..last {
            let comp_top = self.offsets[idx].saturating_sub(self.scroll);
            if comp_top >= self.viewport_h { break; }

            let comp_h = components[idx].height(self.layout_w).saturating_sub(skip);

            let comp_area = Rect {
                x: area.x,
                y: scroll_top + comp_top,
                width: area.width,
                height: comp_h.min(self.viewport_h.saturating_sub(comp_top)),
            };
            if comp_area.height == 0 { continue; }

            let is_sel = selected == Some(idx);
            components[idx].render(comp_area, buf, theme, is_sel);
        }
    }
}
