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
    fn height(&self, _width: u16) -> u16 {
        2 + self.issues.len() as u16 + if self.score.is_some() { 1 } else { 0 } + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let ac = theme.accent();
        let bg = if selected { ratatui::style::Color::Rgb(45, 45, 30) } else { theme.background() };
        let mut y = area.y;

        Paragraph::new(Line::from(vec![
            Span::styled("📊 ", Style::default().fg(ac)),
            Span::styled("回答质量评估", Style::default().fg(ac).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        if let Some(s) = self.score {
            Paragraph::new(Line::from(Span::styled(
                format!("   评分: {:.0}%", s * 100.0), Style::default().fg(theme.text()),
            ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        Paragraph::new(Line::from(Span::styled(
            format!("   完整性: {}", if self.complete { "✅" } else { "❌" }),
            Style::default().fg(theme.text()),
        ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        for issue in &self.issues {
            Paragraph::new(Line::from(Span::styled(
                format!("   • {}", issue), Style::default().fg(ac),
            ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        Paragraph::new(Line::from("")).style(Style::default().bg(bg))
            .render(Rect { y, height: 1, ..area }, buf);
    }
}
