use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ErrorBanner {
    text: String,
}

impl ErrorBanner {
    pub fn new(text: &str) -> Self { Self { text: text.to_string() } }
}

impl MessageComponent for ErrorBanner {
    fn height(&self, width: u16) -> u16 {
        let w = width.saturating_sub(4) as usize;
        let lines = self.text.split('\n').map(|l| {
            let c = l.chars().count();
            if c == 0 { 1 } else { (c + w - 1) / w }
        }).sum::<usize>().max(1);
        (1 + lines + 1) as u16
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let err = theme.error();
        let bg = if selected { ratatui::style::Color::Rgb(70, 30, 30) } else { theme.background() };
        Paragraph::new(Line::from(vec![
            Span::styled("✗ ", Style::default().fg(err)),
            Span::styled(&self.text, Style::default().fg(err).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().bg(bg))
          .render(area, buf);
    }
}
