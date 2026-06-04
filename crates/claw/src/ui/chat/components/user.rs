use super::style::{
    BLOCK_LEFT_RESERVED, blend, body_line, body_padding, block_border, header_line,
    rounded_bottom, rounded_top,
};
use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct UserBubble {
    text: String,
    timestamp: Option<String>,
}

impl UserBubble {
    pub fn new(text: &str, timestamp: Option<&str>) -> Self {
        Self {
            text: text.to_string(),
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }

    /// Number of *body* rows (excludes the top/bottom borders).
    fn body_rows(&self, width: u16) -> u16 {
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        utils::wrap_text(&self.text, usable.max(1))
            .len()
            .max(1) as u16
    }
}

impl MessageComponent for UserBubble {
    /// Total block height: 1 (top) + 1 (header) + body + 1 (bottom).
    fn height(&self, width: u16) -> u16 {
        1 + 1 + self.body_rows(width) + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let border = block_border(theme, selected);
        let interior_bg = if selected {
            blend(theme.user_surface(), theme.primary(), 0.25)
        } else {
            theme.user_surface()
        };
        let avatar = theme.secondary();
        let label = theme.text();

        let mut y = area.y;

        // Top border
        let w = area.width;
        Paragraph::new(rounded_top(w, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Header
        Paragraph::new(header_line(
            "You",
            "▰",
            avatar,
            label,
            self.timestamp.as_deref(),
        ))
        .style(Style::default().bg(interior_bg))
        .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Body
        let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        let text_style = Style::default().fg(theme.text());
        let body_max = (area.y + area.height).saturating_sub(y + 1); // leave room for bottom
        let wrapped = utils::wrap_text(&self.text, usable.max(1));
        let rows = wrapped.len().max(1).min(body_max as usize);
        for (i, line) in wrapped.iter().take(rows).enumerate() {
            let line_y = y + i as u16;
            if i == rows - 1 && rows < wrapped.len() {
                // Show an ellipsis when the last visible row is
                // truncated by height. We still get a clean break
                // before the bottom border.
                Paragraph::new(body_line(&format!("{} …", line), text_style))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y: line_y, height: 1, ..area }, buf);
            } else {
                Paragraph::new(body_line(line, text_style))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y: line_y, height: 1, ..area }, buf);
            }
        }
        if wrapped.is_empty() {
            Paragraph::new(body_padding())
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        } else {
            y += rows as u16;
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
}
