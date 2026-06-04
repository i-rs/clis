use crate::app::App;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub struct MouseEventHandler<'a> {
    pub app: &'a mut App,
}

impl<'a> MouseEventHandler<'a> {
    pub fn new(app: &'a mut App) -> Self { Self { app } }

    pub fn handle(&mut self, mouse: MouseEvent) {
        if self.app.overlay.sidebar_body_idx.is_some() {
            match mouse.kind {
                MouseEventKind::ScrollDown => {
                    if let Some(idx) = self.app.overlay.sidebar_body_idx
                        && let Some(log) = self.app.http_logs.get(idx)
                    {
                        let max = log.request_body.lines().count() * 3;
                        self.app.overlay.sidebar_body_scroll =
                            (self.app.overlay.sidebar_body_scroll + 3).min(max);
                    }
                }
                MouseEventKind::ScrollUp => {
                    self.app.overlay.sidebar_body_scroll =
                        self.app.overlay.sidebar_body_scroll.saturating_sub(3);
                }
                _ => {}
            }
        } else if !self.app.is_processing()
            && self.app.overlay.current.is_none()
        {
            match mouse.kind {
                MouseEventKind::ScrollDown => self.app.scroll_down(),
                MouseEventKind::ScrollUp => self.app.scroll_up(),
                MouseEventKind::Down(MouseButton::Left) => self.handle_click(mouse.column, mouse.row),
                _ => {}
            }
            self.app.mark_overlay_dirty();
        }
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        // The click dispatch is intentionally side-effect free until
        // we know we've landed on a clickable component. We do this
        // with the hit regions that `render_chat` last produced:
        // they are in screen-absolute coordinates and carry a
        // component index, so there is no need to recompute offsets
        // or walk the message vector here.
        if let Some(idx) = self
            .app
            .hit_regions
            .iter()
            .find(|h| row >= h.y_start && row < h.y_end && col >= h.x_start && col < h.x_end)
            .map(|h| h.component_idx)
        {
            self.app.toggle_component_at(idx);
        }
        // `col` is currently unused beyond the hit-test bounding
        // box, but keep the parameter so the caller signature stays
        // stable if we add column-aware targets later.
        let _ = col;
    }
}
