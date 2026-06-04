use super::style::{body_line, block_border, header_line, rounded_bottom, rounded_top};
use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct QualityCard {
    score: Option<f64>,
    complete: bool,
    issues: Vec<String>,
    timestamp: Option<String>,
}

impl QualityCard {
    pub fn new(
        score: Option<f64>,
        complete: bool,
        issues: &[String],
        timestamp: Option<&str>,
    ) -> Self {
        Self {
            score,
            complete,
            issues: issues.to_vec(),
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }
}

impl MessageComponent for QualityCard {
    /// 1 (top) + 1 (header) + 1 (score row) + N (issues) + 1 (bottom)
    fn height(&self, _w: u16) -> u16 {
        let issues_h = self.issues.len() as u16;
        let score_h = 1;
        1 + 1 + score_h + issues_h + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, _selected: bool) {
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
            "回答质量评估",
            "◈",
            theme.accent(),
            theme.accent(),
            self.timestamp.as_deref(),
        ))
        .style(Style::default().bg(interior_bg))
        .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Score + completeness row
        if y < area.y + area.height.saturating_sub(1) {
            let mut parts = String::new();
            if let Some(s) = self.score {
                parts.push_str(&format!("评分: {:.0}%   ", s * 100.0));
            }
            parts.push_str(&format!(
                "完整性: {}",
                if self.complete { "✓ 完整" } else { "✗ 不完整" }
            ));
            Paragraph::new(body_line(
                &parts,
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::BOLD),
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
                Style::default().fg(theme.accent()),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
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
