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
        let offsets = &self.app.component_offsets;
        if offsets.is_empty() { return; }

        // Convert absolute screen row to content-relative row
        let content_row = row.saturating_sub(self.app.chat_y + 1); // +1 for border

        // Find the component that contains this row
        let mut idx = offsets.len();
        for (i, &off) in offsets.iter().enumerate() {
            let next = if i + 1 < offsets.len() { offsets[i + 1] } else { u16::MAX };
            if content_row >= off && content_row < next {
                idx = i;
                break;
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
