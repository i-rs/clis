use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};

use crate::app::App;

/// Renders the status bar at the bottom of the TUI.
///
/// Design: Minimal information-dense footer with clear visual hierarchy.
/// Left side shows transient feedback or mode indicators.
/// Center shows system stats.
/// Right side shows available keyboard shortcuts.
pub(super) fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.config.theme;

    // Background color based on state
    let bg = if app.is_processing() {
        Color::Rgb(15, 15, 30)
    } else if app.overlay.selection_mode {
        Color::Rgb(30, 25, 15)
    } else {
        theme.background()
    };

    // Fill full-width background
    f.render_widget(
        Block::default()
            .style(Style::default().bg(bg)),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    // Copy feedback (transient, highest priority)
    if let Some(fb) = &app.overlay.copy_feedback {
        spans.push(Span::styled(
            format!(" {} ", fb),
            Style::default().fg(theme.secondary()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(theme.dim_text()),
        ));
    }

    if app.overlay.selection_mode {
        // Selection mode indicator
        spans.push(Span::styled(
            " ● [选择模式] ".to_string(),
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "↑↓选择  Space展开  Ctrl+D删除  Ctrl+Shift+C复制  退出Esc",
            Style::default().fg(theme.dim_text()),
        ));
    } else if app.is_processing() {
        const SPINNERS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
        let spinner = SPINNERS[f.count() % SPINNERS.len()];
        spans.push(Span::styled(
            format!(" {} {} ", spinner, app.status_text),
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),  // Amber for active processing
        ));
        // Separator
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(theme.dim_text()),
        ));
        // Tool & message stats
        spans.push(Span::styled(
            format!("⚙ {} ", app.tool_call_count),
            Style::default().fg(theme.primary()),
        ));
        spans.push(Span::styled(
            format!("💬 {} ", app.messages.len()),
            Style::default().fg(theme.primary()),
        ));
    } else {
        // Idle state — cyan dot + clean styling
        spans.push(Span::styled(
            " ● 就绪 ".to_string(),
            Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{} ", app.config.model),
            Style::default().fg(theme.dim_text()),
        ));
        // Separator
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(theme.dim_text()),
        ));
        // Tool & message stats
        spans.push(Span::styled(
            format!("⚙ {} ", app.tool_call_count),
            Style::default().fg(theme.primary()),
        ));
        spans.push(Span::styled(
            format!("💬 {} ", app.messages.len()),
            Style::default().fg(theme.primary()),
        ));
        // Today's token usage summary
        if app.today_stats.requests > 0 {
            spans.push(Span::styled(
                "│ ",
                Style::default().fg(theme.dim_text()),
            ));
            let cost = app.today_stats.cost_usd;
            if cost > 0.001 {
                spans.push(Span::styled(
                    format!("今日: {}次 {:>4}K ${:.2} ",
                        app.today_stats.requests,
                        app.today_stats.tokens / 1000,
                        cost,
                    ),
                    Style::default().fg(theme.accent()),  // Amber
                ));
            } else {
                spans.push(Span::styled(
                    format!("今日: {}次 {:>4}K ",
                        app.today_stats.requests,
                        app.today_stats.tokens / 1000,
                    ),
                    Style::default().fg(theme.accent()),
                ));
            }
        }
        // Keybindings (right side) - more subtle
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "Ctrl+Q ",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "Ctrl+N  ",
            Style::default().fg(theme.dim_text()),
        ));
        if !app.overlay.show_sidebar && !app.http_logs.is_empty() {
                spans.push(Span::styled(
                    "Ctrl+R  ",
                    Style::default().fg(theme.dim_text()),
                ));
            }
        spans.push(Span::styled(
            "Ctrl+L  ",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "Ctrl+P ",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "  ",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "Ctrl+Shift+C",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "  Ctrl+S",
            Style::default().fg(theme.dim_text()),
        ));
        spans.push(Span::styled(
            "  Ctrl+H",
            Style::default().fg(theme.dim_text()),
        ));
    }

    let line = Line::from(spans);
    f.render_widget(line, area);
}
