use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ImageCard { path: String, alt_text: String, width: u32, height: u32, format: String }

impl ImageCard {
    pub fn new(path: &str, alt_text: &str, width: u32, height: u32, format: &str) -> Self {
        Self { path: path.to_string(), alt_text: alt_text.to_string(), width, height, format: format.to_string() }
    }
}

impl MessageComponent for ImageCard {
    fn height(&self, _width: u16) -> u16 { 4 }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let bg = if selected { ratatui::style::Color::Rgb(45, 45, 30) } else { theme.background() };
        let mut y = area.y;

        Paragraph::new(Line::from(Span::styled(
            "🖼 图片消息", Style::default().fg(theme.accent()),
        ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        Paragraph::new(Line::from(Span::styled(
            format!("   {} ({}x{}, {})", self.alt_text, self.width, self.height, self.format),
            Style::default().fg(theme.text()),
        ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        Paragraph::new(Line::from(Span::styled(
            format!("   📁 {}", self.path), Style::default().fg(theme.dim_text()),
        ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        Paragraph::new(Line::from("")).style(Style::default().bg(bg))
            .render(Rect { y, height: 1, ..area }, buf);
    }
}
