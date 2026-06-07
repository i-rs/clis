use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use i_rs_claw_core::theme::Theme;

pub(super) fn render_markdown(text: &str, max_width: usize, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    struct MdLine {
        spans: Vec<(String, Style)>,
        width: usize,
    }
    impl MdLine {
        fn new() -> Self {
            Self {
                spans: Vec::new(),
                width: 0,
            }
        }
        fn add(&mut self, text: &str, style: Style) {
            self.width += UnicodeWidthStr::width(text);
            if let Some(last) = self.spans.last_mut()
                && last.1 == style
            {
                last.0.push_str(text);
                return;
            }
            self.spans.push((text.to_string(), style));
        }
        fn flush(&mut self, out: &mut Vec<Line<'static>>, max_width: usize) {
            if self.spans.is_empty() {
                return;
            }
            if self.width <= max_width {
                let spans: Vec<Span> = self
                    .spans
                    .drain(..)
                    .map(|(t, s)| Span::styled(t, s))
                    .collect();
                out.push(Line::from(spans));
            } else {
                for (text, style) in &self.spans {
                    for w in crate::ui::utils::wrap_text(text, max_width) {
                        out.push(Line::from(Span::styled(w, *style)));
                    }
                }
                self.spans.clear();
            }
        }
    }

    let mut acc = MdLine::new();
    let mut bold = false;
    let mut italic = false;
    let mut in_code_block = false;
    let mut code_text = String::new();
    let mut code_lang = String::new();

    let text_color = theme.text();
    let heading_h1 = theme.primary();
    let heading_h2 = theme.secondary();
    let heading_h3 = theme.dim_text();
    let list_bullet = theme.accent();
    let code_bg = theme.code_bg();
    let inline_code_fg = theme.accent();
    let inline_code_bg = Color::Rgb(30, 30, 30);
    let code_text_color = Color::Rgb(220, 180, 120);
    let code_border = theme.dim_text();
    let hr_color = Color::DarkGray;

    for event in Parser::new(text) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    acc.flush(&mut lines, max_width);
                    let heading_color = match level as u8 {
                        1 => heading_h1,
                        2 => heading_h2,
                        _ => heading_h3,
                    };
                    let prefix = if level as u8 <= 3 && level as u8 > 0 {
                        format!("{} ", "#".repeat(level as usize))
                    } else {
                        String::new()
                    };
                    acc.add(
                        &prefix,
                        Style::default()
                            .fg(heading_color)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                Tag::List(_) => {}
                Tag::Item => {
                    acc.flush(&mut lines, max_width);
                    acc.add("▸ ", Style::default().fg(list_bullet));
                }
                Tag::Emphasis => italic = true,
                Tag::Strong => bold = true,
                Tag::CodeBlock(kind) => {
                    acc.flush(&mut lines, max_width);
                    in_code_block = true;
                    code_text.clear();
                    code_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        _ => String::new(),
                    };
                }
                Tag::Link { .. } => {}
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item => {
                    acc.flush(&mut lines, max_width)
                }
                TagEnd::Emphasis => italic = false,
                TagEnd::Strong => bold = false,
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    if code_text.lines().any(|l| !l.trim().is_empty()) {
                        let label = if code_lang.is_empty() {
                            " code ".to_string()
                        } else {
                            format!(" {} ", code_lang)
                        };
                        let separator = format!("{:─^width$}", label, width = max_width.min(40));
                        lines.push(Line::from(Span::styled(
                            separator,
                            Style::default().fg(code_border).bg(code_bg),
                        )));
                        for cl in code_text.lines() {
                            lines.push(Line::from(Span::styled(
                                format!("  {}", cl),
                                Style::default().fg(code_text_color).bg(code_bg),
                            )));
                        }
                        lines.push(Line::from(Span::styled(
                            String::new(),
                            Style::default().bg(code_bg),
                        )));
                    }
                }
                TagEnd::Link | TagEnd::List(_) => {}
                _ => {}
            },
            Event::Text(t) => {
                if in_code_block {
                    code_text.push_str(&t);
                } else {
                    let mut style = Style::default().fg(text_color);
                    if bold {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if italic {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    acc.add(&t, style);
                }
            }
            Event::Code(t) => acc.add(&t, Style::default().fg(inline_code_fg).bg(inline_code_bg)),
            Event::SoftBreak | Event::HardBreak => acc.flush(&mut lines, max_width),
            Event::Rule => {
                acc.flush(&mut lines, max_width);
                lines.push(Line::from(Span::styled(
                    "  ─────────────────────────────────",
                    Style::default().fg(hr_color),
                )));
            }
            _ => {}
        }
    }
    acc.flush(&mut lines, max_width);
    lines
}
