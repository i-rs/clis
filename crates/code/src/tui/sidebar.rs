use crate::app::App;
use crate::tui::colors::*;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph, Wrap},
};

use super::utils::short_path;

fn fmt_count(n: u32) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{:.1}K", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

fn section_header(label: &'static str, width: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(label.to_uppercase(), Style::default().fg(c_label())),
        Span::styled(
            "─".repeat(width.saturating_sub(label.chars().count() + 1)),
            Style::default().fg(c_border()),
        ),
    ])
}

pub fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(Clear, area);
    let bg = Paragraph::new(Text::from(vec![Line::from("")])).style(Style::default().bg(c_bg_sidebar()));
    frame.render_widget(bg, area);

    // Left border (separator from chat)
    let sep_line = Paragraph::new(Text::from(vec![Line::from(Span::styled(
        "▏",
        Style::default().fg(c_border()),
    ))]))
    .style(Style::default().bg(c_bg()));
    let sep_area = Rect {
        x: area.x,
        y: area.y,
        width: 1,
        height: area.height,
    };
    frame.render_widget(sep_line, sep_area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(3),
        height: area.height,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let content_area = chunks[0];
    let w = content_area.width.saturating_sub(2) as usize;

    let mut items: Vec<Line> = Vec::with_capacity(60 + app.file_changes.len() + app.plan.len());

    // Header
    items.push(section_header(
        " Status",
        content_area.width.saturating_sub(1) as usize,
    ));

    // Directory
    items.push(Line::from(Span::styled(
        "─ Dir ─",
        Style::default().fg(c_muted()),
    )));
    let short = short_path(&app.current_dir);
    let dir = if short.len() > w.saturating_sub(2) {
        format!("..{}", &short[short.len().saturating_sub(w.saturating_sub(4))..])
    } else {
        short
    };
    items.push(Line::from(Span::styled(
        format!(" {}", dir),
        Style::default().fg(c_text()),
    )));
    items.push(Line::from(""));

    // Session & Version
    items.push(Line::from(Span::styled(
        "─ Session ─",
        Style::default().fg(c_muted()),
    )));
    let sid = app.session_id.as_deref().unwrap_or("new");
    let sid_short = if sid.len() > 10 {
        format!("{}..", &sid[..10])
    } else {
        sid.to_string()
    };
    items.push(Line::from(vec![
        Span::styled(" id ", Style::default().fg(c_label())),
        Span::styled(sid_short, Style::default().fg(c_dim())),
    ]));
    items.push(Line::from(vec![
        Span::styled(" ver", Style::default().fg(c_label())),
        Span::styled(
            format!(" {}", app.version),
            Style::default().fg(c_cyan()),
        ),
    ]));
    items.push(Line::from(""));

    // Model
    items.push(Line::from(Span::styled(
        "─ Model ─",
        Style::default().fg(c_muted()),
    )));
    let model = app.config.effective_model();
    let model_short = if model.len() > w.saturating_sub(2) {
        format!("{}..", &model[..w.saturating_sub(4)])
    } else {
        model.to_string()
    };
    items.push(Line::from(vec![
        Span::styled(" ▸ ", Style::default().fg(c_accent())),
        Span::styled(model_short, Style::default().fg(c_text())),
    ]));
    items.push(Line::from(""));

    // Tokens
    items.push(Line::from(Span::styled(
        "─ Tokens ─",
        Style::default().fg(c_muted()),
    )));
    let tok_in = app.token_usage.input;
    let tok_out = app.token_usage.output;
    items.push(Line::from(Span::styled(
        format!(" IN  {}    OUT  {}", fmt_count(tok_in), fmt_count(tok_out)),
        Style::default().fg(c_green()),
    )));
    if let Some(ref s) = app.streaming
        && !s.content.is_empty()
    {
        items.push(Line::from(Span::styled(
            format!(" streaming: ~{} chars", fmt_count(s.content.len() as u32)),
            Style::default().fg(c_cyan()),
        )));
    }
    items.push(Line::from(""));

    // Mode + Messages
    items.push(Line::from(Span::styled(
        "─ Mode ─",
        Style::default().fg(c_muted()),
    )));
    let mode_label = match app.mode {
        crate::app::AppMode::Idle => (" idle", c_status_idle()),
        crate::app::AppMode::Waiting => (" busy", c_status_busy()),
    };
    items.push(Line::from(vec![Span::styled(
        mode_label.0,
        Style::default().fg(mode_label.1),
    )]));
    items.push(Line::from(Span::styled(
        format!(" {} msgs", app.messages.len()),
        Style::default().fg(c_dim()),
    )));
    items.push(Line::from(""));

    // File changes
    if !app.file_changes.is_empty() {
        items.push(Line::from(Span::styled(
            format!("─ Files ({}) ─", app.file_changes.len()),
            Style::default().fg(c_muted()),
        )));
        for path in app.file_changes.iter() {
            let p = if path.len() > w {
                format!("..{}", &path[path.len().saturating_sub(w.saturating_sub(2))..])
            } else {
                path.clone()
            };
            items.push(Line::from(Span::styled(
                format!(" ✎ {}", p),
                Style::default().fg(c_file_edit()),
            )));
        }
        items.push(Line::from(""));
    }

    // LSP
    if crate::runtime::is_lsp_initialized() {
        items.push(Line::from(Span::styled(
            "─ LSP ─",
            Style::default().fg(c_muted()),
        )));
        let diag_count = crate::runtime::lsp_diagnostics();
        if diag_count == 0 {
            items.push(Line::from(Span::styled(
                " ✓ clean",
                Style::default().fg(c_green()),
            )));
        } else {
            items.push(Line::from(Span::styled(
                format!(" ✗ {} errors", diag_count),
                Style::default().fg(c_red()).bold(),
            )));
        }
        items.push(Line::from(""));
    }

    // MCP
    items.push(Line::from(Span::styled(
        "─ MCP ─",
        Style::default().fg(c_muted()),
    )));
    let mcp_servers = crate::runtime::mcp_connected_servers();
    if mcp_servers.is_empty() {
        items.push(Line::from(Span::styled(
            " none connected",
            Style::default().fg(c_muted()),
        )));
    } else {
        for srv in mcp_servers.iter().take(4) {
            let name = if srv.len() > w.saturating_sub(4) {
                format!("{}..", &srv[..w.saturating_sub(6)])
            } else {
                srv.clone()
            };
            items.push(Line::from(vec![
                Span::styled(" ▸ ", Style::default().fg(c_accent())),
                Span::styled(name, Style::default().fg(c_text())),
            ]));
        }
        if mcp_servers.len() > 4 {
            items.push(Line::from(Span::styled(
                format!(" +{} more", mcp_servers.len() - 4),
                Style::default().fg(c_dim()),
            )));
        }
    }
    items.push(Line::from(""));

    // Plan
    if !app.plan.is_empty() {
        items.push(Line::from(Span::styled(
            "─ Plan ─",
            Style::default().fg(c_muted()),
        )));
        for step in app.plan.iter() {
            let preview: String = step.chars().take(w.saturating_sub(6)).collect();
            items.push(Line::from(Span::styled(
                format!(" {}", preview),
                Style::default().fg(c_cyan()),
            )));
        }
        items.push(Line::from(""));
    }

    // Streaming / Live
    if let Some(ref s) = app.streaming {
        items.push(Line::from(Span::styled(
            "─ Live ─",
            Style::default().fg(c_muted()),
        )));
        items.push(Line::from(Span::styled(
            format!(" {} tools done", s.tool_calls.len()),
            Style::default().fg(c_cyan()),
        )));
        if let Some(ref tool) = s.current_tool {
            items.push(Line::from(vec![
                Span::styled(" ▸ ", Style::default().fg(c_orange())),
                Span::styled(&tool.name, Style::default().fg(c_orange()).bold()),
            ]));
            let args_preview: String = tool.args.chars().take(w.saturating_sub(4)).collect();
            for line in args_preview.lines().take(3) {
                items.push(Line::from(Span::styled(
                    format!("   {}", line),
                    Style::default().fg(c_muted()),
                )));
            }
        } else if !s.reasoning.is_empty() {
            items.push(Line::from(Span::styled(
                " thinking...",
                Style::default().fg(c_cyan()),
            )));
        } else if s.content.is_empty() {
            items.push(Line::from(Span::styled(
                " connecting...",
                Style::default().fg(c_cyan()),
            )));
        }
        items.push(Line::from(""));
    }

    // Status
    if let Some(ref msg) = app.status_message {
        items.push(Line::from(Span::styled(
            "─ Status ─",
            Style::default().fg(c_muted()),
        )));
        let preview: String = msg.chars().take(w.saturating_sub(4)).collect();
        items.push(Line::from(Span::styled(
            format!(" {}", preview),
            Style::default().fg(c_orange()),
        )));
    }

    let sidebar_max = items.len().saturating_sub(content_area.height as usize);
    let sidebar_scroll = app.sidebar_scroll.min(sidebar_max);
    let paragraph = Paragraph::new(Text::from(items))
        .wrap(Wrap { trim: false })
        .scroll((sidebar_scroll as u16, 0));
    frame.render_widget(paragraph, content_area);

    // Fixed scroll indicator at bottom
    let indicator_area = chunks[1];
    if sidebar_max > 0 {
        let scroll_indicator = if sidebar_scroll > 0 {
            format!(" ⇡({}/{})", sidebar_scroll, sidebar_max)
        } else {
            " scroll ↓".to_string()
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                scroll_indicator,
                Style::default().fg(c_muted()),
            ))),
            indicator_area,
        );
    } else {
        frame.render_widget(Clear, indicator_area);
    }
}
