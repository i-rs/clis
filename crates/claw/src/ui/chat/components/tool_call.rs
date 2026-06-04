use super::style::{
    BLOCK_LEFT_RESERVED, blend, body_line, block_border, rounded_bottom, rounded_top,
};
use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
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

    /// Glyph used to identify this tool's category in the header.
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

    /// Short, human-readable summary that goes in the header.
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
                "update_user_memory" => "记住用户信息".to_string(),
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

    /// Optional one-line "why" the agent gave for this call.
    fn explanation(&self) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(&self.args)
            .ok()
            .filter(|_| self.name == "i_rs")
            .and_then(|v| {
                v.get("explanation")
                    .and_then(|e| e.as_str().map(|s| s.to_string()))
            })
    }

    /// Number of body rows the *expanded* result would occupy.
    fn result_rows(&self, width: u16) -> u16 {
        if self.result.is_empty() {
            return 0;
        }
        let usable = width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
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
    /// 1 (top) + 1 (header) + 0..(1 explanation + N result) + 1 (bottom)
    fn height(&self, width: u16) -> u16 {
        let mut h = 1 + 1 + 1;
        if self.expanded {
            if self.explanation().is_some() {
                h += 1;
            }
            h += self.result_rows(width);
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
        let avatar_color = accent_color;

        let mut y = area.y;

        // Top border
        Paragraph::new(rounded_top(area.width, border))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Header: glyph + tool name + (timestamp)
        let mut header_spans: Vec<Span<'static>> = Vec::with_capacity(6);
        header_spans.push(Span::raw(" ".repeat(BLOCK_LEFT_RESERVED)));
        header_spans.push(Span::styled(
            self.tool_glyph().to_string(),
            Style::default().fg(avatar_color).add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::raw(" "));
        header_spans.push(Span::styled(
            self.header_text(),
            Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
        ));
        if let Some(ts) = &self.timestamp {
            header_spans.push(Span::styled(
                format!("  {}", ts),
                Style::default().fg(theme.dim_text()),
            ));
        }
        Paragraph::new(Line::from(header_spans))
            .style(Style::default().bg(interior_bg))
            .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // Sub-row: expand/collapse hint
        let toggle_icon = if self.expanded { "▾" } else { "▸" };
        let toggle_hint = if self.expanded {
            "点击收起"
        } else if self.result.is_empty() && self.explanation().is_none() {
            "点击展开"
        } else {
            "点击查看结果"
        };
        Paragraph::new(body_line(
            &format!("{}  {}", toggle_icon, toggle_hint),
            Style::default().fg(theme.dim_text()).add_modifier(Modifier::ITALIC),
        ))
        .style(Style::default().bg(interior_bg))
        .render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        if self.expanded {
            if let Some(exp) = self.explanation() {
                if y < area.y + area.height.saturating_sub(1) {
                    Paragraph::new(body_line(
                        &format!("↳  {}", utils::truncate_str(&exp, 200)),
                        Style::default().fg(theme.dim_text()),
                    ))
                    .style(Style::default().bg(interior_bg))
                    .render(Rect { y, height: 1, ..area }, buf);
                    y += 1;
                }
            }
            if !self.result.is_empty() {
                let usable = area.width.saturating_sub(BLOCK_LEFT_RESERVED as u16).max(1) as usize;
                let max_h = (area.y + area.height).saturating_sub(y + 1);
                let (fmt, ok) = utils::format_json_result(&self.result, usable);
                if ok && !fmt.is_empty() {
                    let take = (fmt.len() as u16).min(max_h);
                    for (i, fl) in fmt.into_iter().take(take as usize).enumerate() {
                        // Re-tint each span to use the theme text color
                        // while preserving the formatter's intent
                        // (it already produces styled spans).
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
                    // Plain-text fallback: chunk by visible width.
                    let lines: Vec<Line> = self
                        .result
                        .split('\n')
                        .flat_map(|l| {
                            let chars: Vec<char> = l.chars().collect();
                            if chars.is_empty() {
                                return vec![body_line("", Style::default().fg(theme.text()))];
                            }
                            chars
                                .chunks(usable.max(1))
                                .map(|chunk| {
                                    body_line(
                                        &chunk.iter().collect::<String>(),
                                        Style::default().fg(theme.text()),
                                    )
                                })
                                .collect()
                        })
                        .collect();
                    let take = (lines.len() as u16).min(max_h);
                    for (i, l) in lines.into_iter().take(take as usize).enumerate() {
                        Paragraph::new(l)
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
                }
            }
        }

        // Bottom border.
        if area.height >= 1 {
            let by = area.y + area.height - 1;
            Paragraph::new(rounded_bottom(area.width, border))
                .style(Style::default().bg(interior_bg))
                .render(Rect { y: by, height: 1, ..area }, buf);
        }
    }

    fn clickable(&self) -> bool {
        true
    }
}
