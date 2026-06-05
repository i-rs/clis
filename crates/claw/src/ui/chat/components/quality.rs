use super::MessageComponent;
use super::style::{block_border, body_line, header_line, render_block_chrome};
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
        let header = header_line(
            "回答质量评估",
            "◈",
            theme.accent(),
            theme.accent(),
            self.timestamp.as_deref(),
        );
        let body = render_block_chrome(area, buf, border, interior_bg, header);

        let mut y = body.top;

        // Score + completeness row
        if body.contains(y) {
            let mut parts = String::new();
            if let Some(s) = self.score {
                parts.push_str(&format!("评分: {:.0}%   ", s * 100.0));
            }
            parts.push_str(&format!(
                "完整性: {}",
                if self.complete {
                    "✓ 完整"
                } else {
                    "✗ 不完整"
                }
            ));
            Paragraph::new(body_line(
                &parts,
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(interior_bg))
            .render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
            y += 1;
        }

        // Issues
        for issue in self.issues.iter() {
            if !body.contains(y) {
                break;
            }
            Paragraph::new(body_line(
                &format!("•  {}", issue),
                Style::default().fg(theme.accent()),
            ))
            .style(Style::default().bg(interior_bg))
            .render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
            y += 1;
        }
    }
}
