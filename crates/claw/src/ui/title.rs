use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};

use crate::app::App;

pub(super) fn render_title(f: &mut Frame, area: Rect, app: &App) {
    // Full-width background
    f.render_widget(
        Block::default()
            .style(Style::default().bg(app.config.theme.background())),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    // App name — standout
    spans.push(Span::styled(
        " ✦ i-rs-claw",
        Style::default()
            .fg(app.config.theme.primary())
            .add_modifier(Modifier::BOLD),
    ));

    if app.is_processing() {
        // Processing indicator
        spans.push(Span::styled(
            format!("  ⏳ {} ", app.status_text),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    } else {
        // Tagline + agent name
        spans.push(Span::styled(
            format!("  个人数据智能助理  [{}]", app.current_agent),
            Style::default().fg(Color::Rgb(180, 180, 200)),
        ));
    }

    // Right-aligned model name (padded to fill width)
    let model_text = format!(" {} ", app.config.model);
    let model_text_ref: &str = &model_text;
    let model_width = unicode_width::UnicodeWidthStr::width(model_text_ref);
    let padding = (area.width as usize).saturating_sub(
        spans.iter().map(|s| {
            let content: &str = &s.content;
            unicode_width::UnicodeWidthStr::width(content)
        }).sum::<usize>()
        + model_width
        + 2, // buffer
    );
    if padding > 0 {
        spans.push(Span::styled(
            " ".repeat(padding),
            Style::default(),
        ));
    }
    spans.push(Span::styled(
        model_text,
        Style::default()
            .fg(Color::Rgb(100, 100, 130))
            .add_modifier(Modifier::BOLD),
    ));

    let line = Line::from(spans);
    f.render_widget(line, area);
}
