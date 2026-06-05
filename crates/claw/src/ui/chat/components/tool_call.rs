use super::style::{
    BLOCK_LEFT_RESERVED, blend, body_line, block_border, render_block_chrome,
};
use super::{ComponentOp, MessageComponent};
use crate::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use serde_json::Value;
use std::cell::{Cell, RefCell};

pub(crate) struct ToolCallCard {
    name: String,
    args: String,
    /// Parsed args JSON, cached at construction time. `None` if the
    /// args string is not valid JSON. The args string never mutates
    /// after construction, so the cache is permanent — every
    /// `header_text` / `explanation` / `args_rows` / `render` call
    /// shares this single parse.
    args_value: Option<Value>,
    result: String,
    /// Parsed result JSON, cached at construction time. Same
    /// invalidation semantics as `args_value`.
    result_value: Option<Value>,
    step: usize,
    total: usize,
    pub expanded: bool,
    /// When the card is expanded, the Args sub-section can be
    /// collapsed independently. Defaults to `true` so the first
    /// expand reveals everything (matches the pre-sub-toggle UX).
    pub args_expanded: bool,
    /// Same as `args_expanded` but for the Result sub-section.
    pub result_expanded: bool,
    timestamp: Option<String>,
    /// `(outer_width, args_rows)` cache for `args_rows()`. The rows
    /// computation now delegates to `compute_args_render_lines`, so
    /// the value returned here is *exactly* the number of lines the
    /// render path will draw — by construction there is no drift.
    args_rows_cache: Cell<Option<(u16, u16)>>,
    /// `(outer_width, result_rows)` cache. Same shared-path
    /// invariant as `args_rows_cache`.
    result_rows_cache: Cell<Option<(u16, u16)>>,
    /// `(body_w, wrapped_args_lines)` cache for the args render path.
    /// Stores either pretty-printed JSON or wrapped plain text
    /// depending on what the args content required. Keyed by body_w
    /// (which is derived 1:1 from outer width).
    args_render_cache: RefCell<Option<(u16, Vec<String>)>>,
    /// `(body_w, result_render_lines)` cache for the result render
    /// path. Empty `Vec` = result is non-JSON or teach-sentinel;
    /// caller falls back to plain wrap. Same keying as
    /// `args_render_cache`.
    result_render_cache: RefCell<Option<(u16, Vec<Line<'static>>)>>,
    /// Cached `ToolStatus` derived from `result` once at
    /// construction. Avoids re-running `to_lowercase()` per render.
    status: Cell<ToolStatus>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolStatus {
    Running,
    Success,
    Failure,
}

impl ToolStatus {
    fn glyph(self) -> &'static str {
        match self {
            ToolStatus::Running => "◌",
            ToolStatus::Success => "✓",
            ToolStatus::Failure => "✗",
        }
    }

    fn color(self, theme: &Theme) -> Color {
        match self {
            ToolStatus::Running => theme.accent(),
            ToolStatus::Success => Color::Rgb(120, 200, 120),
            ToolStatus::Failure => Color::Rgb(220, 110, 110),
        }
    }
}

fn detect_status(result: &str) -> ToolStatus {
    if result.is_empty() {
        return ToolStatus::Running;
    }
    let lower = result.to_lowercase();
    if lower.contains("error")
        || lower.contains("failed")
        || lower.contains("failure")
        || lower.contains("panic")
        || lower.contains("exception")
    {
        ToolStatus::Failure
    } else {
        ToolStatus::Success
    }
}

/// Convert an outer card width into the body width used by the
/// Args / Result sections. Centralised so `height()` and `render()`
/// cannot disagree — historically they used two different formulas
/// and the resulting row-count drift was the cause of the "tool
/// message expansion is wrong" bug.
fn body_w_for(outer_w: u16) -> u16 {
    outer_w.saturating_sub(BLOCK_LEFT_RESERVED as u16 + 2)
}

impl ToolCallCard {
    pub fn new(
        name: &str,
        args: &str,
        result: &str,
        step: usize,
        total: usize,
        expanded: bool,
        timestamp: Option<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            args_value: serde_json::from_str(args).ok(),
            args: args.to_string(),
            result_value: serde_json::from_str(result).ok(),
            result: result.to_string(),
            step,
            total,
            expanded,
            args_expanded: true,
            result_expanded: true,
            timestamp: timestamp.map(|s| s.to_string()),
            args_rows_cache: Cell::new(None),
            result_rows_cache: Cell::new(None),
            args_render_cache: RefCell::new(None),
            result_render_cache: RefCell::new(None),
            status: Cell::new(detect_status(result)),
        }
    }

