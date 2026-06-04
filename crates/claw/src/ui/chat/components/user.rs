use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct UserBubble { text: String }

impl UserBubble {
    pub fn new(text: &str) -> Self { Self { text: text.to_string() } }

    fn wrap_count(text: &str, width: u16) -> u16 {
        let w = width.saturating_sub(6) as usize;
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
        let spacer = "   ";
        Paragraph::new(Line::from(vec![
            Span::raw(spacer),
            Span::styled("◆ ", Style::default().fg(ac).bold()),
            Span::styled(&self.text, Style::default().fg(theme.text())),
        ])).style(if selected { Style::default().bg(ratatui::style::Color::Rgb(30, 45, 30)) }
                 else { Style::default() })
          .render(area, buf);
    }
}
