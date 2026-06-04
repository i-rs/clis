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
        // Map screen row to a component by y-offset.
        // The render_chat places area.y at the top of the chat viewport.
        // component_offsets are already adjusted for scroll.
        let offsets = &self.app.component_offsets;
        if offsets.is_empty() { return; }

        // Find which component this row hits
        let mut idx = offsets.len();
        for (i, &off) in offsets.iter().enumerate() {
            if row < off + 1 {
                // Headers extend to the end of the viewport width
                idx = i;
                break;
            }
        }
        // Handle click near the last component's header
        if idx >= offsets.len() && !offsets.is_empty() {
            if row < self.app.component_total_height as u16 {
                idx = offsets.len() - 1;
            }
        }
        if idx >= self.app.messages.len() { return; }

        let msg = &self.app.messages[idx];
        match msg {
            crate::app::Message::Assistant { reasoning, .. } if !reasoning.is_empty() => {
                if self.app.overlay.reasoning_expanded.contains(&idx) {
                    self.app.overlay.reasoning_expanded.remove(&idx);
                } else {
                    self.app.overlay.reasoning_expanded.insert(idx);
                }
                self.app.mark_dirty();
            }
            crate::app::Message::ToolCall { .. } => {
                if self.app.overlay.tool_call_expanded.contains(&idx) {
                    self.app.overlay.tool_call_expanded.remove(&idx);
                } else {
                    self.app.overlay.tool_call_expanded.insert(idx);
                }
                self.app.mark_dirty();
            }
            _ => {}
        }
    }
}
