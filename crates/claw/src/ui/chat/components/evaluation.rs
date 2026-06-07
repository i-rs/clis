use super::MessageComponent;
use super::style::{block_border, body_line, header_line, render_block_chrome};
use i_rs_claw_core::theme::Theme;
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
    pub fn new(tool: &str, valid: bool, issues: &[String], timestamp: Option<&str>) -> Self {
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
        let header = header_line(
            "工具结果检查",
            "⚠",
            theme.accent(),
            theme.accent(),
            self.timestamp.as_deref(),
        );
        let body = render_block_chrome(area, buf, border, interior_bg, header);

        let mut y = body.top;

        // Tool name sub-header
        if body.contains(y) {
            Paragraph::new(body_line(
                &self.tool,
                Style::default()
                    .fg(theme.accent())
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
                Style::default().fg(theme.text()),
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