    /// Glyph for the tool's category. Mirrors iOS's iconography roughly.
    fn tool_glyph(&self) -> &'static str {
        match self.name.as_str() {
            "i_rs" => "◆",
            "search_conversations" | "search_tools" | "web_search" => "⌕",
            "update_user_memory" => "◉",
            "file_ops" => "▤",
            "generate_image" | "image" => "◐",
            "calculator" | "calc" => "∑",
            "delegate" | "call_code_agent" => "↪",
            _ => "▸",
        }
    }

    /// Short, one-line summary of the *call*: tool name + (command / query).
    /// This is the line that lives in the header, always visible.
    fn header_text(&self) -> String {
        let step_str = if self.total > 1 {
            format!("[{}/{}] ", self.step + 1, self.total)
        } else {
            String::new()
        };
        let Some(val) = &self.args_value else {
            return format!("{}{}", step_str, self.name);
        };
        match self.name.as_str() {
            "i_rs" => {
                let tool = val.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
                let cmd = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
                let cmd_short = utils::truncate_str(cmd, 60);
                format!("{}{}  {}", step_str, tool, cmd_short)
            }
            "search_conversations" => {
                let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                format!("{}搜索历史  {}", step_str, utils::truncate_str(q, 48))
            }
            "search_tools" => {
                let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                format!("{}search  {}", step_str, utils::truncate_str(q, 48))
            }
            "update_user_memory" => format!("{}记住用户信息", step_str),
            "file_ops" => {
                let op = val.get("operation").and_then(|v| v.as_str()).unwrap_or("?");
                let p = val.get("path").and_then(|v| v.as_str()).unwrap_or("?");
                format!("{}{}  {}", step_str, op, utils::truncate_str(p, 48))
            }
            "web_search" => {
                let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                format!("{}搜索网络  {}", step_str, utils::truncate_str(q, 48))
            }
            _ => format!("{}{}", step_str, self.name),
        }
    }

    /// One-line preview of the *result* for the collapsed header.
    fn result_preview(&self) -> Option<String> {
        if self.result.is_empty() {
            return None;
        }
        // Strip surrounding whitespace, take the first non-empty line.
        let first = self.result.lines().map(str::trim).find(|l| !l.is_empty());
        first.map(|l| utils::truncate_str(l, 64).into_owned())
    }

    /// Optional "why" the agent gave for this call.
    fn explanation(&self) -> Option<String> {
        if self.name != "i_rs" {
            return None;
        }
        self.args_value
            .as_ref()
            .and_then(|v| v.get("explanation").and_then(|e| e.as_str().map(String::from)))
    }

    // ─── Render-line computation (single source of truth) ───────────────
    //
    // `compute_args_render_lines` and `compute_result_render_lines`
    // are the *only* place that decides how Args/Result content is
    // wrapped. Both `height()` and `render()` go through them, so
    // the predicted row count can never drift from what actually
    // gets drawn.

    fn args_render_lines(&self, outer_w: u16) -> std::cell::Ref<'_, [String]> {
        let bw = body_w_for(outer_w);
        let cache_miss = match self.args_render_cache.borrow().as_ref() {
            Some((w, _)) => *w != bw,
            None => true,
        };
        if cache_miss {
            let lines = self.compute_args_render_lines(bw);
            *self.args_render_cache.borrow_mut() = Some((bw, lines));
        }
        // Also update the row-count cache so `args_rows()` is free.
        let n = self
            .args_render_cache
            .borrow()
            .as_ref()
            .map(|(_, v)| v.len() as u16)
            .unwrap_or(0);
        self.args_rows_cache.set(Some((outer_w, n)));
        std::cell::Ref::map(self.args_render_cache.borrow(), |opt| {
            opt.as_ref()
                .map(|(_, v)| v.as_slice())
                .expect("cache was just populated")
        })
    }

    fn compute_args_render_lines(&self, body_w: u16) -> Vec<String> {
        // Render-side indent mirrors the historical "  " indent
        // inside body lines, so the wrap column is body_w - 2.
        let wrap_col = (body_w as usize).saturating_sub(2).max(1);
        // Plain-text fallback uses BLOCK_LEFT_RESERVED indent.
        let usable_plain = body_w
            .saturating_sub(BLOCK_LEFT_RESERVED as u16)
            .max(1) as usize;
        if let Some(val) = &self.args_value {
            // Teach-output sentinel — wrap the raw args text.
            if val.as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.trim().starts_with('\u{2318}'))
            {
                return utils::wrap_text(&self.args, usable_plain);
            }
            // Pretty-printed JSON — wrap at wrap_col.
            let pretty = serde_json::to_string_pretty(val)
                .unwrap_or_else(|_| self.args.clone());
            return utils::wrap_text(&pretty, wrap_col);
        }
        // Args is not valid JSON — render as plain text.
        utils::wrap_text(&self.args, usable_plain)
    }

    fn result_render_lines(&self, outer_w: u16) -> std::cell::Ref<'_, [Line<'static>]> {
        let bw = body_w_for(outer_w);
        let cache_miss = match self.result_render_cache.borrow().as_ref() {
            Some((w, _)) => *w != bw,
            None => true,
        };
        if cache_miss {
            let entry = self.compute_result_render_lines(bw).unwrap_or_default();
            *self.result_render_cache.borrow_mut() = Some((bw, entry));
        }
        // Update the row-count cache so `result_rows()` is free.
        let n = self
            .result_render_cache
            .borrow()
            .as_ref()
            .map(|(_, v)| {
                if !v.is_empty() {
                    v.len() as u16
                } else {
                    // Plain-wrap fallback path — count wrapped lines
                    // of self.result at the same indent render uses.
                    let indent = (bw as usize).saturating_sub(2).max(1);
                    utils::wrap_text(&self.result, indent).len() as u16
                }
            })
            .unwrap_or(0);
        self.result_rows_cache.set(Some((outer_w, n)));
        std::cell::Ref::map(self.result_render_cache.borrow(), |opt| {
            opt.as_ref()
                .map(|(_, v)| v.as_slice())
                .expect("cache was just populated")
        })
    }

    /// Produce the formatted JSON lines for the result, or `None`
    /// if the result is non-JSON / teach-sentinel and should fall
    /// back to plain wrap.
    fn compute_result_render_lines(&self, body_w: u16) -> Option<Vec<Line<'static>>> {
        let usable = body_w
            .saturating_sub(BLOCK_LEFT_RESERVED as u16)
            .max(1) as usize;
        let val = self.result_value.as_ref()?;
        if val.as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .is_some_and(|s| s.trim().starts_with('\u{2318}'))
        {
            return None;
        }
        let lines = utils::format_json_lines(val, usable);
        if lines.is_empty() { None } else { Some(lines) }
    }

    // ─── Row counts (delegate to the render-line path) ──────────────────
    //
    // These now exist purely as a cache interface for `height()` —
    // they always return the exact number of lines the render path
    // will draw, because they share the same underlying code.

    fn args_rows(&self, outer_w: u16) -> u16 {
        if self.args.is_empty() {
            return 0;
        }
        if let Some((cached_w, cached_h)) = self.args_rows_cache.get()
            && cached_w == outer_w
        {
            return cached_h;
        }
        // Populate the render cache (which updates args_rows_cache
        // as a side effect) and read back.
        let _ = self.args_render_lines(outer_w);
        self.args_rows_cache
            .get()
            .map(|(_, h)| h)
            .unwrap_or(0)
    }

    fn result_rows(&self, outer_w: u16) -> u16 {
        if self.result.is_empty() {
            return 0;
        }
        if let Some((cached_w, cached_h)) = self.result_rows_cache.get()
            && cached_w == outer_w
        {
            return cached_h;
        }
        let _ = self.result_render_lines(outer_w);
        self.result_rows_cache
            .get()
            .map(|(_, h)| h)
            .unwrap_or(0)
    }
}

