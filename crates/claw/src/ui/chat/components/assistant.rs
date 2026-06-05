use super::style::{
    BLOCK_LEFT_RESERVED, blend, body_line, block_border, header_line, render_block_chrome,
};
use super::{ComponentOp, MessageComponent};
use crate::llm::TokenUsage;
use crate::theme::Theme;
use crate::ui::chat::markdown::render_markdown;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use std::cell::{Cell, RefCell};

/// Cached body render output for one `(text, width)` pair. The cache
/// is invalidated by `AppendText`. Two variants because the markdown
/// and plain-text paths produce different output types.
enum BodyRenderCache {
    /// Markdown-rendered lines, keyed by `(width, theme_id)`. Theme
    /// is folded in because markdown styles are theme-dependent.
    Md { width: u16, theme_id: u64, lines: Vec<Line<'static>> },
    /// Plain wrapped lines (no theme).
    Plain { width: u16, lines: Vec<String> },
}

pub(crate) struct AssistantBlock {
    text: String,
    reasoning: String,
    pub reasoning_expanded: bool,
    timestamp: Option<String>,
    token_usage: Option<TokenUsage>,
    /// Cached `(width, body_row_count)` for `body_rows()`. The count
    /// depends only on the text and the wrapping width, so we
    /// invalidate it whenever `text` mutates. Interior mutability
    /// lets `height(&self)` populate the cache without changing the
    /// trait signature.
    body_rows_cache: Cell<Option<(u16, u16)>>,
    /// Cached render output for the body. Hit by `render()` on every
    /// frame after the first one (or after `AppendText`/width change
    /// invalidates it). Cuts the render path's markdown parse + line
    /// construction to zero on stable frames.
    body_render_cache: RefCell<Option<BodyRenderCache>>,
    /// Cached `is_markdowny(&text)` result. Scanning a long message
    /// is O(n); we only need to recompute after the text mutates.
    is_markdowny_cache: Cell<Option<bool>>,
}

impl AssistantBlock {
    pub fn new(
        text: &str,
        reasoning: &str,
        reasoning_expanded: bool,
        timestamp: Option<&str>,
        token_usage: Option<TokenUsage>,
    ) -> Self {
        Self {
            text: text.to_string(),
            reasoning: reasoning.to_string(),
            reasoning_expanded,
            timestamp: timestamp.map(|s| s.to_string()),
            token_usage,
            body_rows_cache: Cell::new(None),
            body_render_cache: RefCell::new(None),
            is_markdowny_cache: Cell::new(None),
        }
    }

    /// True if the body looks like markdown. Cached because the
    /// scan walks the text and is called from both `body_rows` and
    /// `render`.
    fn is_markdowny_cached(&self) -> bool {
        if let Some(v) = self.is_markdowny_cache.get() {
            return v;
        }
        let v = is_markdowny(&self.text);
        self.is_markdowny_cache.set(Some(v));
        v
    }

    /// Number of *body* rows (excludes the top/bottom borders).
    fn body_rows(&self, width: u16) -> u16 {
        if let Some((cached_w, cached_h)) = self.body_rows_cache.get()
            && cached_w == width
        {
            return cached_h;
        }
        let h = self.compute_body_rows(width);
        self.body_rows_cache.set(Some((width, h)));
        h
    }

    fn compute_body_rows(&self, width: u16) -> u16 {
        if self.text.is_empty() {
            // An assistant block always shows *something* in the body
            // slot. When there's no content yet, show a placeholder.
            return 1;
        }
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        if !self.is_markdowny_cached() {
            return utils::wrap_text(&self.text, usable.max(1))
                .len()
                .max(1) as u16;
        }
        // Count markdown lines without touching the disk-backed theme
        // preset — line count depends only on text + width, not on
        // styles. Use a cheap default theme for the parser.
        let theme = Theme::default();
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

    fn clickable(&self) -> bool { true }

    fn apply(&mut self, op: ComponentOp) {
        match op {
            ComponentOp::AppendText(delta) => {
                self.text.push_str(&delta);
                self.body_rows_cache.set(None);
                self.is_markdowny_cache.set(None);
                *self.body_render_cache.borrow_mut() = None;
            }
            ComponentOp::AppendReasoning(delta) => self.reasoning.push_str(&delta),
            ComponentOp::Toggle => self.reasoning_expanded = !self.reasoning_expanded,
            // Sub-toggles only make sense on ToolCallCard; assistant
            // has no Args/Result sub-sections. Silently ignore.
            ComponentOp::ToggleArgs | ComponentOp::ToggleResult => {}
        }
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
        let meta = match (self.timestamp.as_deref(), self.token_usage) {
            (Some(ts), Some(usage)) => {
                let tu = if usage.total_tokens >= 1000 {
                    format!("{}K", usage.total_tokens / 1000)
                } else {
                    format!("{}tok", usage.total_tokens)
                };
                Some(format!("{} · {}", ts, tu))
            }
            (Some(ts), None) => Some(ts.to_string()),
            (None, Some(usage)) => {
                let tu = if usage.total_tokens >= 1000 {
                    format!("{}Ktok", usage.total_tokens / 1000)
                } else {
                    format!("{}tok", usage.total_tokens)
                };
                Some(tu)
            }
            (None, None) => None,
        };
        let header = header_line("Claw", "◆", avatar, label, meta.as_deref());
        let body = render_block_chrome(area, buf, border, interior_bg, header);

        let mut y = body.top;

        // Body
        let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        let body_h = self.body_rows(area.width);
        let body_max = body.height();

        if self.text.is_empty() {
            Paragraph::new(body_line(
                "...",
                Style::default().fg(theme.dim_text()).add_modifier(Modifier::ITALIC),
            ))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
            y += 1;
        } else if self.is_markdowny_cached() {
            // Cache the markdown rendering keyed by `(width, theme_id)`.
            // On cache hit we skip the pulldown-cmark parse entirely —
            // a big win on stable frames where 9 of 10 visible
            // messages are unchanged from the previous frame.
            let theme_id = theme.id();
            let cache_miss = match self.body_render_cache.borrow().as_ref() {
                Some(BodyRenderCache::Md { width, theme_id: tid, .. }) => {
                    *width != area.width || *tid != theme_id
                }
                _ => true,
            };
            if cache_miss {
                let md = render_markdown(&self.text, usable, theme);
                *self.body_render_cache.borrow_mut() = Some(BodyRenderCache::Md {
                    width: area.width,
                    theme_id,
                    lines: md,
                });
            }
            // Hold a single borrow for the duration of the render loop.
            let cache_ref = self.body_render_cache.borrow();
            let md: &[Line<'static>] = match cache_ref.as_ref() {
                Some(BodyRenderCache::Md { lines, .. }) => lines,
                _ => unreachable!("cache was just populated"),
            };
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
            // Plain text: cache wrapped lines keyed by width.
            let cache_miss = match self.body_render_cache.borrow().as_ref() {
                Some(BodyRenderCache::Plain { width, .. }) => *width != area.width,
                _ => true,
            };
            if cache_miss {
                let wrapped = utils::wrap_text(&self.text, usable.max(1));
                *self.body_render_cache.borrow_mut() = Some(BodyRenderCache::Plain {
                    width: area.width,
                    lines: wrapped,
                });
            }
            let cache_ref = self.body_render_cache.borrow();
            let wrapped: &[String] = match cache_ref.as_ref() {
                Some(BodyRenderCache::Plain { lines, .. }) => lines,
                _ => unreachable!("cache was just populated"),
            };
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
            if body.contains(y) {
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
                    if !body.contains(y) {
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
    }
}
