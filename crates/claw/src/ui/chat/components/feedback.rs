use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct FeedbackRow { positive: bool, message: Option<String> }
impl FeedbackRow {
    pub fn new(positive: bool, message: Option<&str>) -> Self {
        Self { positive, message: message.map(|s| s.to_string()) }
    }
}

impl MessageComponent for FeedbackRow {
    fn height(&self, _w: u16) -> u16 { 1 + if self.message.as_ref().map_or(false, |m| !m.is_empty()) { 1 } else { 0 } }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, _selected: bool) {
        let icon = if self.positive { "👍" } else { "👎" };
        let mut y = area.y;
        let mut parts = vec![Span::raw("   "), Span::styled(format!("{} 已记录", icon), Style::default().fg(theme.accent()))];
        if let Some(ref m) = self.message { if !m.is_empty() { parts.push(Span::styled(format!(" — {}", m), Style::default().fg(theme.dim_text()))); } }
        Paragraph::new(Line::from(parts)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;
    }
}