impl MessageComponent for ToolCallCard {
    /// Collapsed: 1 (top) + 1 (header) + 1 (bottom) = 3.
    /// Expanded: + 1 (divider)
    ///           + 1 explanation line if present
    ///           + 1 (args header) + args_rows    (if args_expanded)
    ///           + 1 (result header) + result_rows (if result_expanded)
    ///           — Args/Result headers always render so the user can
    ///           click them to expand an empty sub-section; the body
    ///           rows are what's gated on the sub-toggle.
    fn height(&self, width: u16) -> u16 {
        let mut h = 1 + 1 + 1;
        if self.expanded {
            h += 1; // divider
            if self.explanation().is_some() {
                h += 1;
            }
            if !self.args.is_empty() {
                h += 1; // section header
                if self.args_expanded {
                    h += self.args_rows(width);
                }
            }
            if !self.result.is_empty() {
                h += 1; // section header
                if self.result_expanded {
                    h += self.result_rows(width);
                }
            }
        }
        h
    }

    fn extra_click_targets(&self, width: u16) -> Vec<(u16, u16, ComponentOp)> {
        if !self.expanded {
            return vec![];
        }
        let mut targets = Vec::new();
        // Header (top border + glyph line): rows 0..2
        targets.push((0, 2, ComponentOp::Toggle));
        // Walk through body layout matching render order
        let mut y = 2u16; // after top border and header
        y += 1; // divider
        if self.explanation().is_some() {
            y += 1;
        }
        if !self.args.is_empty() {
            targets.push((y, 1, ComponentOp::ToggleArgs));
            y += 1;
            if self.args_expanded {
                y = y.saturating_add(self.args_rows(width));
            }
        }
        if !self.result.is_empty() {
            targets.push((y, 1, ComponentOp::ToggleResult));
        }
        targets
    }

