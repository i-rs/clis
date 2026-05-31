use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
};

use crate::app::App;

/// Renders the title bar at the top of the TUI.
///
/// Design: Clean, minimal header with subtle gradient-like effect
/// using box-drawing characters. The left side shows the app identity,
/// center shows status, and right side shows the current model.
pub(super) fn render_title(f: &mut Frame, area: Rect, app: &App) {
    // Background block with theme color
    f.render_widget(
        Block::default().style(Style::default().bg(app.config.theme.background())),
        area,
    );

    let theme = &app.config.theme;
    let primary = theme.primary();
    let dim = theme.dim_text();

    let mut spans: Vec<Span> = Vec::new();

    // Left decorative element - subtle gradient bar
    spans.push(Span::styled("▎", Style::default().fg(primary)));

    // App name with subtle glow effect via repeated chars
    spans.push(Span::styled(
        " ✦ i-rs-claw ",
        Style::default().fg(primary).add_modifier(Modifier::BOLD),
    ));

    // Separator
    spans.push(Span::styled("│", Style::default().fg(dim)));

    // Center content - status or description
    if app.is_processing() {
        const SPINNERS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
        let spinner = SPINNERS[f.count() % SPINNERS.len()];
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

    // Calculate right-side content
    let model_text = format!(" {} ", app.config.model);
    let model_width = unicode_width::UnicodeWidthStr::width(model_text.as_str());

    // Calculate total width of left+center content
    let left_width = {
        let mut total = 0usize;
        for s in &spans {
            total += unicode_width::UnicodeWidthStr::width(s.content.as_ref());
        }
        total
    };

    // Fill remaining space with padding
    let padding = (area.width as usize)
        .saturating_sub(left_width)
        .saturating_sub(model_width)
        .saturating_sub(2); // Extra space for balance

    if padding > 0 {
        spans.push(Span::styled(" ".repeat(padding), Style::default()));
    }

    // Right side - model name with subtle styling
    spans.push(Span::styled(
        model_text,
        Style::default()
            .fg(Color::Rgb(80, 80, 100))
            .add_modifier(Modifier::BOLD),
    ));

    // Right decorative element - matches left
    spans.push(Span::styled("▍", Style::default().fg(primary)));

    let line = Line::from(spans);
    f.render_widget(line, area);
}
