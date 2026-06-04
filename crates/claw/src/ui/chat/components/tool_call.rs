use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::utils;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct ToolCallCard {
    name: String, args: String, result: String, step: usize, total: usize, pub expanded: bool,
}

impl ToolCallCard {
    pub fn new(name: &str, args: &str, result: &str, step: usize, total: usize, expanded: bool) -> Self {
        Self { name: name.to_string(), args: args.to_string(), result: result.to_string(), step, total, expanded }
    }

    fn header_text(&self) -> String {
        let step_str = if self.total > 1 { format!("[{}/{}] ", self.step + 1, self.total) } else { String::new() };
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&self.args) {
            match self.name.as_str() {
                "i_rs" => {
                    let tool = val.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
                    let cmd = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
                    format!("⚡ {}{} {}", step_str, tool, cmd)
                }
                "search_conversations" => {
                    let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                    format!("🔍 搜索历史: {}", q)
                }
                "search_tools" => {
                    let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                    format!("🔍 search: {}", q)
                }
                "update_user_memory" => "🧠 记住用户信息".to_string(),
                "file_ops" => {
                    let op = val.get("operation").and_then(|v| v.as_str()).unwrap_or("?");
                    let p = val.get("path").and_then(|v| v.as_str()).unwrap_or("?");
                    format!("📁 {}: {}", op, p)
                }
                "web_search" => {
                    let q = val.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                    format!("🌐 搜索网络: {}", q)
                }
                _ => format!("⚡ {}{}", step_str, self.name),
            }
        } else {
            format!("⚡ {} {}", step_str, self.name)
        }
    }

    fn explanation(&self) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(&self.args).ok()
            .filter(|_| self.name == "i_rs")
            .and_then(|v| v.get("explanation").and_then(|e| e.as_str().map(|s| s.to_string())))
    }

    fn result_lines(&self, width: u16) -> u16 {
        let w = width.saturating_sub(6) as usize;
        if self.result.is_empty() { return 0; }
        let (fmt, _) = utils::format_json_result(&self.result, w);
        if !fmt.is_empty() { return fmt.len() as u16; }
        self.result.split('\n').map(|l| {
            let c = l.chars().count();
            if c == 0 { 1u16 } else { ((c + w - 1) / w) as u16 }
        }).sum::<u16>().max(1)
    }
}

impl MessageComponent for ToolCallCard {
    fn height(&self, width: u16) -> u16 {
        let mut h = 1u16;
        if self.expanded {
            if self.explanation().is_some() { h += 1; }
            h += self.result_lines(width);
        }
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let (_, ac) = theme.tool_colors();
        let bg = if selected { ratatui::style::Color::Rgb(40, 35, 55) } else { theme.background() };
        let indent = "   ";
        let mut y = area.y;

        // header — clickable
        let toggle = if self.expanded { " ▾" } else { " ▸" };
        Paragraph::new(Line::from(Span::styled(
            format!("{}{}{}", indent, self.header_text(), toggle),
            Style::default().fg(ac).add_modifier(Modifier::BOLD),
        ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        if self.expanded {
            let w = area.width.saturating_sub(6) as usize;
            if let Some(exp) = self.explanation() {
                Paragraph::new(Line::from(Span::styled(
                    format!("{}    └─ {}", indent, exp),
                    Style::default().fg(theme.dim_text()),
                ))).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
                y += 1;
            }
            if !self.result.is_empty() {
                let (fmt, _) = utils::format_json_result(&self.result, w);
                let max_h = (area.y + area.height).saturating_sub(y);
                if !fmt.is_empty() {
                    let h = (fmt.len() as u16).min(max_h);
                    if h > 0 {
                        let lines: Vec<Line> = fmt.into_iter().take(h as usize).map(|l| {
                            let mut spans = l.spans; for s in &mut spans { s.style = s.style.fg(theme.text()); }
                            Line::from(spans).style(Style::default().bg(bg))
                        }).collect();
                        Paragraph::new(lines).render(Rect { y, height: h, ..area }, buf);
                    }
                } else {
                    let lines: Vec<Line> = self.result.split('\n')
                        .flat_map(|l| {
                            let chars: Vec<char> = l.chars().collect();
                            if chars.is_empty() { return vec![Line::from("")]; }
                            chars.chunks(w).map(|chunk| {
                                Line::from(Span::styled(
                                    format!("{}    {}", indent, chunk.iter().collect::<String>()),
                                    Style::default().fg(theme.text()),
                                ))
                            }).collect()
                        }).collect();
                    let h = (lines.len() as u16).min(max_h);
                    if h > 0 { Paragraph::new(lines).render(Rect { y, height: h, ..area }, buf); }
                }
            }
        }
    }

    fn clickable(&self) -> bool { true }
}
