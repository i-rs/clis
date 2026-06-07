use super::MessageComponent;
use super::style::{
    BLOCK_LEFT_RESERVED, block_border, body_line, header_line, render_block_chrome,
};
use i_rs_claw_core::theme::Theme;
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
        let header = header_line(
            "生成图片",
            "◐",
            theme.accent(),
            theme.accent(),
            self.timestamp.as_deref(),
        );
        let body = render_block_chrome(area, buf, border, interior_bg, header);

        let mut y = body.top;

        // Prompt
        if body.contains(y) {
            Paragraph::new(body_line(
                &self.prompt,
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::ITALIC),
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

        // Image area placeholder (we just shade the cells)
        let shade_w =
            self.width_cells
                .min(area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16)) as usize;
        let shade = theme.dim_text();
        for _ in 0..self.height_cells {
            if !body.contains(y) {
                break;
            }
            Paragraph::new(body_line(&"▒".repeat(shade_w), Style::default().fg(shade)))
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