    fn clickable(&self) -> bool { true }

    fn apply(&mut self, op: ComponentOp) {
        match op {
            ComponentOp::Toggle => self.expanded = !self.expanded,
            ComponentOp::ToggleArgs => {
                // Auto-expand the card so the user sees the effect.
                if !self.expanded {
                    self.expanded = true;
                }
                self.args_expanded = !self.args_expanded;
            }
            ComponentOp::ToggleResult => {
                if !self.expanded {
                    self.expanded = true;
                }
                self.result_expanded = !self.result_expanded;
            }
            _ => {}
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let border = block_border(theme, selected);
        let interior_bg = if selected {
            blend(theme.tool_surface(), theme.primary(), 0.25)
        } else {
            theme.tool_surface()
        };
        let accent_color = theme.accent();
        let status = self.status.get();
        let status_color = status.color(theme);

        // Build the custom header (status + tool glyph + name + preview + chevron + ts).
        let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
        let mut header_spans: Vec<Span<'static>> = Vec::with_capacity(8);
        header_spans.push(Span::raw(" ".repeat(BLOCK_LEFT_RESERVED)));
        header_spans.push(Span::styled(
            status.glyph().to_string(),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::raw(" "));
        header_spans.push(Span::styled(
            self.tool_glyph().to_string(),
            Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::raw(" "));
        header_spans.push(Span::styled(
            self.header_text(),
            Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
        ));
        if let Some(preview) = self.result_preview()
            && !self.expanded
        {
            // Truncate available width so preview doesn't collide with chevron.
            let used: usize = header_spans.iter().map(|s| s.content.chars().count()).sum();
            let chevron_w = 4; // "  ▸" or "  ▾"
            let left = usable.saturating_sub(used + chevron_w);
            let preview_clipped = if preview.chars().count() > left {
                utils::truncate_str(&preview, left.saturating_sub(1)).into_owned()
            } else {
                preview
            };
            header_spans.push(Span::styled(
                format!("  {}", preview_clipped),
                Style::default().fg(theme.dim_text()),
            ));
        }
        let chevron = if self.expanded { "▾" } else { "▸" };
        header_spans.push(Span::styled(
            format!("  {}", chevron),
            Style::default().fg(theme.dim_text()).add_modifier(Modifier::BOLD),
        ));
        if let Some(ts) = &self.timestamp {
            // Drop timestamp if it would overflow — keep the preview instead.
            let used: usize = header_spans.iter().map(|s| s.content.chars().count()).sum();
            if used + ts.chars().count() + 2 <= usable {
                header_spans.push(Span::styled(
                    format!("  {}", ts),
                    Style::default().fg(theme.dim_text()),
                ));
            }
        }

        let body = render_block_chrome(area, buf, border, interior_bg, Line::from(header_spans));
        let mut y = body.top;

        if self.expanded {
            // Divider
            if body.contains(y) {
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(area.width.saturating_sub(2) as usize),
                    Style::default().fg(border),
                )))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }

            // Explanation
            if let Some(exp) = self.explanation()
                && body.contains(y)
            {
                Paragraph::new(body_line(
                    &format!("↳  {}", utils::truncate_str(&exp, 200)),
                    Style::default().fg(theme.dim_text()),
                ))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }

            let bw = body_w_for(area.width);

            // Arguments section header — always shown so users can
            // click to expand even when args_expanded=false.
            if !self.args.is_empty() && body.contains(y) {
                let args_chevron = if self.args_expanded { "▾" } else { "▸" };
                Paragraph::new(body_line(
                    &format!("{} Arguments", args_chevron),
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;

                if self.args_expanded {
                    let cache_ref = self.args_render_lines(area.width);
                    let take = (cache_ref.len() as u16).min(body.bottom.saturating_sub(y));
                    // Pick the rendering color based on the variant.
                    // Pretty-printed JSON path uses theme.text();
                    // teach-sentinel / non-JSON uses dim_text().
                    let is_pretty = self
                        .args_value
                        .as_ref()
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                        .map(|s| !s.trim().starts_with('\u{2318}'))
                        .unwrap_or(false);
                    let line_style = if is_pretty {
                        Style::default().fg(theme.text())
                    } else {
                        Style::default().fg(theme.dim_text())
                    };
                    for (i, line) in cache_ref.iter().take(take as usize).enumerate() {
                        Paragraph::new(body_line(&format!("  {}", line), line_style))
                            .style(Style::default().bg(interior_bg))
                            .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
                    }
                    y += take;
                }
            }

            // Result section header — same expand-on-click affordance.
            if !self.result.is_empty() && body.contains(y) {
                let result_chevron = if self.result_expanded { "▾" } else { "▸" };
                Paragraph::new(body_line(
                    &format!("{} Result", result_chevron),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;

                if self.result_expanded {
                    let cache_ref = self.result_render_lines(area.width);
                    if !cache_ref.is_empty() {
                        let take = (cache_ref.len() as u16).min(body.bottom.saturating_sub(y));
                        for (i, fl) in cache_ref.iter().take(take as usize).enumerate() {
                            let spans: Vec<Span<'static>> = fl
                                .spans
                                .iter()
                                .map(|s| Span::styled(s.content.clone(), s.style.fg(theme.text())))
                                .collect();
                            Paragraph::new(Line::from(spans))
                                .style(Style::default().bg(interior_bg))
                                .render(
                                    Rect {
                                        y: y + i as u16,
                                        height: 1,
                                        ..area
                                    },
                                    buf,
                                );
                        }
                    } else {
                        // Plain-wrap fallback for non-JSON / sentinel.
                        let indent = (bw as usize).saturating_sub(2).max(1);
                        let wrapped = utils::wrap_text(&self.result, indent);
                        let take = (wrapped.len() as u16).min(body.bottom.saturating_sub(y));
                        for (i, line) in wrapped.iter().take(take as usize).enumerate() {
                            Paragraph::new(body_line(
                                &format!("  {}", line),
                                Style::default().fg(theme.dim_text()),
                            ))
                            .style(Style::default().bg(interior_bg))
                            .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: the "tool message expansion is wrong" bug was
    /// caused by `height()` and `render()` using two different
    /// formulas for the wrap width. The fix is to make them share
    /// code, so this test asserts the invariant directly: for any
    /// combination of input / width, `args_rows` must equal the
    /// number of lines `args_render_lines` produces.
    #[test]
    fn args_rows_matches_render_lines_for_plain_text() {
        let card = ToolCallCard::new(
            "file_ops",
            "not valid json — just a long plain text string that should wrap multiple times when the card is narrow enough",
            "",
            0,
            1,
            false,
            None,
        );
        for width in [20u16, 40, 60, 80, 100] {
            let rows = card.args_rows(width);
            let lines = card.args_render_lines(width);
            assert_eq!(
                rows as usize,
                lines.len(),
                "width={}: height-rows {} != render-lines {}",
                width,
                rows,
                lines.len()
            );
        }
    }

    #[test]
    fn args_rows_matches_render_lines_for_pretty_json() {
        // Valid JSON takes the pretty-print path, which has a
        // different indent from the plain-text fallback. Both
        // paths must still agree with the row count.
        let args = r#"{"command":"add","tool":"todo","explanation":"Adding a new todo item to the list because the user asked","title":"Write tests for tool_call expansion"}"#;
        let card = ToolCallCard::new("i_rs", args, "", 0, 1, false, None);
        for width in [30u16, 50, 80, 120] {
            let rows = card.args_rows(width);
            let lines = card.args_render_lines(width);
            assert_eq!(
                rows as usize,
                lines.len(),
                "width={}: height-rows {} != render-lines {}",
                width,
                rows,
                lines.len()
            );
        }
    }

    #[test]
    fn result_rows_matches_render_lines_for_plain_text() {
        let card = ToolCallCard::new(
            "i_rs",
            "{}",
            "Added todo 'Write tests' successfully at 2024-01-01. The item is now visible in the list.",
            0,
            1,
            false,
            None,
        );
        for width in [20u16, 40, 60, 80] {
            let rows = card.result_rows(width);
            let lines = card.result_render_lines(width);
            // Plain-wrap fallback path stores empty Vec in the
            // cache; the row count comes from wrap_text of self.result.
            // Both should match the actual wrap.
            assert_eq!(
                rows as usize,
                lines.len().max({
                    let bw = body_w_for(width) as usize;
                    let indent = bw.saturating_sub(2).max(1);
                    utils::wrap_text(&card.result, indent).len()
                }),
                "width={}: height-rows {} != render-lines-or-fallback {}",
                width,
                rows,
                lines.len()
            );
        }
    }

    #[test]
    fn result_rows_matches_render_lines_for_json() {
        let result = r#"{"ok":true,"item":{"id":"todo-1","title":"Write tests","created_at":"2024-01-01T12:00:00Z"}}"#;
        let card = ToolCallCard::new("i_rs", "{}", result, 0, 1, false, None);
        for width in [30u16, 50, 80, 120] {
            let rows = card.result_rows(width);
            let lines = card.result_render_lines(width);
            assert_eq!(
                rows as usize,
                lines.len(),
                "width={}: height-rows {} != render-lines {}",
                width,
                rows,
                lines.len()
            );
        }
    }

    /// When collapsed, height is exactly 3 regardless of content.
    #[test]
    fn collapsed_height_is_three() {
        let card = ToolCallCard::new(
            "i_rs",
            r#"{"tool":"todo","command":"add"}"#,
            "ok",
            0,
            1,
            false,
            None,
        );
        assert_eq!(card.height(80), 3);
        assert_eq!(card.height(20), 3);
    }

    /// Sub-toggles: Args and Result can be collapsed independently
    /// inside an expanded card. The card's outer `expanded` state
    /// still controls whether *any* body shows.
    #[test]
    fn sub_toggles_collapse_args_and_result_independently() {
        let args = r#"{"tool":"todo","command":"add","title":"test"}"#;
        let result = r#"{"ok":true}"#;
        let mut card = ToolCallCard::new("i_rs", args, result, 0, 1, false, None);
        let collapsed = card.height(80);
        card.apply(ComponentOp::Toggle);
        let both_expanded = card.height(80);
        assert!(both_expanded > collapsed, "expanding grows the card");

        card.apply(ComponentOp::ToggleArgs);
        let args_collapsed = card.height(80);
        assert!(
            args_collapsed < both_expanded,
            "ToggleArgs shrinks the card: {} < {}",
            args_collapsed,
            both_expanded
        );

        card.apply(ComponentOp::ToggleResult);
        let both_collapsed = card.height(80);
        assert!(
            both_collapsed < args_collapsed,
            "ToggleResult shrinks further: {} < {}",
            both_collapsed,
            args_collapsed
        );

        // Re-expand both — height returns to the original.
        card.apply(ComponentOp::ToggleArgs);
        card.apply(ComponentOp::ToggleResult);
        assert_eq!(card.height(80), both_expanded);
    }

    /// `ToggleArgs` on a collapsed card auto-expands the card so
    /// the user sees the effect. Otherwise the key would seem dead.
    #[test]
    fn sub_toggle_auto_expands_card() {
        let mut card = ToolCallCard::new(
            "i_rs",
            r#"{"tool":"todo"}"#,
            "ok",
            0,
            1,
            false,
            None,
        );
        assert!(!card.expanded);
        card.apply(ComponentOp::ToggleArgs);
        assert!(card.expanded, "ToggleArgs on collapsed card expands it");
        assert!(!card.args_expanded, "and flips args state");
    }
}

