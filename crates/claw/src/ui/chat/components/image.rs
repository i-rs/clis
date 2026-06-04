use super::style::{BLOCK_LEFT_RESERVED, body_line, block_border, header_line, rounded_bottom, rounded_top};
use super::MessageComponent;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ImageCard {
    prompt: String,
    width_cells: u16,
    height_cells: u16,
    timestamp: Option<String>,
}

impl ImageCard {
    pub fn new(prompt: &str, width_cells: u16, height_cells: u16, timestamp: Option<&str>) -> Self {
        Self {
            prompt: prompt.to_string(),
            width_cells,
            height_cells,
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }
}

impl MessageComponent for ImageCard {
    /// 1 (top) + 1 (header) + 1 (prompt) + image_rows + 1 (bottom)
    fn height(&self, _w: u16) -> u16 {
        1 + 1 + 1 + self.height_cells + 1
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
            "生成图片",
            "◐",
            theme.accent(),
            theme.accent(),
            self.timestamp.as_deref(),
        ))
        .style(Style::default().bg(interior_bg))
        .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Prompt
        if y < area.y + area.height.saturating_sub(1) {
            Paragraph::new(body_line(
                &self.prompt,
                Style::default().fg(theme.text()).add_modifier(Modifier::ITALIC),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        }

        // Image area placeholder (we just shade the cells)
        let max_image_rows = (area.y + area.height).saturating_sub(y + 1);
        let take = self.height_cells.min(max_image_rows);
        let shade = theme.dim_text();
        for r in 0..take {
            let shade_w = self
                .width_cells
                .min(area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16))
                as usize;
            Paragraph::new(body_line(
                &"▒".repeat(shade_w),
                Style::default().fg(shade),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y: y + r, height: 1, ..area }, buf);
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
