use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct FeedbackRow { positive: bool, message: Option<String> }

impl FeedbackRow {
    pub fn new(positive: bool, message: Option<&str>) -> Self {
        Self { positive, message: message.map(|s| s.to_string()) }
    }
}

impl MessageComponent for FeedbackRow {
    fn height(&self, _width: u16) -> u16 {
        1 + if self.message.as_ref().map_or(false, |m| !m.is_empty()) { 1 } else { 0 } + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let icon = if self.positive { "👍" } else { "👎" };
        let bg = if selected { ratatui::style::Color::Rgb(45, 45, 30) } else { theme.background() };
        let mut y = area.y;

        Paragraph::new(Line::from(Span::styled(
            format!("{} 反馈已记录", icon),
            Style::default().fg(theme.accent()),
        ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        if let Some(ref msg) = self.message {
            if !msg.is_empty() {
                Paragraph::new(Line::from(Span::styled(msg, Style::default().fg(theme.dim_text()))))
                    .style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }
        }

        Paragraph::new(Line::from("")).style(Style::default().bg(bg))
            .render(Rect { y, height: 1, ..area }, buf);
    }
}
