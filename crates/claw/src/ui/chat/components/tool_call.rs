use super::style::{
    BLOCK_LEFT_RESERVED, blend, body_line, block_border, rounded_bottom, rounded_top,
};
use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ToolCallCard {
    name: String,
    args: String,
    result: String,
    step: usize,
    total: usize,
    pub expanded: bool,
    timestamp: Option<String>,
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
            args: args.to_string(),
            result: result.to_string(),
            step,
            total,
            expanded,
            timestamp: timestamp.map(|s| s.to_string()),
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
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&self.args) {
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
        } else {
            format!("{}{}", step_str, self.name)
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
        serde_json::from_str::<serde_json::Value>(&self.args)
            .ok()
            .filter(|_| self.name == "i_rs")
            .and_then(|v| {
                v.get("explanation")
                    .and_then(|e| e.as_str().map(|s| s.to_string()))
            })
    }

    /// How many rows does the *Args* section take when expanded.
    fn args_rows(&self, width: u16) -> u16 {
        if self.args.is_empty() {
            return 0;
        }
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16 + 4).max(1) as usize;
        // Try pretty JSON first.
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&self.args) {
            if let Some(arr) = val.as_array()
                && arr
                    .first()
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s.trim().starts_with('\u{2318}'))
            {
                return utils::wrap_text(&self.args, usable).len().max(1) as u16;
            }
            let pretty = serde_json::to_string_pretty(&val).unwrap_or_else(|_| self.args.clone());
            return utils::wrap_text(&pretty, usable).len().max(1) as u16;
        }
        utils::wrap_text(&self.args, usable).len().max(1) as u16
    }

    /// How many rows does the *Result* section take when expanded.
    fn result_rows(&self, width: u16) -> u16 {
        if self.result.is_empty() {
            return 0;
        }
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16 + 4).max(1) as usize;
        let (fmt, ok) = utils::format_json_result(&self.result, usable);
        if ok && !fmt.is_empty() {
            return fmt.len() as u16;
        }
        self.result
            .split('\n')
            .map(|l| {
                let c = l.chars().count();
                if c == 0 {
                    1u16
                } else {
                    ((c + usable - 1) / usable).max(1) as u16
                }
            })
            .sum::<u16>()
            .max(1)
    }
}

impl MessageComponent for ToolCallCard {
    /// Collapsed: 1 (top) + 1 (header) + 1 (bottom) = 3.
    /// Expanded: + 1 (divider) + 1 (args header) + args_rows
    ///           + 1 (result header) + result_rows
    ///           (+ 1 explanation line if present).
    fn height(&self, width: u16) -> u16 {
        let mut h = 1 + 1 + 1;
        if self.expanded {
            if self.explanation().is_some() {
                h += 1;
            }
            if !self.args.is_empty() {
                h += 1 + self.args_rows(width); // section header + body
            }
            if !self.result.is_empty() {
                h += 1 + self.result_rows(width);
            }
        }
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let border = block_border(theme, selected);
        let interior_bg = if selected {
            blend(theme.tool_surface(), theme.primary(), 0.25)
        } else {
            theme.tool_surface()
        };
        let accent_color = theme.accent();
        let status = detect_status(&self.result);
        let status_color = status.color(theme);

        let mut y = area.y;

        // Top border
        Paragraph::new(rounded_top(area.width, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Header: status icon + tool glyph + tool name + result preview + chevron
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
        Paragraph::new(Line::from(header_spans))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        if self.expanded {
            let bottom = area.y + area.height.saturating_sub(1);
            // Divider
            if y < bottom {
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
                && y < bottom
            {
                Paragraph::new(body_line(
                    &format!("↳  {}", utils::truncate_str(&exp, 200)),
                    Style::default().fg(theme.dim_text()),
                ))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }

            // Arguments section
            if !self.args.is_empty() && y < bottom {
                Paragraph::new(body_line(
                    "▸ Arguments",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
                let body_w = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16 + 2);
                let take = self.args_rows(body_w).min(bottom.saturating_sub(y));
                if take > 0 {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&self.args) {
                        if let Some(arr) = val.as_array()
                            && arr
                                .first()
                                .and_then(|v| v.as_str())
                                .is_some_and(|s| s.trim().starts_with('\u{2318}'))
                        {
                            // Non-JSON teach output — render as plain text
                            for (i, line) in utils::wrap_text(&self.args, body_w.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize)
                                .iter()
                                .take(take as usize)
                                .enumerate()
                            {
                                Paragraph::new(body_line(
                                    &format!("  {}", line),
                                    Style::default().fg(theme.dim_text()),
                                ))
                                .style(Style::default().bg(interior_bg))
                                .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
                            }
                            y += take;
                        } else {
                            let pretty = serde_json::to_string_pretty(&val)
                                .unwrap_or_else(|_| self.args.clone());
                            let indent = body_w.saturating_sub(2).max(1) as usize;
                            for (i, line) in utils::wrap_text(&pretty, indent)
                                .iter()
                                .take(take as usize)
                                .enumerate()
                            {
                                Paragraph::new(body_line(
                                    &format!("  {}", line),
                                    Style::default().fg(theme.text()),
                                ))
                                .style(Style::default().bg(interior_bg))
                                .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
                            }
                            y += take;
                        }
                    } else {
                        for (i, line) in utils::wrap_text(
                            &self.args,
                            body_w.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize,
                        )
                        .iter()
                        .take(take as usize)
                        .enumerate()
                        {
                            Paragraph::new(body_line(
                                &format!("  {}", line),
                                Style::default().fg(theme.dim_text()),
                            ))
                            .style(Style::default().bg(interior_bg))
                            .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
                        }
                        y += take;
                    }
                }
            }

            // Result section
            if !self.result.is_empty() && y < bottom {
                Paragraph::new(body_line(
                    "▸ Result",
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y, height: 1, ..area }, buf);
                y += 1;
                let body_w = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16 + 2);
                let (fmt, ok) = utils::format_json_result(&self.result, body_w.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize);
                if ok && !fmt.is_empty() {
                    let take = (fmt.len() as u16).min(bottom.saturating_sub(y));
                    for (i, fl) in fmt.into_iter().take(take as usize).enumerate() {
                        let spans: Vec<Span<'static>> = fl
                            .spans
                            .into_iter()
                            .map(|s| Span::styled(s.content, s.style.fg(theme.text())))
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
                    y += take;
                } else {
                    let indent = body_w.saturating_sub(2).max(1) as usize;
                    let take = (utils::wrap_text(&self.result, indent).len() as u16)
                        .min(bottom.saturating_sub(y));
                    for (i, line) in utils::wrap_text(&self.result, indent)
                        .into_iter()
                        .take(take as usize)
                        .enumerate()
                    {
                        Paragraph::new(body_line(
                            &format!("  {}", line),
                            Style::default().fg(theme.dim_text()),
                        ))
                        .style(Style::default().bg(interior_bg))
                        .render(Rect { y: y + i as u16, height: 1, ..area }, buf);
                    }
                    y += take;
                }
            }
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
