use super::style::{
    BLOCK_LEFT_RESERVED, body_line, block_border, header_line, rounded_bottom, rounded_top,
};
use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct EvaluationInline {
    tool: String,
    valid: bool,
    issues: Vec<String>,
    timestamp: Option<String>,
}

impl EvaluationInline {
    pub fn new(
        tool: &str,
        valid: bool,
        issues: &[String],
        timestamp: Option<&str>,
    ) -> Self {
        Self {
            tool: tool.to_string(),
            valid,
            issues: issues.to_vec(),
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }
}

impl MessageComponent for EvaluationInline {
    /// 1 (top) + 1 (header) + 1 (issues header) + N (issues) + 1 (bottom)
    fn height(&self, _width: u16) -> u16 {
        if self.valid {
            return 0;
        }
        (1 + 1 + 1 + self.issues.len() as u16 + 1).max(1)
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, _selected: bool) {
        if self.valid {
            return;
        }
        let border = block_border(theme, false);
        let interior_bg = theme.surface();
        let mut y = area.y;

        // Top border
        Paragraph::new(rounded_top(area.width, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Header
        Paragraph::new(header_line(
            "工具结果检查",
            "⚠",
            theme.accent(),
            theme.accent(),
            self.timestamp.as_deref(),
        ))
        .style(Style::default().bg(interior_bg))
        .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Tool name sub-header
        if y < area.y + area.height.saturating_sub(1) {
            Paragraph::new(body_line(
                &self.tool,
                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        // Issues
        let max_issue_rows = (area.y + area.height).saturating_sub(y + 1) as usize;
        for (i, issue) in self.issues.iter().take(max_issue_rows).enumerate() {
            Paragraph::new(body_line(
                &format!("•  {}", issue),
                Style::default().fg(theme.text()),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
        }
        if !self.issues.is_empty() {
            y += self.issues.len().min(max_issue_rows) as u16;
        }

        // Bottom border
        if area.height >= 1 {
            let by = area.y + area.height - 1;
            Paragraph::new(rounded_bottom(area.width, border))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y: by, height: 1, ..area }, buf);
        }
    }
}
