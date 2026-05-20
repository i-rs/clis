use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};

use crate::app::App;

pub(super) fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let bg = if app.is_processing() {
        Color::Blue
    } else if app.selection_mode {
        Color::Rgb(40, 30, 10)
    } else {
        app.config.theme.background()
    };

    // Fill full-width background
    f.render_widget(
        Block::default()
            .style(Style::default().bg(bg)),
        area,
    );

    let mut spans: Vec<Span> = Vec::new();

    // Copy feedback (transient, highest priority)
    if let Some(fb) = &app.copy_feedback {
        spans.push(Span::styled(
            format!(" {} ", fb),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
    }

    if app.selection_mode {
        // Selection mode indicator
        spans.push(Span::styled(
            " ● [选择模式] ".to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "↑↓选择  Space展开  Ctrl+D删除  Ctrl+Shift+C复制  退出Esc",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
    } else if app.is_processing() {
        // Processing state
        spans.push(Span::styled(
            format!(" ⏳ {} ", app.status_text),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
        // Separator
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
        // Tool & message stats
        spans.push(Span::styled(
            format!("⚙ {} ", app.tool_call_count),
            Style::default().fg(app.config.theme.primary()),
        ));
        spans.push(Span::styled(
            format!("💬 {} ", app.messages.len()),
            Style::default().fg(app.config.theme.primary()),
        ));
    } else {
        // Idle state — green dot + bold
        spans.push(Span::styled(
            " ● 就绪 ".to_string(),
            Style::default().fg(app.config.theme.secondary()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{} ", app.config.model),
            Style::default().fg(app.config.theme.primary()),
        ));
        // Separator
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
        // Tool & message stats
        spans.push(Span::styled(
            format!("⚙ {} ", app.tool_call_count),
            Style::default().fg(app.config.theme.primary()),
        ));
        spans.push(Span::styled(
            format!("💬 {} ", app.messages.len()),
            Style::default().fg(app.config.theme.primary()),
        ));
        // Today's token usage summary
        if app.today_stats.requests > 0 {
            spans.push(Span::styled(
                "│ ",
                Style::default().fg(app.config.theme.dim_text()),
            ));
            let cost = app.today_stats.cost_usd;
            if cost > 0.001 {
                spans.push(Span::styled(
                    format!("今日: {}次 {:>4}K ${:.2} ",
                        app.today_stats.requests,
                        app.today_stats.tokens / 1000,
                        cost,
                    ),
                    Style::default().fg(app.config.theme.accent()),
                ));
            } else {
                spans.push(Span::styled(
                    format!("今日: {}次 {:>4}K ",
                        app.today_stats.requests,
                        app.today_stats.tokens / 1000,
                    ),
                    Style::default().fg(app.config.theme.accent()),
                ));
            }
        }
        // Keybindings (right side)
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(app.config.theme.dim_text()),
        ));
        spans.push(Span::styled(
            "Ctrl+Q ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "Ctrl+N  ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        if !app.show_sidebar && !app.http_logs.is_empty() {
                spans.push(Span::styled(
                    "Ctrl+R  ",
                    Style::default().fg(Color::Rgb(140, 140, 160)),
                ));
            }
        spans.push(Span::styled(
            "Ctrl+L  ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "Ctrl+P ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  ",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "Ctrl+Shift+C",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  Ctrl+S",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
        spans.push(Span::styled(
            "  Ctrl+H",
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ));
    }

    let line = Line::from(spans);
    f.render_widget(line, area);
}
