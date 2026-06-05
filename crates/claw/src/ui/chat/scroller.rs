use super::components::MessageComponent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use crate::theme::Theme;
use ratatui_interact::traits::ClickRegionRegistry;
use std::cell::RefCell;
use std::rc::Rc;

/// Concrete cell type used to hold a per-message component. The
/// `Box` provides a sized wrapper that the `RefCell` can own; method
/// calls auto-deref through it to the underlying trait object.
pub(crate) type ComponentCell = Rc<RefCell<Box<dyn MessageComponent>>>;

/// Content-relative layout entry for one clickable component. The
/// Scroller keeps a Vec of these internally to make per-component
/// Y math easy; the App-facing click dispatch goes through the
/// library's `ClickRegionRegistry` instead, so the public API
/// doesn't expose this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HitRegion {
    component_idx: usize,
    y_start: u16,
    y_end: u16,
}

pub(crate) struct Scroller {
    offsets: Vec<u16>,
    total_height: u16,
    pub scroll: u16,
    pub viewport_h: u16,
    layout_w: u16,
    /// Content-relative click regions for every component that opts
    /// in via `MessageComponent::clickable() == true`. `y_start` /
    /// `y_end` are rows in the same space as `offsets` (0 = top of
    /// the chat content). `register_clicks` translates these into
    /// screen-absolute `Rect`s when populating the click registry.
    hits: Vec<HitRegion>,
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
    fn register_clicks_only_registers_clickable_components() {
        // Three blocks: middle is clickable. Spacing=1, so offsets
        // are [0, 4, 8] — the clickable block occupies rows 4..7.
        let comps: Vec<ComponentCell> = vec![
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: true }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
        ];
        let scr = Scroller::new(&comps, 80, 20);
        let mut reg: ClickRegionRegistry<usize> = ClickRegionRegistry::new();
        scr.register_clicks(Rect { x: 0, y: 0, width: 80, height: 20 }, &mut reg);
        assert_eq!(reg.len(), 1, "only one clickable component");
        // The middle block was registered with `data = 1`. It lives
        // on rows 4..7, so a click at (col=0, row=5) hits it.
        assert_eq!(reg.handle_click(0, 5), Some(&1));
        // Non-clickable blocks: no region, so no hit.
        assert_eq!(reg.handle_click(0, 1), None);
        assert_eq!(reg.handle_click(0, 9), None);
    }

    #[test]
    fn register_clicks_translates_to_screen_absolute_rect() {
        // Clickable block at content-y [4, 7), with a chat pane at
        // screen y=10..30. After translation the screen-absolute
        // rect should be (0, 14, 80, 3).
        let comps: Vec<ComponentCell> = vec![
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: true }) as Box<dyn MessageComponent>)),
        ];
        let scr = Scroller::new(&comps, 80, 20);
        let mut reg: ClickRegionRegistry<usize> = ClickRegionRegistry::new();
        scr.register_clicks(Rect { x: 0, y: 10, width: 80, height: 20 }, &mut reg);
        // Click at screen row 15 should still hit the block (it
        // occupies screen rows 10+4..10+7 = 14..17).
        assert_eq!(reg.handle_click(0, 15), Some(&1));
        // Click above the block: no hit.
        assert_eq!(reg.handle_click(0, 13), None);
    }
    #[test]
    fn register_clicks_with_scroll_offset_adjusts_y() {
        // Two blocks: heights [3, 3], spacing=1 → total=7, offsets [0, 4].
        // Clickable block at content-y [4, 7). Viewport is 5 so
        // max_scroll=2, making set_scroll(2) work.
        let comps: Vec<ComponentCell> = vec![
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: false }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 3, click: true }) as Box<dyn MessageComponent>)),
        ];
        let mut scr = Scroller::new(&comps, 80, 5);
        scr.set_scroll(2);
        let mut reg: ClickRegionRegistry<usize> = ClickRegionRegistry::new();
        // Pane at screen y=10, height 5.
        scr.register_clicks(Rect { x: 0, y: 10, width: 80, height: 5 }, &mut reg);
        // After scroll=2 the block shifts from content-y 4..7 to
        // screen rows 10+(4-2)..10+(6-2) = 12..15.
        assert_eq!(reg.handle_click(0, 12), Some(&1), "click within shifted block (top edge)");
        assert_eq!(reg.handle_click(0, 14), Some(&1), "click within shifted block (middle)");
        // The old code would have registered at screen-y 14..17 but
        // after shift the block is actually at 12..15, so a click at
        // row 16 should now miss.
        assert_eq!(reg.handle_click(0, 16), None, "click below shifted block");
        assert_eq!(reg.handle_click(0, 11), None, "click above shifted block");
    }

    #[test]
    fn register_clicks_skips_fully_scrolled_off_components() {
        // Three blocks: heights [2, 4, 2], spacing=1 → total=10,
        // offsets [0, 3, 8]. Clickable middle block at y_start=3,
        // y_end=7. Viewport=2 gives max_scroll=8. At scroll >= 7
        // the clickable block is completely above the viewport.
        let comps: Vec<ComponentCell> = vec![
            Rc::new(RefCell::new(Box::new(StubBlock { h: 2, click: false }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 4, click: true }) as Box<dyn MessageComponent>)),
            Rc::new(RefCell::new(Box::new(StubBlock { h: 2, click: false }) as Box<dyn MessageComponent>)),
        ];
        let mut scr = Scroller::new(&comps, 80, 2);
        scr.set_scroll(8);
        let mut reg: ClickRegionRegistry<usize> = ClickRegionRegistry::new();
        scr.register_clicks(Rect { x: 0, y: 0, width: 80, height: 2 }, &mut reg);
        assert_eq!(reg.len(), 0, "no visible clickable components");
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
            hits,
        }
    }

    pub fn max_scroll(&self) -> u16 { self.total_height.saturating_sub(self.viewport_h) }

    pub fn set_scroll(&mut self, s: u16) { self.scroll = s.min(self.max_scroll()); }

    /// Translate the content-relative click regions into
    /// screen-absolute `Rect`s and register them into `registry`.
    ///
    /// `pane` is the `Rect` that the chat content occupies on the
    /// current frame; its `x`/`y` are added to every region's
    /// coordinates and its `width` covers the full chat width. The
    /// registry is cleared before being repopulated, so the caller
    /// can keep owning it across frames.
    pub fn register_clicks(&self, pane: Rect, registry: &mut ClickRegionRegistry<usize>) {
        registry.clear();
        let scroll_end = self.scroll.saturating_add(self.viewport_h);
        for h in &self.hits {
            // Clip hit region to the visible viewport and translate
            // to screen-absolute coordinates.  Without the scroll
            // adjustment the registered regions drift off-target as
            // the user scrolls, making click targets unresponsive.
            let visible_start = h.y_start.max(self.scroll);
            let visible_end = h.y_end.min(scroll_end);

            if visible_end > visible_start {
                let area = Rect {
                    x: pane.x,
                    y: pane.y + visible_start.saturating_sub(self.scroll),
                    width: pane.width,
                    height: visible_end.saturating_sub(visible_start),
                };
                registry.register(area, h.component_idx);
            }
        }
    }

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
