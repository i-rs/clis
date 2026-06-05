//! Collapsible tool-result card with diff display and step counter.

use super::{ComponentOp, blend, block_border, render_block_chrome, header_line, body_line};
use crate::tui::colors::*;
use crate::tui::ui::chat::{is_diff_output, render_diff_line};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use std::cell::Cell;

pub(crate) struct ToolResultCard {
    content: String,
    diff: Option<String>,
    step: usize,
    total_steps: usize,
    collapsed: bool,
    height_cache: Cell<Option<(u16, u16)>>,
}

impl ToolResultCard {
    pub fn new(
        content: &str,
        diff: Option<&str>,
        step: usize,
        total_steps: usize,
        collapsed: bool,
    ) -> Self {
        Self {
            content: content.to_string(),
            diff: diff.map(|s| s.to_string()),
            step,
            total_steps,
            collapsed,
            height_cache: Cell::new(None),
        }
    }

    fn body_lines(&self) -> u16 {
        let (_tool_name, tool_result) = self.content.split_once('\n').unwrap_or(("", &self.content));

        if self.collapsed {
            if tool_result.is_empty() { 0 } else {
                (tool_result.lines().count().min(3) + 1) as u16
            }
        } else {
            let mut h = 0u16;
            if !tool_result.is_empty() {
                h += tool_result.lines().count().min(12) as u16;
                if tool_result.lines().count() > 12 || self.content.chars().count() > 1200 {
                    h += 1; // "… more bytes"
                }
            }
            if let Some(ref d) = self.diff && !d.is_empty() {
                h += 1 + d.lines().count().min(12) as u16; // "─ diff ─" header + lines
                if d.lines().count() > 12 {
                    h += 1; // "... more lines"
                }
            }
            h
        }
    }
}

impl super::MessageComponent for ToolResultCard {
    fn height(&self, width: u16) -> u16 {
        if let Some((cw, ch)) = self.height_cache.get() && cw == width { return ch; }
        let h = 1 + 1 + self.body_lines() + 1;
        self.height_cache.set(Some((width, h)));
        h
    }

    fn clickable(&self) -> bool { true }

    fn apply(&mut self, op: ComponentOp) {
        if op == ComponentOp::Toggle {
            self.collapsed = !self.collapsed;
            self.height_cache.set(None);
        }
    }

    fn extra_click_targets(&self, _width: u16) -> Vec<(u16, u16, ComponentOp)> {
        // Entire header row is clickable
        vec![(1, 1, ComponentOp::Toggle)]
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _y_offset: u16, selected: bool) {
        let border = block_border(selected);
        let bg = if selected { blend(c_bg_tool(), c_accent(), 0.08) } else { c_bg_tool() };

        let (tool_name, tool_result) = self.content.split_once('\n').unwrap_or(("", &self.content));
        let label = if tool_name.is_empty() { "Tool" } else { tool_name };

        let step_str = if self.total_steps > 1 {
            Some(format!("[{}/{}]", self.step, self.total_steps))
        } else {
            None
        };

        let chevron = if self.collapsed { "▸" } else { "▾" };

        let header = header_line(label, chevron, c_orange(), c_text(), step_str.as_deref());
        let body = render_block_chrome(area, buf, border, bg, header, _y_offset);
        let mut y = body.top;

        if self.collapsed {
            if !tool_result.is_empty() {
                let preview: String = tool_result.chars().take(400).collect();
                for line in preview.lines().take(3) {
                    if !body.contains(y) { break; }
                    let truncated: String = line.chars().take(120).collect();
                    let mut text = truncated;
                    if line.chars().count() > 120 { text.push('…'); }
                    Paragraph::new(body_line(&text, Style::default().fg(c_dim())))
                        .style(Style::default().bg(bg))
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
                if preview.len() < tool_result.len() {
                    let remaining = tool_result.len().saturating_sub(preview.len());
                    let more_text = format!("… {} more bytes", remaining);
                    Paragraph::new(body_line(&more_text, Style::default().fg(c_muted())))
                        .style(Style::default().bg(bg))
                        .render(Rect { y, height: 1, ..area }, buf);
                }
            }
        } else {
            let has_diff = is_diff_output(tool_result);
            let preview: String = tool_result.chars().take(1200).collect();

            if !tool_result.is_empty() {
                for line in preview.lines().take(12) {
                    if !body.contains(y) { break; }
                    if has_diff {
                        let mut spans: Vec<Span<'static>> = vec![
                            Span::raw(" ".repeat(super::BLOCK_INDENT)),
                            ];
                        spans.extend(render_diff_line(line));
                        Paragraph::new(Line::from(spans))
                            .style(Style::default().bg(bg))
                            .render(Rect { y, height: 1, ..area }, buf);
                    } else {
                        Paragraph::new(body_line(line, Style::default().fg(c_tool_output())))
                            .style(Style::default().bg(bg))
                            .render(Rect { y, height: 1, ..area }, buf);
                    }
                    y += 1;
                }
                if preview.len() < tool_result.len() || tool_result.lines().count() > 12 {
                    let remaining = tool_result.len().saturating_sub(preview.len());
                    let more_text = format!("… {} more bytes", remaining);
                    Paragraph::new(body_line(&more_text, Style::default().fg(c_muted())))
                        .style(Style::default().bg(bg))
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
            }

            // Diff section
            if let Some(ref diff_text) = self.diff && !diff_text.is_empty() {
                if !body.contains(y) { return; }
                Paragraph::new(body_line("─ diff ─", Style::default().fg(c_dim())))
                    .style(Style::default().bg(bg))
                    .render(Rect { y, height: 1, ..area }, buf);
                y += 1;

                for line in diff_text.lines().take(12) {
                    if !body.contains(y) { break; }
                    let mut spans: Vec<Span<'static>> = vec![
                        Span::raw(" ".repeat(super::BLOCK_INDENT)),
                        ];
                    spans.extend(render_diff_line(line));
                    Paragraph::new(Line::from(spans))
                        .style(Style::default().bg(bg))
                        .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
                if diff_text.lines().count() > 12 {
                    let more_text = format!("... +{} more lines", diff_text.lines().count().saturating_sub(12));
                    Paragraph::new(body_line(&more_text, Style::default().fg(c_muted())))
                        .style(Style::default().bg(bg))
                        .render(Rect { y, height: 1, ..area }, buf);
                }
            }
        }
    }
}
