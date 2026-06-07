use super::MessageComponent;
use super::style::{
    BLOCK_LEFT_RESERVED, blend, block_border, body_line, body_padding, header_line,
    render_block_chrome,
};
use i_rs_claw_core::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};
use std::cell::{Cell, RefCell};

pub(crate) struct UserBubble {
    text: String,
    timestamp: Option<String>,
    /// Cached `(width, body_row_count)` for `body_rows()`.
    body_rows_cache: Cell<Option<(u16, u16)>>,
    /// Cached wrapped lines keyed by width. User text never mutates
    /// after construction, so the cache is valid for the bubble's
    /// lifetime and shared between `height()` and `render()`.
    wrapped_cache: RefCell<Option<(u16, Vec<String>)>>,
}

impl UserBubble {
    pub fn new(text: &str, timestamp: Option<&str>) -> Self {
        Self {
            text: text.to_string(),
            timestamp: timestamp.map(|s| s.to_string()),
            body_rows_cache: Cell::new(None),
            wrapped_cache: RefCell::new(None),
        }
    }

    /// Number of *body* rows (excludes the top/bottom borders).
    fn body_rows(&self, width: u16) -> u16 {
        if let Some((cached_w, cached_h)) = self.body_rows_cache.get()
            && cached_w == width
        {
            return cached_h;
        }
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        let h = utils::wrap_text(&self.text, usable.max(1)).len().max(1) as u16;
        self.body_rows_cache.set(Some((width, h)));
        h
    }

    /// Return the wrapped lines for `width`, populating the cache on
    /// the first call. The cache is held by `RefCell` so the trait's
    /// `&self`-taking `render` can populate it.
    fn wrapped_lines(&self, width: u16) -> std::cell::Ref<'_, Vec<String>> {
        let cache_miss = match self.wrapped_cache.borrow().as_ref() {
            Some((cached_w, _)) => *cached_w != width,
            None => true,
        };
        if cache_miss {
            let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
            let wrapped = utils::wrap_text(&self.text, usable.max(1));
            *self.wrapped_cache.borrow_mut() = Some((width, wrapped));
        }
        std::cell::Ref::map(self.wrapped_cache.borrow(), |opt| {
            opt.as_ref()
                .map(|(_, v)| v)
                .expect("cache was just populated")
        })
    }
}

impl MessageComponent for UserBubble {
    /// Total block height: 1 (top) + 1 (header) + body + 1 (bottom).
    fn height(&self, width: u16) -> u16 {
        1 + 1 + self.body_rows(width) + 1
    }

    fn clickable(&self) -> bool {
        true
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
        let header = header_line("You", "▰", avatar, label, self.timestamp.as_deref());
        let body = render_block_chrome(area, buf, border, interior_bg, header);

        // Body
        let text_style = Style::default().fg(theme.text());
        let wrapped = self.wrapped_lines(area.width);
        let rows = wrapped.len().max(1).min(body.height() as usize);
        for (i, line) in wrapped.iter().take(rows).enumerate() {
            let line_y = body.top + i as u16;
            if i == rows - 1 && rows < wrapped.len() {
                // Show an ellipsis when the last visible row is
                // truncated by height. We still get a clean break
                // before the bottom border.
                Paragraph::new(body_line(&format!("{} …", line), text_style))
                    .style(Style::default().bg(interior_bg))
                    .render(
                        Rect {
                            y: line_y,
                            height: 1,
                            ..area
                        },
                        buf,
                    );
            } else {
                Paragraph::new(body_line(line, text_style))
                    .style(Style::default().bg(interior_bg))
                    .render(
                        Rect {
                            y: line_y,
                            height: 1,
                            ..area
                        },
                        buf,
                    );
            }
        }
        if wrapped.is_empty() {
            Paragraph::new(body_padding())
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
