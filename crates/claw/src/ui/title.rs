use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};

use crate::app::App;

pub(super) fn render_title(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(
        Block::default()
            .style(Style::default().bg(app.config.theme.background())),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::styled(
        "╺",
        Style::default().fg(app.config.theme.primary()),
    ));

    spans.push(Span::styled(
        " ✦ i-rs-claw ",
        Style::default()
            .fg(app.config.theme.primary())
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled(
        "│",
        Style::default().fg(app.config.theme.dim_text()),
    ));

    if app.is_processing() {
        const SPINNERS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
        let spinner = SPINNERS[f.count() % SPINNERS.len()];
        spans.push(Span::styled(
            format!(" {} {} ", spinner, app.status_text),
            Style::default().fg(app.config.theme.accent()).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            format!(" 个人数据智能助理 [{}] ", app.current_agent),
            Style::default().fg(Color::Rgb(180, 180, 200)),
        ));
    }

    let model_text = format!(" {} ", app.config.model);
    let model_text_ref: &str = &model_text;
    let model_width = unicode_width::UnicodeWidthStr::width(model_text_ref);
    let padding = (area.width as usize).saturating_sub(
        spans.iter().map(|s| {
            let content: &str = &s.content;
            unicode_width::UnicodeWidthStr::width(content)
        }).sum::<usize>()
        + model_width
        + 2,
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

    spans.push(Span::styled(
        "╸",
        Style::default().fg(app.config.theme.primary()),
    ));

    let line = Line::from(spans);
    f.render_widget(line, area);
}
