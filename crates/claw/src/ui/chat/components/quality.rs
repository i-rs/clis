use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct QualityCard { score: Option<f64>, complete: bool, issues: Vec<String> }
impl QualityCard {
    pub fn new(score: Option<f64>, complete: bool, issues: &[String]) -> Self {
        Self { score, complete, issues: issues.to_vec() }
    }
}

impl MessageComponent for QualityCard {
    fn height(&self, _w: u16) -> u16 { 2 + self.issues.len() as u16 + if self.score.is_some() { 1 } else { 0 } }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let ac = theme.accent();
        let bg = if selected { ratatui::style::Color::Rgb(40, 40, 25) } else { theme.background() };
        let mut y = area.y;
        let mut parts = vec![Span::raw("   "), Span::styled("📊 ", Style::default().fg(ac)),
                             Span::styled("质量", Style::default().fg(ac).bold())];
        if let Some(s) = self.score {
            parts.push(Span::styled(format!(" {:.0}%", s * 100.0), Style::default().fg(theme.dim_text())));
        }
        parts.push(Span::styled(
            format!(" {}", if self.complete { "✅" } else { "❌" }),
            Style::default().fg(theme.text()),
        ));
        Paragraph::new(Line::from(parts)).style(Style::default().bg(bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;
        for issue in &self.issues {
            Paragraph::new(Line::from(Span::styled(format!("     • {}", issue), Style::default().fg(ac))))
                .style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }
    }
}
