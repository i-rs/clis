use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ErrorBanner { text: String }
impl ErrorBanner { pub fn new(text: &str) -> Self { Self { text: text.to_string() } } }

impl MessageComponent for ErrorBanner {
    fn height(&self, _width: u16) -> u16 { 2 }
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let bg = if selected { ratatui::style::Color::Rgb(60, 25, 25) } else { theme.background() };
        Paragraph::new(Line::from(vec![
            Span::raw("   "), Span::styled("✗ ", Style::default().fg(theme.error())),
            Span::styled(&self.text, Style::default().fg(theme.error()).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().bg(bg)).render(area, buf);
    }
}
