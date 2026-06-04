use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct UserBubble {
    text: String,
}

impl UserBubble {
    pub fn new(text: &str) -> Self { Self { text: text.to_string() } }

    fn wrap_count(text: &str, width: u16) -> usize {
        if width < 4 { return 1; }
        let w = width.saturating_sub(4) as usize;
        text.split('\n').map(|l| {
            let c = l.chars().count();
            if c == 0 { 1 } else { (c + w - 1) / w }
        }).sum::<usize>().max(1)
    }
}

impl MessageComponent for UserBubble {
    fn height(&self, width: u16) -> u16 { (Self::wrap_count(&self.text, width) + 2) as u16 }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let (ac, _) = theme.user_colors();
        let prefix = Span::styled("◆ ", Style::default().fg(ac));
        let body = Span::styled(&self.text, Style::default().bold().fg(theme.text()));
        let p = Paragraph::new(Line::from(vec![prefix, body]))
            .style(if selected { Style::default().bg(ratatui::style::Color::Rgb(35, 55, 35)) }
                   else { Style::default() });
        p.render(area, buf);
    }
}
