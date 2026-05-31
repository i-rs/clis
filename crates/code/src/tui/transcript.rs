use crate::app::AgentMessage;
use crate::tui::colors::*;
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render_transcript(frame: &mut Frame, app: &crate::app::App) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Transcript (Ctrl+T to close, ↑↓ scroll) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for msg in &app.messages {
        match msg {
            AgentMessage::User { content } => {
                lines.push(Line::from(Span::styled(
                    "── User ──",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
            }
            AgentMessage::Assistant {
                content,
                reasoning,
                tool_calls,
                reasoning_expanded: _,
            } => {
                lines.push(Line::from(Span::styled(
                    "── Assistant ──",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                if !reasoning.is_empty() {
                    for line in reasoning.lines() {
                        lines.push(Line::from(Span::styled(
                            line.to_string(),
                            Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC),
                        )));
                    }
                }
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
                if let Some(tcs) = tool_calls {
                    for tc in tcs {
                        let name = tc.get("name").and_then(|v| v.as_str()).unwrap_or("tool");
                        lines.push(Line::from(Span::styled(
                            format!("  [tool_call] {}", name),
                            Style::default().fg(Color::Yellow),
                        )));
                    }
                }
            }
            AgentMessage::ToolResult { content, .. } => {
                let (tool_name, tool_result) = content.split_once('\n').unwrap_or(("", content));
                lines.push(Line::from(Span::styled(
                    format!(
                        "── {} ──",
                        if tool_name.is_empty() {
                            "Tool"
                        } else {
                            tool_name
                        }
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                if !tool_result.is_empty() {
                    for line in tool_result.lines() {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
            }
            AgentMessage::FileEdit { path, summary } => {
                lines.push(Line::from(Span::styled(
                    format!("── File Edit: {} ──", path),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )));
                for line in summary.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
            }
            AgentMessage::System { content } => {
                lines.push(Line::from(Span::styled(
                    "── System ──",
                    Style::default().fg(C_DIM),
                )));
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
            }
            AgentMessage::Separator { .. } => {}
        }
        lines.push(Line::from(""));
    }

    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let scroll = app.transcript_scroll.min(max_scroll);
    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, inner);
}
