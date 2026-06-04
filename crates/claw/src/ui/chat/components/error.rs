use super::style::{BLOCK_LEFT_RESERVED, blend, body_line, header_line, rounded_bottom, rounded_top};
use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ErrorBanner {
    text: String,
    timestamp: Option<String>,
}

impl ErrorBanner {
    pub fn new(text: &str, timestamp: Option<&str>) -> Self {
        Self {
            text: text.to_string(),
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }

    fn body_rows(&self, width: u16) -> u16 {
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        utils::wrap_text(&self.text, usable.max(1))
            .len()
            .max(1) as u16
    }
}

impl MessageComponent for ErrorBanner {
    /// 1 (top) + 1 (header) + body + 1 (bottom)
    fn height(&self, width: u16) -> u16 {
        1 + 1 + self.body_rows(width) + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, _selected: bool) {
        let border = theme.error();
        let interior_bg = blend(theme.error_surface(), theme.error(), 0.15);
        let avatar = theme.error();
        let label_color = theme.error();

        let mut y = area.y;

        // Top border
        Paragraph::new(rounded_top(area.width, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Header
        Paragraph::new(header_line("Error", "✗", avatar, label_color, self.timestamp.as_deref()))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Body
        let body_max = (area.y + area.height).saturating_sub(y + 1);
        let rows = self.body_rows(area.width).min(body_max);
        let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        let wrapped = utils::wrap_text(&self.text, usable.max(1));
        let take = wrapped.len().min(rows as usize);
        for (i, line) in wrapped.iter().take(take).enumerate() {
            Paragraph::new(body_line(
                line,
                Style::default().fg(theme.error()).add_modifier(Modifier::BOLD),
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
