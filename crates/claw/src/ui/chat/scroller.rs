use super::components::MessageComponent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use crate::theme::Theme;

pub(crate) struct Scroller {
    offsets: Vec<u16>,
    total_height: u16,
    pub scroll: u16,
    pub viewport_h: u16,
    layout_w: u16,
    spacing: u16,
}

impl Scroller {
    pub fn new(components: &[Box<dyn MessageComponent>], width: u16, viewport_h: u16) -> Self {
        let spacing = 1u16;
        let mut offsets = Vec::with_capacity(components.len());
        let mut total = 0u16;
        for c in components {
            offsets.push(total);
            total += c.height(width) + spacing;
        }
        Self { offsets, total_height: total.saturating_sub(spacing), scroll: 0, viewport_h, layout_w: width, spacing }
    }

    pub fn total(&self) -> u16 { self.total_height }
    pub fn offsets(&self) -> &[u16] { &self.offsets }
    pub fn max_scroll(&self) -> u16 { self.total_height.saturating_sub(self.viewport_h) }

    pub fn set_scroll(&mut self, s: u16) { self.scroll = s.min(self.max_scroll()); }

    pub fn visible_range(&self) -> (usize, usize, u16) {
        if self.offsets.is_empty() { return (0, 0, 0); }
        let scroll_end = self.scroll + self.viewport_h;
        let first = match self.offsets.binary_search(&self.scroll) {
            Ok(i) => i, Err(i) => i.saturating_sub(1),
        };
        let first_offset = self.offsets[first];
        let skip = self.scroll.saturating_sub(first_offset);
        let mut last = first;
        while last < self.offsets.len() {
            let last_end = if last + 1 < self.offsets.len() { self.offsets[last + 1] } else { self.total_height };
            if last_end >= scroll_end { break; }
            last += 1;
        }
        if last + 1 <= self.offsets.len() { last += 1; }
        (first, last.min(self.offsets.len()), skip)
    }

    pub fn render(
        &self, components: &[Box<dyn MessageComponent>], area: Rect, buf: &mut Buffer,
        theme: &Theme, selected: Option<usize>,
    ) {
        let (first, last, skip) = self.visible_range();
        let scroll_top = area.y;
        let dim = Style::default().fg(theme.dim_text());
        for idx in first..last {
            let comp_top = self.offsets[idx].saturating_sub(self.scroll);
            if comp_top >= self.viewport_h { break; }
            let comp_h = components[idx].height(self.layout_w).saturating_sub(skip);
            let y = scroll_top + comp_top;
            let comp_area = Rect { x: area.x, y, width: area.width, height: comp_h.min(self.viewport_h.saturating_sub(comp_top)) };
            if comp_area.height == 0 { continue; }
            components[idx].render(comp_area, buf, theme, selected == Some(idx));
            let sep_y = comp_area.y + comp_area.height;
            if sep_y < area.y + area.height && idx + 1 < components.len() {
                for x in area.x + 3..area.right().saturating_sub(1) {
                    if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, sep_y)) {
                        cell.set_char('─').set_style(dim);
                    }
                }
            }
        }
    }
}
