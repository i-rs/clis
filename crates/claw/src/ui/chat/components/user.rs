use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct UserBubble { text: String }

impl UserBubble {
    pub fn new(text: &str) -> Self { Self { text: text.to_string() } }

    fn wrap_count(text: &str, width: u16) -> u16 {
        let w = width.saturating_sub(8) as usize;
        text.split('\n').map(|l| {
            let c = l.chars().count();
            if c == 0 { 1 } else { (c + w - 1) / w }
        }).sum::<usize>().max(1) as u16
    }
}

impl MessageComponent for UserBubble {
    fn height(&self, width: u16) -> u16 { Self::wrap_count(&self.text, width) + 2 }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let (ac, _) = theme.user_colors();
        let bg = Color::Rgb(16, 28, 22); // subtle green-tinted background
        let s_bg = if selected { Color::Rgb(30, 50, 35) } else { bg };
        let indent = "   ";

        Paragraph::new(Line::from(vec![
            Span::raw(indent),
            Span::styled("◆", Style::default().fg(ac).bold()),
            Span::raw("  "),
            Span::styled(&self.text, Style::default().fg(theme.text())),
        ])).style(Style::default().bg(s_bg))
          .render(area, buf);
    }
}
