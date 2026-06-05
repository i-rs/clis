use crate::app::App;
use crate::ui::chat_api::ComponentOp;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui_interact::events::is_left_click;

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
                _ if is_left_click(&mouse) => self.handle_click(mouse.column.saturating_sub(1), mouse.row.saturating_sub(1)),
                _ => {}
            }
            self.app.mark_overlay_dirty();
        }
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        // The library's registry does the row/col hit-test for us
        // and returns encoded `(component_idx << 4) | op_variant`
        // data, or `None` if the click was on a non-clickable region.
        if let Some(data) = self.app.hit_regions.handle_click(col, row).copied() {
            let idx = data >> 4;
            let op = match data & 0xF {
                1 => ComponentOp::ToggleArgs,
                2 => ComponentOp::ToggleResult,
                _ => ComponentOp::Toggle,
            };
            if self.app.apply_to_component(idx, op) {
                self.app.mark_dirty();
            }
        }
    }
}
