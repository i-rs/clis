use super::style::{body_line, block_border, header_line, render_block_chrome};
use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct FeedbackRow {
    positive: bool,
    message: Option<String>,
    timestamp: Option<String>,
}

impl FeedbackRow {
    pub fn new(positive: bool, message: Option<&str>, timestamp: Option<&str>) -> Self {
        Self {
            positive,
            message: message.map(|s| s.to_string()),
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }
}

impl MessageComponent for FeedbackRow {
    /// 1 (top) + 1 (header) + (1 if has message) + 1 (bottom)
    fn height(&self, _w: u16) -> u16 {
        let has_msg = self.message.as_ref().map_or(false, |m| !m.is_empty());
        1 + 1 + (has_msg as u16) + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, _selected: bool) {
        let border = block_border(theme, false);
        let interior_bg = theme.surface();
        let glyph = if self.positive { "👍" } else { "👎" };
        let accent_color = if self.positive {
            theme.secondary()
        } else {
            theme.error()
        };
        let label = if self.positive { "好评" } else { "差评" };
        let header = header_line(
            label,
            glyph,
            accent_color,
            accent_color,
            self.timestamp.as_deref(),
        );
        let body = render_block_chrome(area, buf, border, interior_bg, header);

        // Optional message body.
        if let Some(m) = &self.message
            && !m.is_empty()
            && body.height() >= 1
        {
            Paragraph::new(body_line(
                m,
                Style::default().fg(theme.dim_text()).add_modifier(Modifier::ITALIC),
            ))
            .style(Style::default().bg(interior_bg))
            .render(
                Rect {
                    y: body.top,
                    height: 1,
                    ..area
                },
                buf,
            );
        }
    }
}
