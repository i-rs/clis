use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct EvaluationInline {
    tool: String,
    valid: bool,
    issues: Vec<String>,
}

impl EvaluationInline {
    pub fn new(tool: &str, valid: bool, issues: &[String]) -> Self {
        Self { tool: tool.to_string(), valid, issues: issues.to_vec() }
    }
}

impl MessageComponent for EvaluationInline {
    fn height(&self, _width: u16) -> u16 {
        if self.valid { 0 } else { (2 + self.issues.len()) as u16 }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        if self.valid { return; }
        let bg = if selected { ratatui::style::Color::Rgb(75, 40, 25) } else { theme.background() };
        let mut y = area.y;

        Paragraph::new(Line::from(Span::styled(
            format!("⚠ 工具结果检查: {}", self.tool),
            Style::default().fg(theme.error()).add_modifier(Modifier::BOLD),
        ))).style(Style::default().bg(bg))
          .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        for issue in &self.issues {
            Paragraph::new(Line::from(Span::styled(
                format!("   • {}", issue),
                Style::default().fg(theme.accent()).bg(bg),
            ))).render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        Paragraph::new(Line::from(""))
            .style(Style::default().bg(bg))
            .render(Rect { y, height: 1, ..area }, buf);
    }
}
