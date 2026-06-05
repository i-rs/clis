//! Scrollable viewport over a list of components.
//!
//! Pre-computes virtual Y offsets for each component from their cached
//! heights, then renders only the visible set each frame.  Provides
//! O(log n) mapping from screen coordinates back to component index
//! for click dispatch.

use super::{ComponentCell, ComponentOp};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// A viewport over a slice of `ComponentCell` handles.
pub struct Scroller {
    /// Per-component `(start_y, end_y)` in virtual space.
    offsets: Vec<(usize, usize)>,
    /// Total virtual height of all components (no gaps).
    total_height: usize,
    /// Viewport width used for offset computation.
    #[allow(dead_code)]
    width: u16,
}

impl Scroller {
    /// Create an empty scroller (no components).
    pub const fn new_empty() -> Self {
        Self { offsets: vec![], total_height: 0, width: 0 }
    }

    /// Number of components in this scroller.
    pub fn component_count(&self) -> usize {
        self.offsets.len()
    }

    /// Compute virtual offsets for the given components.
    ///
    /// Call this whenever `msg_gen` changes or the terminal
    /// width changes.  It calls `height(width)` on each component,
    /// which is cached, so this is O(n) with cheap per-item work.
    pub fn compute(components: &[ComponentCell], width: u16) -> Self {
        let mut offsets = Vec::with_capacity(components.len());
        let mut y: usize = 0;
        for cell in components {
            let h = cell.borrow().height(width) as usize;
            offsets.push((y, y + h));
            y += h;
        }
        let total_height = y;
        Self { offsets, total_height, width }
    }

    /// Recompute offsets — convenience when width changed.
    #[allow(dead_code)]
    pub fn recompute(&mut self, components: &[ComponentCell], width: u16) {
        *self = Self::compute(components, width);
    }

    /// Total virtual height.
    pub fn total_height(&self) -> usize { self.total_height }

    /// Return the number of rows that can be scrolled past
    /// (total - viewport) so scroll_offset fits in `[0, max_scroll]`.
    pub fn max_scroll(&self, viewport_height: u16) -> usize {
        self.total_height.saturating_sub(viewport_height as usize)
    }

    /// Find the component index containing `virtual_y`.
    ///
    /// Returns `(component_idx, local_y_within_component)`.
    pub fn component_at_virtual_y(&self, virtual_y: usize) -> Option<(usize, u16)> {
        if self.offsets.is_empty() { return None; }
        // Binary search for the first offset with end_y > virtual_y
        let idx = self.offsets.partition_point(|&(_, end)| end <= virtual_y);
        self.offsets.get(idx).map(|&(start, end)| {
            let local_y = virtual_y.saturating_sub(start) as u16;
            (idx, local_y.min((end - start) as u16))
        })
    }

    /// Check if `virtual_y` falls within a click target of a component.
    /// Returns the `ComponentOp` to apply, if any.
    #[allow(dead_code)]
    pub fn click_target_at(&self, virtual_y: usize, components: &[ComponentCell]) -> Option<ComponentOp> {
        let (idx, local_y) = self.component_at_virtual_y(virtual_y)?;
        let comp = components[idx].borrow();

        // First check if the click falls in extra_click_targets
        for (row_off, height, op) in comp.extra_click_targets(self.width).into_iter() {
            if local_y >= row_off && local_y < row_off + height {
                return Some(op);
            }
        }

        // Fall back to clickable() toggle on the whole component
        if comp.clickable() {
            return Some(ComponentOp::Toggle);
        }

        None
    }

    /// Render only the visible portion of components into the buffer.
    ///
    /// Writes into `area` starting from `scroll_offset` virtual rows up.
    pub fn render(
        &self,
        components: &[ComponentCell],
        scroll_offset: usize,
        area: Rect,
        buf: &mut Buffer,
        selected_idx: Option<usize>,
    ) {
        if area.width == 0 || area.height == 0 { return; }
        let viewport_end = scroll_offset + area.height as usize;

        // Find first visible component
        let first = self.offsets.partition_point(|&(_, end)| end <= scroll_offset);
        if first >= self.offsets.len() { return; }

        let mut screen_y = area.y;
        let screen_bottom = area.y + area.height;

        for (idx, _) in components.iter().enumerate().take(self.offsets.len()).skip(first) {
            let (start_y, end_y) = self.offsets[idx];
            if start_y >= viewport_end { break; }
            if screen_y >= screen_bottom { break; }

            let comp_height = (end_y - start_y) as u16;
            let scrolled_off = scroll_offset.saturating_sub(start_y) as u16;
            let available = screen_bottom.saturating_sub(screen_y);
            let visible_h = comp_height.saturating_sub(scrolled_off).min(available);

            if visible_h == 0 { continue; }

            let comp_area = Rect {
                x: area.x,
                y: screen_y,
                width: area.width,
                height: visible_h,
            };

            let comp = components[idx].borrow();
            let is_selected = selected_idx == Some(idx);
            comp.render(comp_area, buf, scrolled_off, is_selected);

            screen_y += visible_h;
        }
    }

    /// Convenience: find which component a screen row maps to,
    /// accounting for scroll offset and hint offset.
    #[allow(dead_code)]
    pub fn component_from_screen(
        &self,
        screen_row: u16,
        scroll_offset: usize,
        hint_shown: bool,
        chat_area_y: u16,
    ) -> Option<(usize, u16)> {
        let hint_off = if hint_shown { 1 } else { 0 };
        let content_row = (screen_row as usize).saturating_sub(chat_area_y as usize + hint_off);
        let virtual_y = content_row + scroll_offset;
        self.component_at_virtual_y(virtual_y)
    }
}
