use super::style::{
    BLOCK_LEFT_RESERVED, blend, body_line, block_border, header_line, rounded_bottom,
    rounded_top,
};
use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::chat::markdown::render_markdown;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct AssistantBlock {
    text: String,
    reasoning: String,
    pub reasoning_expanded: bool,
    timestamp: Option<String>,
}

impl AssistantBlock {
    pub fn new(
        text: &str,
        reasoning: &str,
        reasoning_expanded: bool,
        timestamp: Option<&str>,
    ) -> Self {
        Self {
            text: text.to_string(),
            reasoning: reasoning.to_string(),
            reasoning_expanded,
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }

    /// Number of *body* rows (excludes the top/bottom borders).
    fn body_rows(&self, width: u16) -> u16 {
        if self.text.is_empty() {
            // An assistant block always shows *something* in the body
            // slot. When there's no content yet, show a placeholder.
            return 1;
        }
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        if !is_markdowny(&self.text) {
            return utils::wrap_text(&self.text, usable.max(1))
                .len()
                .max(1) as u16;
        }
        // Use a default theme for line counting only — the actual
        // render uses the real theme and the result has the same
        // number of lines.
        let theme = Theme::from_preset("midnight").unwrap_or_default();
        let md = render_markdown(&self.text, usable, &theme);
        md.len().max(1) as u16
    }

    fn reasoning_rows(&self) -> u16 {
        if self.reasoning.is_empty() || !self.reasoning_expanded {
            return 0;
        }
        self.reasoning.lines().filter(|l| !l.trim().is_empty()).count() as u16
    }
}

fn is_markdowny(text: &str) -> bool {
    text.contains("**")
        || text.contains("__")
        || text.contains("`")
        || text.lines().any(|l| {
            l.starts_with("# ")
                || l.starts_with("## ")
                || l.starts_with("### ")
                || l.starts_with("- ")
                || l.starts_with("* ")
        })
}

impl MessageComponent for AssistantBlock {
    /// 1 (top) + 1 (header) + body + reasoning toggle + reasoning rows + 1 (bottom)
    fn height(&self, width: u16) -> u16 {
        let mut h = 1 + 1 + self.body_rows(width) + 1;
        if !self.reasoning.is_empty() {
            h += 1; // toggle row
            h += self.reasoning_rows();
        }
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let border = block_border(theme, selected);
        let interior_bg = if selected {
            blend(theme.assistant_surface(), theme.primary(), 0.25)
        } else {
            theme.assistant_surface()
        };
        let avatar = theme.primary();
        let label = theme.text();

        let mut y = area.y;

        // Top border
        Paragraph::new(rounded_top(area.width, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Header
        Paragraph::new(header_line("Claw", "◆", avatar, label, self.timestamp.as_deref()))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Body
        let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        let body_h = self.body_rows(area.width);
        let body_max = (area.y + area.height).saturating_sub(y + 1); // leave room for bottom

        if self.text.is_empty() {
            Paragraph::new(body_line(
                "...",
                Style::default().fg(theme.dim_text()).add_modifier(Modifier::ITALIC),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        } else if is_markdowny(&self.text) {
            let md = render_markdown(&self.text, usable, theme);
            let take = (md.len() as u16).min(body_max).min(body_h);
            for (i, ml) in md.iter().take(take as usize).enumerate() {
                let mut spans: Vec<Span<'static>> = Vec::with_capacity(ml.spans.len() + 1);
                spans.push(Span::raw(" ".repeat(BLOCK_LEFT_RESERVED)));
                for s in &ml.spans {
                    spans.push(Span::styled(s.content.clone(), s.style.fg(theme.text())));
                }
                Paragraph::new(Line::from(spans))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
            }
            y += take;
        } else {
            let wrapped = utils::wrap_text(&self.text, usable.max(1));
            let rows = wrapped.len().max(1).min(body_max as usize);
            for (i, line) in wrapped.iter().take(rows).enumerate() {
                Paragraph::new(body_line(line, Style::default().fg(theme.text())))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
            }
            y += rows as u16;
        }

        // Reasoning toggle + body
        if !self.reasoning.is_empty() {
            let chevron = if self.reasoning_expanded { "▾" } else { "▸" };
            let chars = self.reasoning.chars().count();
            let chars_label = if chars >= 1000 {
                format!("{}k chars", chars / 1000)
            } else {
                format!("{} chars", chars)
            };
            let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(6);
            spans.push(Span::raw(" ".repeat(BLOCK_LEFT_RESERVED)));
            spans.push(Span::styled(
                "🧠".to_string(),
                Style::default().fg(theme.accent()),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                "思考过程".to_string(),
                Style::default()
                    .fg(theme.dim_text())
                    .add_modifier(Modifier::BOLD),
            ));
            // Right-align-ish: push the count + chevron but stop if it
            // would overflow the row.
            let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
            let tail = format!("  {}  {}", chars_label, chevron);
            if used + tail.chars().count() <= usable {
                spans.push(Span::styled(
                    chars_label,
                    Style::default().fg(theme.dim_text()),
                ));
                spans.push(Span::styled(
                    format!("  {}", chevron),
                    Style::default()
                        .fg(theme.dim_text())
                        .add_modifier(Modifier::BOLD),
                ));
            } else if used + 2 + 1 <= usable {
                spans.push(Span::styled(
                    format!("  {}", chevron),
                    Style::default()
                        .fg(theme.dim_text())
                        .add_modifier(Modifier::BOLD),
                ));
            }
            if y < area.y + area.height.saturating_sub(1) {
                Paragraph::new(Line::from(spans))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }

            if self.reasoning_expanded {
                for rl in self
                    .reasoning
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                {
                    if y >= area.y + area.height.saturating_sub(1) {
                        break;
                    }
                    let truncated = utils::truncate_str(rl, usable.saturating_sub(1).max(8));
                    Paragraph::new(body_line(
                        &truncated,
                        Style::default()
                            .fg(theme.dim_text())
                            .add_modifier(Modifier::ITALIC),
                    ))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
            }
        }

        // Bottom border — drawn on top of the surface background so
        // the rounded corner reads as a clean break, not a "bump".
        if area.height >= 1 {
            let by = area.y + area.height - 1;
            Paragraph::new(rounded_bottom(area.width, border))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y: by, height: 1, ..area }, buf);
        }
    }

    fn clickable(&self) -> bool {
        !self.reasoning.is_empty()
    }
}
