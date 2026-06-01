use std::sync::OnceLock;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
};

use crate::app::{App, spinner_char};

static PADDING_CACHE: OnceLock<String> = OnceLock::new();

fn get_padding(width: usize) -> &'static str {
    let s = PADDING_CACHE.get_or_init(|| " ".repeat(256));
    &s[..width.min(s.len())]
}

pub(super) fn render_title(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(
        Block::default().style(Style::default().bg(app.config.theme.background())),
        area,
    );

    let theme = &app.config.theme;
    let primary = theme.primary();
    let dim = theme.dim_text();

    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::styled("▎", Style::default().fg(primary)));

    spans.push(Span::styled(
        " ✦ i-rs-claw ",
        Style::default().fg(primary).add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled("│", Style::default().fg(dim)));

    if app.is_processing() {
        let spinner = spinner_char(app.spinner_start);
        spans.push(Span::styled(
            format!(" {} {} ", spinner, app.status_text),
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            format!(" {} ", app.current_agent),
            Style::default()
                .fg(Color::Rgb(180, 180, 200))
                .add_modifier(Modifier::BOLD),
        ));
    }

    let model_text = format!(" {} ", app.config.model);
    let model_width = unicode_width::UnicodeWidthStr::width(model_text.as_str());

    let left_width = {
        let mut total = 0usize;
        for s in &spans {
            total += unicode_width::UnicodeWidthStr::width(s.content.as_ref());
        }
        total
    };

    let padding = (area.width as usize)
        .saturating_sub(left_width)
        .saturating_sub(model_width)
        .saturating_sub(2);

    if padding > 0 {
        spans.push(Span::styled(get_padding(padding), Style::default()));
    }

    spans.push(Span::styled(
        model_text,
        Style::default()
            .fg(Color::Rgb(80, 80, 100))
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled("▍", Style::default().fg(primary)));

    let line = Line::from(spans);
    f.render_widget(line, area);
}
