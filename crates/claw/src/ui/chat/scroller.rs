use super::components::MessageComponent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use crate::theme::Theme;
use std::cell::RefCell;
use std::rc::Rc;

/// Concrete cell type used to hold a per-message component. The
/// `Box` provides a sized wrapper that the `RefCell` can own; method
/// calls auto-deref through it to the underlying trait object.
pub(crate) type ComponentCell = Rc<RefCell<Box<dyn MessageComponent>>>;

/// Clickable region for a single message component. Coordinates are
/// relative to the chat content area (i.e. the top border row is
/// excluded). `component_idx` is the index into the original
/// `messages`/`components` Vec that the App holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitRegion {
    pub component_idx: usize,
    pub y_start: u16,
    pub y_end: u16,
    pub x_start: u16,
    pub x_end: u16,
}

pub(crate) struct Scroller {
    offsets: Vec<u16>,
    total_height: u16,
    pub scroll: u16,
    pub viewport_h: u16,
    layout_w: u16,
    spacing: u16,
    /// Click regions for every component that opts in via
    /// `MessageComponent::clickable() == true`. `y_start`/`y_end`
    /// are content-relative rows in the same space as `offsets`.
    pub hits: Vec<HitRegion>,
}

#[cfg(test)]
struct StubBlock {
    h: u16,
    click: bool,
}
#[cfg(test)]
impl MessageComponent for StubBlock {
    fn height(&self, _w: u16) -> u16 { self.h }
    fn render(&self, area: Rect, buf: &mut Buffer, _theme: &Theme, _selected: bool) {
        // Draw a top and bottom border so the scroller test can verify
        // which row each block actually occupies.
        for x in area.x..area.x + area.width {
            if let Some(c) = buf.cell_mut(ratatui::layout::Position::new(x, area.y)) {
                c.set_symbol("─");
            }
            if area.height > 1
                && let Some(c) = buf.cell_mut(ratatui::layout::Position::new(x, area.y + area.height - 1))
            {
                c.set_symbol("─");
            }
        }
    }
    fn clickable(&self) -> bool { self.click }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(heights: &[u16]) -> Vec<ComponentCell> {
        heights
            .iter()
            .map(|h| {
                Rc::new(RefCell::new(Box::new(StubBlock { h: *h, click: false }) as Box<dyn MessageComponent>))
            })
            .collect()
    }

    fn has_border(buf: &Buffer, row: u16, area: &Rect) -> bool {
        (area.x..area.x + area.width).all(|x| {
            buf.cell(ratatui::layout::Position::new(x, row))
                .map(|c| c.symbol() == "─")
                .unwrap_or(false)
        })
    }

    #[test]
    fn render_with_partial_top_component_does_not_squeeze_later_blocks() {
        // Layout: 3 blocks of height 6 / 4 / 4. Spacing = 1.
        // Offsets: [0, 7, 12], total = 16. Layout width 80, viewport 10.
        let comps = blocks(&[6, 4, 4]);
        let mut scr = Scroller::new(&comps, 80, 10);
        // Scroll 2 rows into the first block: skip = 2.
        scr.set_scroll(2);
        let area = Rect { x: 0, y: 0, width: 80, height: 11 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::from_preset("midnight").unwrap_or_default();
        scr.render(&comps, area, &mut buf, &theme, None);

        // First block should render with height 6 - 2 = 4.
        //   top border at row 0, bottom border at row 3.
        // Second block starts at offset 7 → comp_top 7-2 = 5, should
        // render with FULL height 4 (the pre-fix bug rendered it at
        // 4-2 = 2). Top border at row 5, bottom border at row 8.
        assert!(has_border(&buf, 0, &area), "first block top border at row 0");
        assert!(has_border(&buf, 3, &area), "first block bottom border at row 3");
        assert!(has_border(&buf, 5, &area), "second block top border at row 5");
        assert!(has_border(&buf, 8, &area), "second block bottom border at row 8 (height 4, not 2)");
    }

    #[test]
    fn render_at_top_renders_all_blocks_with_full_height() {
        let comps = blocks(&[4, 4, 4]);
        let scr = Scroller::new(&comps, 80, 20);
        let area = Rect { x: 0, y: 0, width: 80, height: 20 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::from_preset("midnight").unwrap_or_default();
        scr.render(&comps, area, &mut buf, &theme, None);

        // Block 0: rows 0..3, Block 1: rows 5..8, Block 2: rows 10..13.
        assert!(has_border(&buf, 0, &area));
        assert!(has_border(&buf, 3, &area));
        assert!(has_border(&buf, 5, &area));
        assert!(has_border(&buf, 8, &area));
        assert!(has_border(&buf, 10, &area));
    }

    #[test]
    fn hit_regions_cover_clickable_components_only() {
        // Three blocks: middle is clickable. The whole block (no
        // spacing) is the click target. Offsets: [0, 4, 8]; total 11.
        let comps: Vec<ComponentCell> = vec![
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: true }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
        ];
        let scr = Scroller::new(&comps, 80, 20);
        assert_eq!(scr.hits.len(), 1, "only one clickable component");
        let h = scr.hits[0];
        assert_eq!(h.component_idx, 1);
        assert_eq!((h.y_start, h.y_end), (4, 7));
        // x covers the full chat width
        assert_eq!(h.x_start, 0);
        assert_eq!(h.x_end, 80);
    }

    #[test]
    fn hit_test_returns_correct_component_for_click() {
        let comps: Vec<ComponentCell> = vec![
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: true }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
        ];
        let scr = Scroller::new(&comps, 80, 20);
        // Click on row 5 (within block 1) → component 1
        assert_eq!(scr.hit_test(5, 10), Some(1));
        // Click on row 0 (block 0, not clickable) → None
        assert_eq!(scr.hit_test(0, 10), None);
        // Click on row 9 (block 2, not clickable) → None
        assert_eq!(scr.hit_test(9, 10), None);
    }
}

