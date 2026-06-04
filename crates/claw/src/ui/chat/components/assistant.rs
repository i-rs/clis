use super::MessageComponent;
use crate::theme::Theme;
use crate::ui::chat::markdown::render_markdown;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) struct AssistantBlock {
    text: String,
    reasoning: String,
    pub reasoning_expanded: bool,
    md_cache: std::cell::RefCell<Option<Vec<Line<'static>>>>,
    md_width: std::cell::Cell<u16>,
}

impl AssistantBlock {
    pub fn new(text: &str, reasoning: &str, reasoning_expanded: bool) -> Self {
        Self { text: text.to_string(), reasoning: reasoning.to_string(), reasoning_expanded,
              md_cache: std::cell::RefCell::new(None), md_width: std::cell::Cell::new(0) }
    }

    fn text_count(text: &str, width: u16) -> u16 {
        let w = width.saturating_sub(6) as usize;
        text.split('\n').map(|l| {
            let c = l.chars().count();
            if c == 0 { 1 } else { (c + w - 1) / w }
        }).sum::<usize>().max(1) as u16
    }

    fn get_md(&self, width: u16, theme: &Theme) -> Vec<Line<'static>> {
        if self.md_width.get() != width {
            let w = width.saturating_sub(6) as usize;
            let out = render_markdown(&self.text, w, theme);
            self.md_cache.replace(Some(out));
            self.md_width.set(width);
        }
        self.md_cache.borrow().clone().unwrap_or_default()
    }
}

impl MessageComponent for AssistantBlock {
    fn height(&self, width: u16) -> u16 {
        let body = Self::text_count(&self.text, width);
        let mut h = 1 + body + 1;
        if !self.reasoning.is_empty() {
            h += 1;
            if self.reasoning_expanded { h += self.reasoning.lines().count() as u16; }
        }
        h
    }

    fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme, selected: bool) {
        let (ac, _) = theme.assistant_colors();
        let bg = if selected { ratatui::style::Color::Rgb(30, 40, 60) } else { theme.background() };
        let indent = "   ";
        let mut y = area.y;

        // header: "Claw:" in accent bold
        Paragraph::new(Line::from(vec![
            Span::raw(indent), Span::styled("Claw", Style::default().fg(ac).add_modifier(Modifier::BOLD)),
            Span::styled(":", Style::default().fg(theme.dim_text())),
        ])).style(Style::default().bg(bg)).render(Rect { y, height: 1, ..area }, buf);
        y += 1;

        // body
        let md = self.get_md(area.width, theme);
        let body_h = Self::text_count(&self.text, area.width);
        if !md.is_empty() {
            let h = (md.len() as u16).min((area.y + area.height).saturating_sub(y));
            if h > 0 {
                let lines: Vec<Line> = md.into_iter().take(h as usize).map(|l| {
                    let mut spans = l.spans; for s in &mut spans { s.style = s.style.fg(theme.text()); }
                    Line::from(spans)
                }).collect();
                Paragraph::new(lines).style(Style::default().bg(bg))
                    .render(Rect { y, height: h, ..area }, buf);
            }
            y += h;
        } else {
            let h = body_h.min((area.y + area.height).saturating_sub(y));
            Paragraph::new(Line::from(vec![
                Span::raw(indent), Span::styled(&self.text, Style::default().fg(theme.text())),
            ])).style(Style::default().bg(bg)).render(Rect { y, height: h, ..area }, buf);
            y += h;
        }

        // reasoning toggle
        if !self.reasoning.is_empty() {
            let icon = if self.reasoning_expanded { "▾" } else { "▸" };
            Paragraph::new(Line::from(Span::styled(
                format!("{}   {} 思考过程", indent, icon),
                Style::default().fg(theme.dim_text()).italic(),
            ))).style(Style::default().bg(bg))
              .render(Rect { y, height: 1, ..area }, buf);
            y += 1;

            if self.reasoning_expanded {
                let lines: Vec<Line> = self.reasoning.lines()
                    .map(|l| Line::from(Span::styled(
                        format!("{}     {}", indent, l),
                        Style::default().fg(theme.dim_text()).italic(),
                    ))).collect();
                let h = (lines.len() as u16).min((area.y + area.height).saturating_sub(y));
                Paragraph::new(lines).style(Style::default().bg(bg))
                    .render(Rect { y, height: h, ..area }, buf);
            }
        }
    }

    fn clickable(&self) -> bool { !self.reasoning.is_empty() }
}