impl Scroller {
    pub fn new(components: &[ComponentCell], width: u16, viewport_h: u16) -> Self {
        // Each block has its own rounded border, so the only spacing we
        // need between blocks is one empty row of breathing room.
        // Zero-height components (e.g. filtered empty messages) should
        // *not* contribute spacing — otherwise a sequence of empties
        // would waste a row each.
        let spacing = 1u16;
        let mut offsets = Vec::with_capacity(components.len());
        let mut hits = Vec::new();
        let mut total = 0u16;
        let mut last_was_real = false;
        for (i, c) in components.iter().enumerate() {
            let h = c.borrow().height(width);
            // Don't insert spacing for the very first row, and don't
            // insert spacing after a zero-height component.
            if i > 0 && h > 0 && last_was_real {
                total = total.saturating_add(spacing);
            }
            offsets.push(total);
            if c.borrow().clickable() && h > 0 {
                hits.push(HitRegion {
                    component_idx: i,
                    y_start: total,
                    y_end: total.saturating_add(h),
                    x_start: 0,
                    x_end: width,
                });
            }
            total = total.saturating_add(h);
            last_was_real = h > 0;
        }
        Self {
            offsets,
            total_height: total,
            scroll: 0,
            viewport_h,
            layout_w: width,
            spacing,
            hits,
        }
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

    /// Map a content-relative (row, col) click to the component it
    /// hits, if any. Returns `None` for non-clickable regions.
    pub fn hit_test(&self, row: u16, col: u16) -> Option<usize> {
        // Hits are stored in component order; the hit list is short
        // (only clickable blocks), so a linear scan is fine.
        for h in &self.hits {
            if row >= h.y_start && row < h.y_end && col >= h.x_start && col < h.x_end {
                return Some(h.component_idx);
            }
        }
        None
    }

    pub fn render(
        &self, components: &[ComponentCell], area: Rect, buf: &mut Buffer,
        theme: &Theme, selected: Option<usize>,
    ) {
        let (first, last, skip) = self.visible_range();
        let scroll_top = area.y;
        for idx in first..last {
            let comp_top = self.offsets[idx].saturating_sub(self.scroll);
            if comp_top >= self.viewport_h { break; }
            // `skip` is the number of rows of the *first* visible
            // component that are scrolled off the top of the viewport.
            // It only applies to that component. Subtracting it from
            // every visible component squeezes later blocks — short
            // blocks (tool_call collapsed, user 1-line, quality with
            // 0 issues) get clipped to 0 rows and disappear, while
            // taller ones lose their body content.
            let full_h = components[idx].borrow().height(self.layout_w);
            let comp_h = if idx == first {
                full_h.saturating_sub(skip)
            } else {
                full_h
            };
            let y = scroll_top + comp_top;
            let comp_area = Rect { x: area.x, y, width: area.width, height: comp_h.min(self.viewport_h.saturating_sub(comp_top)) };
            if comp_area.height == 0 { continue; }
            components[idx].borrow().render(comp_area, buf, theme, selected == Some(idx));
        }
    }
}
